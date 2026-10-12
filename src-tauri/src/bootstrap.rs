//! Self-bootstrap of the Steamworks runtime library.
//!
//! Valve's Steamworks SDK is closed-source and only redistributed as a
//! shared library (steam_api64.dll on Windows, libsteam_api.dylib on macOS,
//! libsteam_api.so on Linux). The license permits redistribution alongside
//! the application but forbids static linking.
//!
//! To keep the user-facing artifact a single executable file (no sidecar
//! .dll or .dylib), we embed the runtime library in the binary at compile
//! time (`include_bytes!`) and extract it to a stable user-writable
//! location on first launch. Subsequent runs reuse the cached copy.
//!
//! Each SAM version extracts into its own `steamworks/<version>/` directory
//! (see `extract`), so an upgrade never rewrites a library that an older SAM,
//! still running, has loaded.
//!
//! All extraction targets `%LOCALAPPDATA%` (Windows) or `~/Library/
//! Application Support` (macOS) - standard user-writable app data
//! directories. Nothing touches `%TEMP%`, system32, or anything that
//! would look suspicious to anti-virus heuristics.
//!
//! Loading the copy differs. Windows links steam_api64.dll with /DELAYLOAD
//! (build.rs): the import resolves at the first Steam call, after `bootstrap`
//! has added the copy's directory to the DLL search path. Linux and macOS
//! resolve linked libraries before `main` runs, when there is no copy yet, so
//! there SAM does not need the library to start. build.rs defines a stub in
//! SAM for every function the library exports, each a jump through a table,
//! and `bootstrap` loads the copy with `dlopen` and fills the table. With
//! nothing left for the library to provide, the Linux linker drops it, and
//! macOS links it weak.

use std::path::PathBuf;
use std::sync::OnceLock;

/// What `bootstrap` achieved, for `ready`.
static LOADED: OnceLock<Result<(), String>> = OnceLock::new();

/// Extracts the Steamworks library and makes it the one Steam calls reach.
/// Runs first thing in `main`, before anything calls into Steam.
pub fn bootstrap() {
    let result =
        install().map_err(|e| format!("SAM could not load Valve's Steamworks library: {e}"));
    if let Err(e) = &result {
        eprintln!("[bootstrap] {e}");
    }
    let _ = LOADED.set(result);
}

/// Ok once Steam can be called. A Steam call without the library crashes SAM.
pub fn ready() -> Result<(), String> {
    LOADED
        .get()
        .cloned()
        .unwrap_or_else(|| Err("the Steamworks library was never loaded".to_string()))
}

#[cfg(target_os = "windows")]
fn install() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    const DLL_BYTES: &[u8] = include_bytes!("../vendor/steam_api64.dll");
    const DLL_NAME: &str = "steam_api64.dll";

    let local_appdata = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?;
    let base = PathBuf::from(local_appdata).join("SAM-Colony-Edition");
    let dir = extract(base, DLL_NAME, DLL_BYTES)?;

    // SetDllDirectoryW adds `dir` to the standard DLL search path for the
    // current process. Combined with the /DELAYLOAD linker flag, the first
    // Steamworks call triggers a DLL load that finds our extracted copy.
    let mut wide: Vec<u16> = dir.as_os_str().encode_wide().collect();
    wide.push(0);
    if unsafe { windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW(wide.as_ptr()) } == 0 {
        return Err("SetDllDirectoryW failed".to_string());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
const LIBRARY: (&str, &[u8]) = (
    "libsteam_api.dylib",
    include_bytes!("../vendor/libsteam_api.dylib"),
);
#[cfg(target_os = "linux")]
const LIBRARY: (&str, &[u8]) = (
    "libsteam_api.so",
    include_bytes!("../vendor/libsteam_api.so"),
);

#[cfg(unix)]
fn install() -> Result<(), String> {
    let (name, bytes) = LIBRARY;
    let dir = extract(data_dir()?.join("SAM-Colony-Edition"), name, bytes)?;
    stubs::load(&dir.join(name))
}

#[cfg(target_os = "macos")]
fn data_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join("Library/Application Support"))
}

/// XDG_DATA_HOME, or its default ~/.local/share.
#[cfg(target_os = "linux")]
fn data_dir() -> Result<PathBuf, String> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or_else(|| "neither XDG_DATA_HOME nor HOME is set".to_string())
}

/// The stubs build.rs defines for every Steamworks function, and the table
/// they jump through.
#[cfg(unix)]
mod stubs {
    use std::ffi::{c_void, CStr, CString};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;
    use std::sync::atomic::{AtomicPtr, Ordering};

    // NAMES, and the stub for each, which jumps to the address at the same
    // index in TABLE.
    include!(concat!(env!("OUT_DIR"), "/steam_stubs.rs"));

    /// Null until `load`: a stub called before that jumps to address 0.
    static TABLE: [AtomicPtr<c_void>; NAMES.len()] =
        [const { AtomicPtr::new(std::ptr::null_mut()) }; NAMES.len()];

    /// Loads the library at `path` and points every stub at its function.
    pub(super) fn load(path: &Path) -> Result<(), String> {
        let path_c = CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
        // RTLD_GLOBAL puts its symbols in the global scope, as linking it did.
        // It is never unloaded.
        let lib = unsafe { libc::dlopen(path_c.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if lib.is_null() {
            let reason = unsafe { libc::dlerror() };
            return Err(if reason.is_null() {
                format!("could not load {}", path.display())
            } else {
                unsafe { CStr::from_ptr(reason) }
                    .to_string_lossy()
                    .into_owned()
            });
        }
        for (slot, name) in TABLE.iter().zip(NAMES) {
            // Null for a function this build lacks (SteamRealPath on macOS),
            // which steamworks-rs never calls.
            slot.store(
                unsafe { libc::dlsym(lib, name.as_ptr()) },
                Ordering::Relaxed,
            );
        }
        Ok(())
    }
}

/// Extracts `bytes` as `name` into `base/steamworks/<SAM version>/` and
/// returns that directory, or why it could not.
///
/// The directory is per version because a running SAM keeps its copy loaded.
/// Rewriting that file in place on upgrade crashes the older process (SIGBUS
/// on Linux, a code-signature kill on macOS), and on Windows the write fails
/// and leaves the new process without its DLL. The write goes through a
/// temporary file and a rename, so a process loading the library never sees
/// a half-written file and, on Unix, an older mapping keeps its own inode.
// ponytail: copies left by older versions (and the one that releases before
// this layout put directly in `base`) are never pruned, a few hundred KB per
// release. Prune them at startup if that ever matters; on Windows a copy can
// be in use by a running SAM that has not delay-loaded it yet, so pruning
// must tolerate that.
fn extract(base: PathBuf, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let dir = base.join("steamworks").join(env!("CARGO_PKG_VERSION"));
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create {}: {e}", dir.display()))?;

    let path = dir.join(name);
    if needs_write(&path, bytes) {
        let tmp = dir.join(format!("{}.{}.tmp", name, std::process::id()));
        if let Err(e) = std::fs::write(&tmp, bytes).and_then(|()| std::fs::rename(&tmp, &path)) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("failed to write {}: {e}", path.display()));
        }
    }
    Ok(dir)
}

/// Returns true if the path doesn't exist or its contents differ from `expected`.
/// Avoids rewriting when the embedded blob is already on disk.
fn needs_write(path: &std::path::Path, expected: &[u8]) -> bool {
    match std::fs::read(path) {
        Ok(existing) => existing != expected,
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn extracts_into_a_directory_per_version() {
        let base = std::env::temp_dir().join(format!("sam-bootstrap-test-{}", std::process::id()));
        let older = base.join("steamworks").join("0.0.0");
        std::fs::create_dir_all(&older).unwrap();
        std::fs::write(older.join("lib"), b"old").unwrap();

        let dir = super::extract(base.clone(), "lib", b"new").unwrap();
        assert_eq!(dir, base.join("steamworks").join(env!("CARGO_PKG_VERSION")));
        assert_eq!(std::fs::read(dir.join("lib")).unwrap(), b"new");
        // The older version's copy, possibly loaded by a running SAM, is untouched.
        assert_eq!(std::fs::read(older.join("lib")).unwrap(), b"old");

        // A second launch reuses the copy and leaves no temporary file behind.
        assert_eq!(super::extract(base.clone(), "lib", b"new"), Ok(dir.clone()));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn steam_calls_reach_the_loaded_library() {
        use steamworks_sys as sys;

        let base = std::env::temp_dir().join(format!("sam-load-test-{}", std::process::id()));
        let (name, bytes) = super::LIBRARY;
        let dir = super::extract(base.clone(), name, bytes).unwrap();
        super::stubs::load(&dir.join(name)).unwrap();

        // Through the stubs to plain functions of the library, which need no
        // Steam client: arguments in, results out.
        unsafe {
            let mut address: sys::SteamNetworkingIPAddr = std::mem::zeroed();
            sys::SteamAPI_SteamNetworkingIPAddr_SetIPv4(&mut address, 0x7f00_0001, 27015);
            assert_eq!(
                sys::SteamAPI_SteamNetworkingIPAddr_GetIPv4(&mut address),
                0x7f00_0001
            );
            assert!(sys::SteamAPI_SteamNetworkingIPAddr_IsLocalHost(
                &mut address
            ));
        }

        std::fs::remove_dir_all(&base).unwrap();
    }
}

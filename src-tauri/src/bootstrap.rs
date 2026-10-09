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

#[cfg(target_os = "windows")]
pub fn bootstrap() {
    use std::os::windows::ffi::OsStrExt;

    const DLL_BYTES: &[u8] = include_bytes!("../vendor/steam_api64.dll");
    const DLL_NAME: &str = "steam_api64.dll";

    let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") else {
        eprintln!("[bootstrap] LOCALAPPDATA not set, skipping DLL bootstrap");
        return;
    };
    let base = std::path::PathBuf::from(local_appdata).join("SAM-Colony-Edition");
    let Some(dir) = extract(base, DLL_NAME, DLL_BYTES) else {
        return;
    };

    // SetDllDirectoryW adds `dir` to the standard DLL search path for the
    // current process. Combined with the /DELAYLOAD linker flag, the first
    // Steamworks call triggers a DLL load that finds our extracted copy.
    let mut wide: Vec<u16> = dir.as_os_str().encode_wide().collect();
    wide.push(0);
    unsafe {
        if windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW(wide.as_ptr()) == 0 {
            eprintln!("[bootstrap] SetDllDirectoryW failed");
        }
    }
}

#[cfg(target_os = "macos")]
pub fn bootstrap() {
    use std::os::unix::process::CommandExt;

    const DYLIB_BYTES: &[u8] = include_bytes!("../vendor/libsteam_api.dylib");
    const DYLIB_NAME: &str = "libsteam_api.dylib";
    const BOOTSTRAPPED_ENV: &str = "SAM_COLONY_BOOTSTRAPPED";

    // Already re-exec'd with DYLD_LIBRARY_PATH set: dyld will find the
    // dylib, nothing else to do.
    if std::env::var_os(BOOTSTRAPPED_ENV).is_some() {
        return;
    }

    let Some(home) = std::env::var_os("HOME") else {
        eprintln!("[bootstrap] HOME not set, skipping dylib bootstrap");
        return;
    };
    let base =
        std::path::PathBuf::from(home).join("Library/Application Support/SAM-Colony-Edition");
    let Some(dir) = extract(base, DYLIB_NAME, DYLIB_BYTES) else {
        return;
    };

    // dyld reads DYLD_LIBRARY_PATH at process start, so we re-exec ourselves
    // with it set. The flag env var stops infinite recursion.
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[bootstrap] current_exe failed: {}", e);
            return;
        }
    };
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let err = std::process::Command::new(&current_exe)
        .args(&args)
        .env(BOOTSTRAPPED_ENV, "1")
        .env("DYLD_LIBRARY_PATH", &dir)
        .exec();
    eprintln!("[bootstrap] re-exec failed: {}", err);
    std::process::exit(1);
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn bootstrap() {
    use std::os::unix::process::CommandExt;

    const SO_BYTES: &[u8] = include_bytes!("../vendor/libsteam_api.so");
    const SO_NAME: &str = "libsteam_api.so";
    const BOOTSTRAPPED_ENV: &str = "SAM_COLONY_BOOTSTRAPPED";

    if std::env::var_os(BOOTSTRAPPED_ENV).is_some() {
        return;
    }

    // Prefer XDG_DATA_HOME, fall back to ~/.local/share (XDG default).
    let dir = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        });
    let Some(dir) = dir else {
        eprintln!("[bootstrap] neither XDG_DATA_HOME nor HOME set, skipping .so bootstrap");
        return;
    };
    let Some(dir) = extract(dir.join("SAM-Colony-Edition"), SO_NAME, SO_BYTES) else {
        return;
    };

    // ld.so reads LD_LIBRARY_PATH at process start, so re-exec with it set.
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[bootstrap] current_exe failed: {}", e);
            return;
        }
    };
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    // Prepend our dir to any pre-existing LD_LIBRARY_PATH.
    let new_ld_path = match std::env::var_os("LD_LIBRARY_PATH") {
        Some(existing) => {
            let mut s = std::ffi::OsString::from(&dir);
            s.push(":");
            s.push(existing);
            s
        }
        None => std::ffi::OsString::from(&dir),
    };
    let err = std::process::Command::new(&current_exe)
        .args(&args)
        .env(BOOTSTRAPPED_ENV, "1")
        .env("LD_LIBRARY_PATH", new_ld_path)
        .exec();
    eprintln!("[bootstrap] re-exec failed: {}", err);
    std::process::exit(1);
}

/// Extracts `bytes` as `name` into `base/steamworks/<SAM version>/` and
/// returns that directory, or logs why it could not and returns None.
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
fn extract(base: std::path::PathBuf, name: &str, bytes: &[u8]) -> Option<std::path::PathBuf> {
    let dir = base.join("steamworks").join(env!("CARGO_PKG_VERSION"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[bootstrap] failed to create {}: {}", dir.display(), e);
        return None;
    }

    let path = dir.join(name);
    if needs_write(&path, bytes) {
        let tmp = dir.join(format!("{}.{}.tmp", name, std::process::id()));
        if let Err(e) = std::fs::write(&tmp, bytes).and_then(|()| std::fs::rename(&tmp, &path)) {
            let _ = std::fs::remove_file(&tmp);
            eprintln!("[bootstrap] failed to write {}: {}", path.display(), e);
            return None;
        }
    }
    Some(dir)
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
        assert_eq!(
            super::extract(base.clone(), "lib", b"new"),
            Some(dir.clone())
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);

        std::fs::remove_dir_all(&base).unwrap();
    }
}

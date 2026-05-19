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
    let dir = std::path::PathBuf::from(local_appdata).join("SAM-Colony-Edition");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[bootstrap] failed to create {}: {}", dir.display(), e);
        return;
    }

    let dll_path = dir.join(DLL_NAME);
    if needs_write(&dll_path, DLL_BYTES) {
        if let Err(e) = std::fs::write(&dll_path, DLL_BYTES) {
            eprintln!("[bootstrap] failed to write {}: {}", dll_path.display(), e);
            return;
        }
    }

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
    let dir = std::path::PathBuf::from(home)
        .join("Library/Application Support/SAM-Colony-Edition");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[bootstrap] failed to create {}: {}", dir.display(), e);
        return;
    }

    let dylib_path = dir.join(DYLIB_NAME);
    if needs_write(&dylib_path, DYLIB_BYTES) {
        if let Err(e) = std::fs::write(&dylib_path, DYLIB_BYTES) {
            eprintln!("[bootstrap] failed to write {}: {}", dylib_path.display(), e);
            return;
        }
    }

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
    let dir = dir.join("SAM-Colony-Edition");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[bootstrap] failed to create {}: {}", dir.display(), e);
        return;
    }

    let so_path = dir.join(SO_NAME);
    if needs_write(&so_path, SO_BYTES) {
        if let Err(e) = std::fs::write(&so_path, SO_BYTES) {
            eprintln!("[bootstrap] failed to write {}: {}", so_path.display(), e);
            return;
        }
    }

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

/// Returns true if the path doesn't exist or its contents differ from `expected`.
/// Avoids rewriting when the embedded blob is already on disk.
fn needs_write(path: &std::path::Path, expected: &[u8]) -> bool {
    match std::fs::read(path) {
        Ok(existing) => existing != expected,
        Err(_) => true,
    }
}

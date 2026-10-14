//! Where SAM keeps its files: the shared Colony layout.
//!
//! This mirrors `colony_ui::paths` from Project-Colony-Resources
//! (design/filesystem.md). SAM is a Tauri app and colony-ui is built on iced, so
//! the table is reproduced here rather than imported. Keep it identical.
//!
//! | Kind | Linux | Windows | macOS |
//! |---|---|---|---|
//! | config | `~/.config/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | `~/Library/Application Support/Colony/SAM/` |
//! | data | `~/.local/share/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | `~/Library/Application Support/Colony/SAM/` |
//! | cache | `~/.cache/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\cache\` | `~/Library/Caches/Colony/SAM/` |
//!
//! Windows uses `AppData\Local`, never `Roaming`. Config and data are the same
//! directory on Windows and macOS, so files are kept apart by name or
//! sub-directory, never by relying on the roots differing.

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

/// SAM's directory name under `Colony/`, spelled as the program spells itself.
pub const PROGRAM: &str = "SAM";

const VENDOR: &str = "Colony";

/// `<data>/Colony/apps/` is the shared install root of the Colony launcher, so
/// no program may take that name.
const APPS: &str = "apps";

/// Windows has no cache root of its own: the cache sits in LocalAppData next to
/// config and data, so this component keeps "clear the cache" from deleting
/// them.
const CACHE: &str = "cache";

/// Accept only a single, safe path component that is not the reserved `apps`.
///
/// A trailing dot or space is refused because Windows strips it, so `SAM.`
/// would open the same folder as `SAM`. `apps` is compared ignoring case
/// because Windows and default macOS filesystems do.
fn program_component(name: &str) -> io::Result<&str> {
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', '\0'])
        || name.ends_with(['.', ' '])
        || Path::new(name).components().count() != 1
        || Path::new(name).file_name() != Some(OsStr::new(name))
        || name.to_uppercase().eq_ignore_ascii_case(APPS);
    if invalid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name:?} cannot name a Colony program directory"),
        ));
    }
    Ok(name)
}

fn missing(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("cannot determine the platform {what} directory"),
    )
}

/// The same paths without creating them, for display.
pub mod locate {
    use super::*;

    /// See [`super::config_dir`].
    pub fn config_dir(program: &str) -> io::Result<PathBuf> {
        Ok(dirs::config_local_dir()
            .ok_or_else(|| missing("config"))?
            .join(VENDOR)
            .join(program_component(program)?))
    }

    /// See [`super::data_dir`].
    pub fn data_dir(program: &str) -> io::Result<PathBuf> {
        Ok(dirs::data_local_dir()
            .ok_or_else(|| missing("data"))?
            .join(VENDOR)
            .join(program_component(program)?))
    }

    /// See [`super::cache_dir`].
    pub fn cache_dir(program: &str) -> io::Result<PathBuf> {
        let base = dirs::cache_dir()
            .ok_or_else(|| missing("cache"))?
            .join(VENDOR)
            .join(program_component(program)?);
        Ok(if cfg!(windows) { base.join(CACHE) } else { base })
    }
}

fn ensure(path: PathBuf) -> io::Result<PathBuf> {
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// What the user chose (preferences), created if missing.
pub fn config_dir(program: &str) -> io::Result<PathBuf> {
    ensure(locate::config_dir(program)?)
}

/// What the program produced and cannot re-derive, created if missing.
pub fn data_dir(program: &str) -> io::Result<PathBuf> {
    ensure(locate::data_dir(program)?)
}

/// What the program can rebuild by asking again, created if missing.
pub fn cache_dir(program: &str) -> io::Result<PathBuf> {
    ensure(locate::cache_dir(program)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_root_nests_under_colony_then_the_program() {
        for path in [
            locate::config_dir(PROGRAM).unwrap(),
            locate::data_dir(PROGRAM).unwrap(),
            locate::cache_dir(PROGRAM).unwrap(),
        ] {
            let path = if cfg!(windows) && path.ends_with(CACHE) {
                path.parent().unwrap().to_path_buf()
            } else {
                path
            };
            assert!(path.ends_with(Path::new(VENDOR).join(PROGRAM)), "{path:?}");
        }
    }

    #[test]
    fn the_cache_is_never_the_config_or_data_directory() {
        let cache = locate::cache_dir(PROGRAM).unwrap();
        for other in [locate::config_dir(PROGRAM).unwrap(), locate::data_dir(PROGRAM).unwrap()] {
            assert_ne!(cache, other);
            assert!(!other.starts_with(&cache), "{other:?} sits inside the cache {cache:?}");
        }
    }

    #[test]
    fn unsafe_or_reserved_names_are_rejected() {
        for bad in [
            "", ".", "..", "../../etc", "a/b", "a\\b", "with\0nul", "SAM.", "SAM ", "apps", "Apps",
        ] {
            for locate in [locate::config_dir, locate::data_dir, locate::cache_dir] {
                let err = locate(bad).unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{bad:?}");
            }
        }
    }

    #[test]
    fn ordinary_names_are_accepted() {
        for good in [PROGRAM, "Colony", "SAM - Colony Edition"] {
            assert!(locate::config_dir(good).is_ok(), "{good:?}");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_follows_the_xdg_defaults() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        // Only assert the shape on a machine that does not override XDG_*.
        if std::env::var_os("XDG_CACHE_HOME").is_none() {
            assert_eq!(locate::cache_dir(PROGRAM).unwrap(), home.join(".cache/Colony/SAM"));
        }
        if std::env::var_os("XDG_CONFIG_HOME").is_none() {
            assert_eq!(locate::config_dir(PROGRAM).unwrap(), home.join(".config/Colony/SAM"));
        }
    }
}

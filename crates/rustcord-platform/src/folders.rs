//! Where Rustcord keeps its files on this system: the XDG base
//! directories on Linux, the user's known folders on Windows. Each is
//! Rustcord's own folder, not yet made; `None` when the system names none.

use std::{fs, path::Path, path::PathBuf};

/// The local store and what belongs with it: `$XDG_DATA_HOME/rustcord`, or
/// `%LOCALAPPDATA%\Rustcord` (this computer only: the store is sealed with keys
/// of this computer's keyring).
pub fn data() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        local().map(|local| app_folder(local, WINDOWS_NAME, LEGACY_WINDOWS_NAME, None))
    }
    #[cfg(not(windows))]
    {
        xdg("XDG_DATA_HOME", ".local/share", Some("app"))
    }
}

/// The small files read before the window opens: `$XDG_CONFIG_HOME/rustcord`,
/// or `%APPDATA%\Rustcord`.
pub fn config() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        crate::win::known_folder(crate::win::Known::RoamingAppData)
            .map(|roaming| app_folder(roaming, WINDOWS_NAME, LEGACY_WINDOWS_NAME, None))
    }
    #[cfg(not(windows))]
    {
        xdg("XDG_CONFIG_HOME", ".config", None)
    }
}

/// Pictures kept between runs: `$XDG_CACHE_HOME/rustcord`, or
/// `%LOCALAPPDATA%\Rustcord\cache`.
pub fn cache() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        data().map(|data| data.join("cache"))
    }
    #[cfg(not(windows))]
    {
        xdg("XDG_CACHE_HOME", ".cache", None)
    }
}

/// The log: `$XDG_STATE_HOME/rustcord`, or `%LOCALAPPDATA%\Rustcord\logs`.
pub fn state() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        data().map(|data| data.join("logs"))
    }
    #[cfg(not(windows))]
    {
        xdg("XDG_STATE_HOME", ".local/state", None)
    }
}

/// The person's home folder: `$HOME`, or the profile folder on Windows.
pub fn home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        crate::win::known_folder(crate::win::Known::Profile)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|home| home.is_absolute())
    }
}

/// The person's Downloads folder on Windows, wherever they moved it.
#[cfg(windows)]
pub fn downloads() -> Option<PathBuf> {
    crate::win::known_folder(crate::win::Known::Downloads)
}

#[cfg(windows)]
const WINDOWS_NAME: &str = "Rustcord";
#[cfg(windows)]
const LEGACY_WINDOWS_NAME: &str = "RUNE";

#[cfg(windows)]
fn local() -> Option<PathBuf> {
    crate::win::known_folder(crate::win::Known::LocalAppData)
}

/// `$variable/rustcord`, or `~/{fallback}/rustcord`: as the specification says, a
/// variable that is empty or not absolute is ignored. Existing RUNE folders
/// are moved over without replacing files Rustcord already has.
#[cfg(not(windows))]
fn xdg(variable: &str, fallback: &str, preserve_child: Option<&str>) -> Option<PathBuf> {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home().map(|home| home.join(fallback)))
        .map(|base| app_folder(base, "rustcord", "rune", preserve_child))
}

fn app_folder(
    base: PathBuf,
    name: &str,
    legacy_name: &str,
    preserve_child: Option<&str>,
) -> PathBuf {
    let current = base.join(name);
    let legacy = base.join(legacy_name);
    if !legacy.exists() {
        return current;
    }
    let preserved = preserve_child.filter(|child| legacy.join(child).exists());
    if !current.exists() && preserved.is_none() {
        return fs::rename(&legacy, &current).map_or(legacy, |_| current);
    }
    if !current.exists() && fs::create_dir_all(&current).is_err() {
        return legacy;
    }
    merge_folder(&legacy, &current, preserved);
    current
}

fn merge_folder(legacy: &Path, current: &Path, preserve_child: Option<&str>) {
    let Ok(entries) = fs::read_dir(legacy) else {
        return;
    };
    for entry in entries.flatten() {
        let source = entry.path();
        if preserve_child.is_some_and(|child| entry.file_name().to_str() == Some(child)) {
            continue;
        }
        let destination = current.join(entry.file_name());
        if !destination.exists() {
            let _ = fs::rename(source, destination);
            continue;
        }
        let source_is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
        let destination_is_dir =
            fs::symlink_metadata(&destination).is_ok_and(|metadata| metadata.file_type().is_dir());
        if source_is_dir && destination_is_dir {
            merge_folder(&source, &destination, None);
            let _ = fs::remove_dir(source);
        }
    }
    let _ = fs::remove_dir(legacy);
}

#[cfg(test)]
mod tests {
    // The system's folders: absolute, and Rustcord's own.
    #[test]
    fn each_folder_is_absolute_and_rustcord_s_own() {
        for folder in [
            super::data(),
            super::config(),
            super::cache(),
            super::state(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(folder.is_absolute(), "{folder:?}");
            assert!(
                folder
                    .components()
                    .any(|part| part.as_os_str().eq_ignore_ascii_case("rustcord")),
                "{folder:?}"
            );
        }
    }
}

//! XDG application configuration and recent project list.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const APP_DIR: &str = "embed-emu";
const FILE_NAME: &str = "gui.json";

/// Number of paths kept in the recent list.
pub const MAX_RECENT: usize = 10;

/// Persisted GUI configuration.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// Most recently opened or saved projects, newest first.
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
}

/// Returns the config path from XDG variables.
#[must_use]
pub fn path() -> Option<PathBuf> {
    path_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

fn path_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = config_base(xdg, home)?;
    Some(base.join(APP_DIR).join(FILE_NAME))
}

fn config_base(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    if let Some(xdg) = non_empty(xdg) {
        return Some(PathBuf::from(xdg));
    }
    non_empty(home).map(|home| PathBuf::from(home).join(".config"))
}

fn non_empty(value: Option<OsString>) -> Option<OsString> {
    value.filter(|value| !value.is_empty())
}

/// Loads config, returning defaults when the file is missing or invalid.
#[must_use]
pub fn load(path: &Path) -> AppConfig {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Saves config atomically.
pub fn save(path: &Path, config: &AppConfig) -> Result<(), String> {
    let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    crate::persist::write_atomic(path, &text)
}

/// Moves a path to the front of the recent list and enforces the limit.
pub fn push_recent(config: &mut AppConfig, path: &Path) {
    let path = path.to_path_buf();
    config.recent_files.retain(|item| item != &path);
    config.recent_files.insert(0, path);
    config.recent_files.truncate(MAX_RECENT);
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, MAX_RECENT, load, path_from, push_recent, save};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("embed-emu-config-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn xdg_path_wins_over_home() {
        let path = path_from(
            Some(OsString::from("/tmp/xdg")),
            Some(OsString::from("/tmp/home")),
        )
        .expect("path");
        assert_eq!(path, PathBuf::from("/tmp/xdg/embed-emu/gui.json"));
    }

    #[test]
    fn home_is_used_without_xdg() {
        let path = path_from(None, Some(OsString::from("/tmp/home"))).expect("path");
        assert_eq!(path, PathBuf::from("/tmp/home/.config/embed-emu/gui.json"));
    }

    #[test]
    fn empty_xdg_falls_back_to_home() {
        let path =
            path_from(Some(OsString::new()), Some(OsString::from("/tmp/home"))).expect("path");
        assert_eq!(path, PathBuf::from("/tmp/home/.config/embed-emu/gui.json"));
        assert!(path_from(None, None).is_none());
    }

    #[test]
    fn recent_list_deduplicates_and_limits() {
        let mut config = AppConfig::default();
        for index in 0..MAX_RECENT + 2 {
            push_recent(&mut config, Path::new(&format!("/tmp/{index}")));
        }
        assert_eq!(config.recent_files.len(), MAX_RECENT);
        assert_eq!(config.recent_files[0], PathBuf::from("/tmp/11"));
        push_recent(&mut config, Path::new("/tmp/5"));
        assert_eq!(config.recent_files[0], PathBuf::from("/tmp/5"));
        assert_eq!(config.recent_files.len(), MAX_RECENT);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = temp_dir("round-trip");
        let path = dir.join("gui.json");
        let mut config = AppConfig::default();
        push_recent(&mut config, Path::new("/tmp/one.emu.json"));
        save(&path, &config).expect("save");
        let loaded = load(&path);
        assert_eq!(loaded.recent_files, config.recent_files);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_or_invalid_config_uses_defaults() {
        let dir = temp_dir("defaults");
        assert!(load(&dir.join("missing.json")).recent_files.is_empty());
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("invalid.json");
        std::fs::write(&path, "not json").expect("write");
        assert!(load(&path).recent_files.is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}

//! Persistent application configuration.
//!
//! Rust owns the config; the frontend only reads and changes it through the
//! commands in [`crate::commands`]. Writes are atomic — a temporary file first,
//! then `rename` — so a crash mid-write cannot leave broken JSON behind.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::locale::Language;

pub const CONFIG_VERSION: u32 = 1;

/// A colour without an alpha channel: `SetSysColors` has no notion of opacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// The pair of system colours this app drives.
///
/// Formally these are `COLOR_HIGHLIGHT` and `COLOR_HOTLIGHT`, but on
/// Windows 10/11 there is only one visible effect: the selection rectangle you
/// drag with the mouse in Explorer and on the desktop. `highlight` paints its
/// outline, `hotlight` its translucent fill. Selected files themselves are
/// drawn by the Explorer theme and ignore these colours; classic Win32 apps
/// (Notepad, older list controls) still use `COLOR_HIGHLIGHT` for selected text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorPair {
    pub highlight: Rgb,
    pub hotlight: Rgb,
}

impl ColorPair {
    pub const fn new(highlight: Rgb, hotlight: Rgb) -> Self {
        Self {
            highlight,
            hotlight,
        }
    }
}

/// Palette used for monitors that do not have a profile yet.
/// The first entry is the Windows default.
const DEFAULT_PALETTE: [ColorPair; 6] = [
    ColorPair::new(Rgb::new(0, 120, 215), Rgb::new(0, 102, 204)),
    ColorPair::new(Rgb::new(16, 137, 62), Rgb::new(13, 110, 50)),
    ColorPair::new(Rgb::new(196, 43, 28), Rgb::new(160, 35, 23)),
    ColorPair::new(Rgb::new(136, 23, 152), Rgb::new(110, 19, 123)),
    ColorPair::new(Rgb::new(202, 80, 16), Rgb::new(165, 65, 13)),
    ColorPair::new(Rgb::new(0, 153, 153), Rgb::new(0, 122, 122)),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorProfile {
    /// When disabled, moving onto this monitor leaves the colours alone.
    pub enabled: bool,
    pub highlight: Rgb,
    pub hotlight: Rgb,
    /// User-supplied monitor name; `None` means show the system one.
    pub label: Option<String>,
}

impl MonitorProfile {
    /// Default profile for the monitor at `index` (zero based).
    pub fn for_index(index: usize) -> Self {
        let pair = DEFAULT_PALETTE[index % DEFAULT_PALETTE.len()];
        Self {
            enabled: true,
            highlight: pair.highlight,
            hotlight: pair.hotlight,
            label: None,
        }
    }

    pub fn colors(&self) -> ColorPair {
        ColorPair::new(self.highlight, self.hotlight)
    }
}

impl Default for MonitorProfile {
    fn default() -> Self {
        Self::for_index(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    /// `false` means "last time we changed the system colours and never put
    /// them back". On the next start that triggers a restore from
    /// [`AppConfig::saved_originals`]; without it a colour we set ourselves
    /// would be recorded as the "original" one forever.
    pub clean_shutdown: bool,
    /// The system colours as they were before we first touched them.
    pub saved_originals: Option<ColorPair>,
    /// `true` applies the colour lazily, on the first left click after the
    /// cursor moves to another monitor. `false` applies it as the cursor
    /// crosses the boundary.
    pub apply_on_click: bool,
    /// Start minimized to the tray.
    pub start_hidden: bool,
    /// Interface language; defaults to the Windows setting.
    pub language: Language,
    /// Keyed by the stable monitor id from [`crate::monitors`].
    pub monitors: HashMap<String, MonitorProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            clean_shutdown: true,
            saved_originals: None,
            apply_on_click: true,
            start_hidden: false,
            language: Language::default(),
            monitors: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Reads the config, falling back to defaults if the file is missing or
    /// broken.
    ///
    /// A corrupt config is no reason to refuse to start: the user loses their
    /// settings, but the app keeps working.
    pub fn load(path: &Path) -> Self {
        let raw = match fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!("no config found, using defaults");
                return Self::default();
            }
            Err(err) => {
                tracing::error!(?err, "could not read the config");
                return Self::default();
            }
        };

        match serde_json::from_str::<Self>(&raw) {
            Ok(config) => config.migrated(),
            Err(err) => {
                tracing::error!(?err, "config is corrupt, using defaults");
                Self::default()
            }
        }
    }

    /// Hook for future format versions.
    fn migrated(mut self) -> Self {
        if self.version != CONFIG_VERSION {
            tracing::info!(from = self.version, to = CONFIG_VERSION, "migrating config");
            self.version = CONFIG_VERSION;
        }
        self
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let tmp = tmp_path(path);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&tmp, json)?;
        // Windows will not rename over an existing file, so the old one has to
        // go first. The window where no config exists is microseconds long, and
        // the data is already safe in the .tmp file by then.
        let _ = fs::remove_file(path);
        fs::rename(&tmp, path)
    }

    /// The monitor's profile, if it exists and is enabled.
    pub fn active_profile(&self, monitor_id: &str) -> Option<&MonitorProfile> {
        self.monitors.get(monitor_id).filter(|p| p.enabled)
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_current_version() {
        assert_eq!(AppConfig::default().version, CONFIG_VERSION);
    }

    #[test]
    fn roundtrips_through_json() {
        let mut config = AppConfig {
            saved_originals: Some(ColorPair::new(Rgb::new(1, 2, 3), Rgb::new(4, 5, 6))),
            ..Default::default()
        };
        config
            .monitors
            .insert("abc".into(), MonitorProfile::for_index(2));

        let json = serde_json::to_string(&config).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(back.saved_originals, config.saved_originals);
        assert_eq!(
            back.monitors["abc"].highlight,
            config.monitors["abc"].highlight
        );
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let config: AppConfig = serde_json::from_str("{}").unwrap();
        assert!(config.apply_on_click);
        assert!(config.clean_shutdown);
        assert!(config.monitors.is_empty());
    }

    #[test]
    fn unknown_version_is_migrated() {
        let config: AppConfig = serde_json::from_str(r#"{"version": 0}"#).unwrap();
        assert_eq!(config.migrated().version, CONFIG_VERSION);
    }

    #[test]
    fn disabled_profile_is_not_active() {
        let mut config = AppConfig::default();
        let mut profile = MonitorProfile::for_index(0);
        profile.enabled = false;
        config.monitors.insert("m1".into(), profile);

        assert!(config.active_profile("m1").is_none());
        assert!(config.active_profile("nope").is_none());
    }

    #[test]
    fn palette_wraps_around() {
        let first = MonitorProfile::for_index(0).highlight;
        let wrapped = MonitorProfile::for_index(DEFAULT_PALETTE.len()).highlight;
        assert_eq!(first, wrapped);
    }
}

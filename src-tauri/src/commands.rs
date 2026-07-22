//! IPC commands for the frontend.
//!
//! The frontend keeps no settings of its own: it reads them from here and
//! writes them back the same way.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::config::{AppConfig, ColorPair, MonitorProfile};
use crate::locale::{Language, Locale};
use crate::monitors::MonitorInfo;
use crate::state::AppState;
use crate::tray;

type Shared<'a> = State<'a, Arc<AppState>>;

/// The settings the frontend controls, without the config's internal fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub apply_on_click: bool,
    pub start_hidden: bool,
    pub language: Language,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub monitors: Vec<MonitorInfo>,
    pub profiles: std::collections::HashMap<String, MonitorProfile>,
    pub preferences: Preferences,
    /// The language to render the UI in: `system` is already resolved here.
    pub resolved_language: Locale,
    pub originals: ColorPair,
    pub applied: Option<ColorPair>,
}

fn preferences_of(config: &AppConfig) -> Preferences {
    Preferences {
        apply_on_click: config.apply_on_click,
        start_hidden: config.start_hidden,
        language: config.language,
    }
}

/// Everything the UI needs at startup, in a single call.
#[tauri::command]
pub fn get_overview(state: Shared<'_>) -> Overview {
    let monitors = state.monitors().monitors.clone();
    let config = state.config();

    Overview {
        monitors,
        profiles: config.monitors.clone(),
        preferences: preferences_of(&config),
        resolved_language: config.language.resolve(),
        originals: state.colors.originals(),
        applied: state.colors.applied(),
    }
}

/// Rebuilds the monitor list (the "refresh" button in the UI).
#[tauri::command]
pub fn refresh_monitors(state: Shared<'_>) -> Vec<MonitorInfo> {
    state.refresh_monitors().monitors
}

#[tauri::command]
pub fn set_monitor_profile(
    state: Shared<'_>,
    monitor_id: String,
    profile: MonitorProfile,
) -> Result<(), String> {
    if state.monitors().by_id(&monitor_id).is_none() {
        return Err(format!("monitor {monitor_id} not found"));
    }

    state.update_config(|config| {
        config.monitors.insert(monitor_id, profile);
    });
    Ok(())
}

/// Resets a monitor's profile to the default colours for its position.
#[tauri::command]
pub fn reset_monitor_profile(
    state: Shared<'_>,
    monitor_id: String,
) -> Result<MonitorProfile, String> {
    let index = state
        .monitors()
        .by_id(&monitor_id)
        .map(|m| m.index.saturating_sub(1) as usize)
        .ok_or_else(|| format!("monitor {monitor_id} not found"))?;

    let profile = MonitorProfile::for_index(index);
    state.update_config(|config| {
        config.monitors.insert(monitor_id, profile.clone());
    });
    Ok(profile)
}

/// Saves the settings and returns the language the UI should render in.
///
/// The resolved language is returned on purpose: with `system` the frontend
/// would have no way of knowing what it turned into.
#[tauri::command]
pub fn set_preferences(
    app: tauri::AppHandle,
    state: Shared<'_>,
    preferences: Preferences,
) -> Locale {
    let locale = preferences.language.resolve();

    state.update_config(|config| {
        config.apply_on_click = preferences.apply_on_click;
        config.start_hidden = preferences.start_hidden;
        config.language = preferences.language;
    });

    // Rust draws the tray menu, so without this it would stay in the old
    // language.
    tray::relocalize(&app, locale);
    locale
}

/// Sets a colour pair right away — for the "apply" button and live preview.
#[tauri::command]
pub fn apply_colors(state: Shared<'_>, colors: ColorPair) -> bool {
    state.colors.apply(colors)
}

/// Puts the original system colours back.
#[tauri::command]
pub fn restore_colors(state: Shared<'_>) -> bool {
    state.colors.restore()
}

#[tauri::command]
pub fn get_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    if enabled {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|e| e.to_string())
}

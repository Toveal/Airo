//! The tray icon and its menu.

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Runtime, WebviewWindow};

use crate::locale::Locale;
use crate::state::AppState;

pub const MAIN_WINDOW: &str = "main";

const ID_SHOW: &str = "show";
const ID_QUIT: &str = "quit";

/// Kept around so the menu can be rebuilt when the language changes.
struct TrayHandle(TrayIcon);

fn menu<R: Runtime, M: Manager<R>>(manager: &M, locale: Locale) -> tauri::Result<Menu<R>> {
    let show = MenuItem::with_id(manager, ID_SHOW, locale.tray_show(), true, None::<&str>)?;
    let quit = MenuItem::with_id(manager, ID_QUIT, locale.tray_quit(), true, None::<&str>)?;
    Menu::with_items(manager, &[&show, &quit])
}

pub fn setup(app: &App) -> tauri::Result<()> {
    let locale = app.state::<Arc<AppState>>().config().language.resolve();

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().ok_or_else(|| {
            tauri::Error::AssetNotFound("the default window icon is missing".into())
        })?)
        .tooltip("Airo")
        .menu(&menu(app, locale)?)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_SHOW => show_window(app),
            ID_QUIT => crate::request_exit(app),
            _ => {}
        })
        .build(app)?;

    app.manage(TrayHandle(tray));
    Ok(())
}

/// Redraws the tray menu in another language.
pub fn relocalize(app: &AppHandle, locale: Locale) {
    let Some(handle) = app.try_state::<TrayHandle>() else {
        return;
    };

    match menu(app, locale).and_then(|menu| handle.0.set_menu(Some(menu))) {
        Ok(()) => tracing::debug!(?locale, "tray menu redrawn"),
        Err(err) => tracing::error!(?err, "could not update the tray menu"),
    }
}

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW)
}

pub fn show_window(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };

    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);

    if visible && !minimized {
        let _ = window.hide();
    } else {
        show_window(app);
    }
}

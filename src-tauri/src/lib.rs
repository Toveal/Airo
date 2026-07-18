//! Airo — changes the Windows selection colours depending on which monitor the
//! cursor is on.

mod config;
mod locale;
mod logging;
mod monitors;
mod session;
mod state;
mod sys_colors;
mod tray;
mod watcher;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, RunEvent, WindowEvent};

use crate::state::AppState;
use crate::watcher::Watcher;

const CONFIG_FILE: &str = "config.json";

/// Tells "the user clicked the close button" apart from "the app is exiting".
///
/// Without this flag `prevent_close` stops anyone from closing the window,
/// ourselves included: the event loop then ends with the WebView2 window still
/// alive and Chromium complains with
/// `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`.
static EXITING: AtomicBool = AtomicBool::new(false);

/// Holds the watcher in Tauri state so it can be stopped on exit.
struct WatcherSlot(Mutex<Option<Watcher>>);

/// Shuts the app down properly: close the windows first, then exit.
///
/// `AppHandle::exit` does not close windows on its own — it only flips the
/// event loop to `ControlFlow::Exit`, which leaves WebView2 with an
/// undestroyed window.
pub(crate) fn request_exit(app: &AppHandle) {
    EXITING.store(true, Ordering::SeqCst);

    let windows = app.webview_windows();
    if windows.is_empty() {
        app.exit(0);
        return;
    }

    for (label, window) in windows {
        if let Err(err) = window.close() {
            tracing::error!(?err, %label, "could not close the window, exiting the hard way");
            app.exit(0);
            return;
        }
    }
    // Once the last window is gone the runtime raises ExitRequested itself,
    // followed by RunEvent::Exit — that is where shutdown() runs.
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(setup)
        .on_window_event(|window, event| {
            // The close button hides the window into the tray instead of
            // quitting: monitor tracking has to keep running. Quitting goes
            // through the tray, and then the close must be allowed — otherwise
            // the window could never be destroyed at all.
            if let WindowEvent::CloseRequested { api, .. } = event
                && !EXITING.load(Ordering::SeqCst)
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("could not build the application");

    app.run(|app, event| {
        if matches!(event, RunEvent::Exit) {
            shutdown(app);
        }
    });
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = app.path().app_log_dir()?;
    logging::init(&log_dir);

    let config_path = app.path().app_config_dir()?.join(CONFIG_FILE);
    tracing::info!(path = ?config_path, "config");

    let state = AppState::init(config_path);
    let start_hidden = state.config().start_hidden;

    let watcher = Watcher::start(app.handle().clone(), Arc::clone(&state));

    app.manage(state);
    app.manage(WatcherSlot(Mutex::new(Some(watcher))));

    tray::setup(app)?;

    if start_hidden && let Some(window) = app.get_webview_window(tray::MAIN_WINDOW) {
        let _ = window.hide();
    }

    Ok(())
}

/// Stops the watcher and gives the user their colours back.
///
/// This is the only place where the rollback is guaranteed: `Drop` cannot be
/// relied on, because exiting goes through `app.exit`, which does not unwind
/// the worker threads' stacks.
fn shutdown(app: &AppHandle) {
    if let Some(slot) = app.try_state::<WatcherSlot>()
        && let Ok(mut guard) = slot.0.lock()
        && let Some(mut watcher) = guard.take()
    {
        watcher.stop();
    }

    if let Some(state) = app.try_state::<Arc<AppState>>() {
        state.shutdown();
    }

    tracing::info!("application stopped");
}

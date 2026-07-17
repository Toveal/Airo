//! The watcher thread: tracks the cursor, clicks and session events.
//!
//! A global `WH_MOUSE_LL` hook would be the obvious tool here, but it sits in
//! the input path of the entire OS, needs its own message loop and regularly
//! upsets antivirus software — far too much for "which monitor is the cursor
//! on". Polling `GetCursorPos` every 16 ms gives the same answer and intercepts
//! nothing.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{HMONITOR, MONITOR_DEFAULTTONEAREST, MonitorFromPoint};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow};

use crate::session::{SessionEvent, SessionWatcher};
use crate::state::AppState;

/// Polling interval. 16 ms is roughly a frame at 60 Hz: a monitor change and a
/// left click are both caught before the system repaints the selection.
const TICK: Duration = Duration::from_millis(16);

/// Check for the secure desktop (UAC, lock screen) every this many ticks. The
/// check is the most expensive one here and the event is rare.
const SECURE_DESKTOP_EVERY: u32 = 12;

pub const EVENT_MONITOR_CHANGED: &str = "monitor-changed";
pub const EVENT_MONITORS_CHANGED: &str = "monitors-changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorChangedPayload {
    pub id: String,
    pub index: u32,
    pub label: String,
}

pub struct Watcher {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Watcher {
    pub fn start(app: AppHandle, state: Arc<AppState>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let (sender, receiver) = mpsc::channel();

        let handle = {
            let running = Arc::clone(&running);
            thread::Builder::new()
                .name("airo-watcher".into())
                .spawn(move || {
                    // The SessionWatcher is stopped by its Drop when this
                    // closure ends, so this thread has to own it.
                    let _session = SessionWatcher::start(sender);
                    poll_loop(&app, &state, &running, &receiver);
                })
                .expect("could not spawn the watcher thread")
        };

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            tracing::error!("the watcher thread panicked");
        }
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Default)]
struct LoopState {
    last_monitor: Option<isize>,
    /// The monitor whose colours are waiting for the first click.
    pending: Option<String>,
    lmb_down: bool,
    secure_desktop: bool,
    tick: u32,
}

fn poll_loop(
    app: &AppHandle,
    state: &Arc<AppState>,
    running: &AtomicBool,
    session: &Receiver<SessionEvent>,
) {
    tracing::info!("watcher started");
    let mut loop_state = LoopState::default();

    while running.load(Ordering::Relaxed) {
        thread::sleep(TICK);

        while let Ok(event) = session.try_recv() {
            handle_session_event(app, state, &mut loop_state, event);
        }

        if let Some(handle) = cursor_monitor() {
            handle_cursor(app, state, &mut loop_state, handle);
        }

        handle_mouse_button(state, &mut loop_state);

        loop_state.tick = loop_state.tick.wrapping_add(1);
        if loop_state.tick % SECURE_DESKTOP_EVERY == 0 {
            handle_secure_desktop(state, &mut loop_state);
        }
    }

    tracing::info!("watcher stopped");
}

// --- Handlers ----------------------------------------------------------------

fn handle_cursor(
    app: &AppHandle,
    state: &Arc<AppState>,
    loop_state: &mut LoopState,
    handle: HMONITOR,
) {
    let raw = handle.0 as isize;
    if loop_state.last_monitor == Some(raw) {
        return;
    }
    loop_state.last_monitor = Some(raw);

    let Some((id, payload)) = ({
        let monitors = state.monitors();
        monitors.by_handle(handle).map(|m| {
            (
                m.id.clone(),
                MonitorChangedPayload {
                    id: m.id.clone(),
                    index: m.index,
                    label: m.label.clone(),
                },
            )
        })
    }) else {
        // The cursor is on a monitor missing from the snapshot, so the
        // configuration changed without a WM_DISPLAYCHANGE. The list will be
        // rebuilt on the next event.
        tracing::warn!(handle = raw, "monitor not found in the snapshot");
        return;
    };

    tracing::debug!(%id, label = %payload.label, "cursor moved to another monitor");
    emit(app, EVENT_MONITOR_CHANGED, payload);

    if state.config().apply_on_click {
        loop_state.pending = Some(id);
    } else {
        apply_for_monitor(state, &id);
    }
}

fn handle_mouse_button(state: &Arc<AppState>, loop_state: &mut LoopState) {
    // SAFETY: GetAsyncKeyState reads global keyboard/mouse state and has no
    // side effects; the high bit means "the button is down right now".
    let down = unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON.0)) } as u16 & 0x8000 != 0;

    let pressed = down && !loop_state.lmb_down;
    loop_state.lmb_down = down;

    if !pressed {
        return;
    }

    if let Some(id) = loop_state.pending.take() {
        apply_for_monitor(state, &id);
    }
}

fn handle_secure_desktop(state: &Arc<AppState>, loop_state: &mut LoopState) {
    // On the secure desktop (UAC, Ctrl+Alt+Del) our session has no foreground
    // window.
    // SAFETY: GetForegroundWindow has no side effects.
    let has_window = !unsafe { GetForegroundWindow() }.is_invalid();

    if !has_window && !loop_state.secure_desktop {
        loop_state.secure_desktop = true;
        tracing::debug!("entered the secure desktop");
    } else if has_window && loop_state.secure_desktop {
        loop_state.secure_desktop = false;
        tracing::debug!("left the secure desktop");
        // winlogon may have reset the system colours to its own, so set ours
        // again.
        state.colors.reapply();
    }
}

fn handle_session_event(
    app: &AppHandle,
    state: &Arc<AppState>,
    loop_state: &mut LoopState,
    event: SessionEvent,
) {
    tracing::debug!(?event, "session event");

    match event {
        SessionEvent::DisplayChanged => {
            state.refresh_monitors();
            // The snapshot changed, so the previous HMONITOR means nothing any
            // more; force a fresh detection on the next tick.
            loop_state.last_monitor = None;
            loop_state.pending = None;
            emit(app, EVENT_MONITORS_CHANGED, ());
        }
        SessionEvent::Unlocked | SessionEvent::RemoteConnect | SessionEvent::ConsoleConnect => {
            // winlogon may have reset the colours; put ours back and re-detect
            // the monitor under the cursor.
            state.colors.reapply();
            loop_state.last_monitor = None;
        }
        // Locking and signing out are not our business: the lock screen lives
        // on the secure desktop and does not use the session's system colours.
        SessionEvent::Locked
        | SessionEvent::Logoff
        | SessionEvent::RemoteDisconnect
        | SessionEvent::Logon
        | SessionEvent::ConsoleDisconnect => {}
    }
}

// --- Helpers -----------------------------------------------------------------

/// The monitor under the cursor. `None` if the position could not be read,
/// for example while the secure desktop is up.
fn cursor_monitor() -> Option<HMONITOR> {
    let mut point = POINT::default();

    // SAFETY: the pointer refers to a local that outlives the call.
    unsafe { GetCursorPos(&mut point) }.ok()?;

    // SAFETY: MONITOR_DEFAULTTONEAREST guarantees a non-null result for any
    // point, including coordinates outside every monitor.
    Some(unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) })
}

/// Applies the monitor's profile; a disabled monitor gets the originals back.
fn apply_for_monitor(state: &Arc<AppState>, monitor_id: &str) {
    let colors = {
        let config = state.config();
        config
            .active_profile(monitor_id)
            .map(crate::config::MonitorProfile::colors)
    };

    match colors {
        Some(colors) => state.colors.apply(colors),
        None => state.colors.restore(),
    };
}

fn emit<P: Serialize + Clone>(app: &AppHandle, event: &str, payload: P) {
    if let Err(err) = app.emit(event, payload) {
        tracing::warn!(?err, event, "could not send the event to the frontend");
    }
}

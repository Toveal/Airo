//! A hidden window for system notifications.
//!
//! It exists for exactly two things: session events (`WM_WTSSESSION_CHANGE`)
//! and display configuration changes (`WM_DISPLAYCHANGE`). The window is a
//! plain one on purpose, not message-only: message-only windows never receive
//! broadcasts such as `WM_DISPLAYCHANGE`.

use std::cell::RefCell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::RemoteDesktop::{
    NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification, WTSUnRegisterSessionNotification,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MSG, PostMessageW,
    PostQuitMessage, RegisterClassW, TranslateMessage, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE,
    WM_WTSSESSION_CHANGE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::w;

const WTS_CONSOLE_CONNECT: u32 = 0x1;
const WTS_CONSOLE_DISCONNECT: u32 = 0x2;
const WTS_REMOTE_CONNECT: u32 = 0x3;
const WTS_REMOTE_DISCONNECT: u32 = 0x4;
const WTS_SESSION_LOGON: u32 = 0x5;
const WTS_SESSION_LOGOFF: u32 = 0x6;
const WTS_SESSION_LOCK: u32 = 0x7;
const WTS_SESSION_UNLOCK: u32 = 0x8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    Locked,
    Unlocked,
    Logon,
    Logoff,
    ConsoleConnect,
    ConsoleDisconnect,
    RemoteConnect,
    RemoteDisconnect,
    DisplayChanged,
}

thread_local! {
    /// The sender lives on the same thread as the window, so no global
    /// `static` is needed — a window procedure always runs on its own thread.
    static SENDER: RefCell<Option<Sender<SessionEvent>>> = const { RefCell::new(None) };
}

static WINDOW_CLASS: OnceLock<Option<u16>> = OnceLock::new();

/// Owns the window thread and shuts it down properly on drop.
pub struct SessionWatcher {
    hwnd: &'static AtomicIsize,
    handle: Option<JoinHandle<()>>,
}

static HWND_SLOT: AtomicIsize = AtomicIsize::new(0);

impl SessionWatcher {
    pub fn start(sender: Sender<SessionEvent>) -> Self {
        let handle = thread::Builder::new()
            .name("airo-session".into())
            .spawn(move || run(sender, &HWND_SLOT))
            .expect("could not spawn the session watcher thread");

        Self {
            hwnd: &HWND_SLOT,
            handle: Some(handle),
        }
    }
}

impl Drop for SessionWatcher {
    fn drop(&mut self) {
        let hwnd = self.hwnd.swap(0, Ordering::SeqCst);
        if hwnd != 0 {
            // SAFETY: the handle was published by the window thread after a
            // successful CreateWindowExW; PostMessageW is thread safe and the
            // window is only ever destroyed by its own thread in response.
            let _ =
                unsafe { PostMessageW(Some(HWND(hwnd as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        }

        if let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            tracing::error!("the session watcher thread panicked");
        }
    }
}

fn run(sender: Sender<SessionEvent>, slot: &AtomicIsize) {
    SENDER.with(|s| *s.borrow_mut() = Some(sender));

    if WINDOW_CLASS.get_or_init(register_class).is_none() {
        tracing::error!("window class not registered, session events unavailable");
        return;
    }

    // SAFETY: the class is registered above; zero size and no WS_VISIBLE mean
    // the window is never shown to the user.
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            w!("AiroSessionMonitor"),
            w!("Airo Session Monitor"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        )
    };

    let Ok(hwnd) = hwnd else {
        tracing::error!("could not create the window for session events");
        return;
    };

    slot.store(hwnd.0 as isize, Ordering::SeqCst);

    // SAFETY: the window was just created by this thread and lives until the
    // end of this function.
    if let Err(err) = unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) } {
        tracing::error!(?err, "could not subscribe to session events");
    }

    tracing::info!("session watcher started");

    let mut msg = MSG::default();
    // SAFETY: a message loop on the thread that owns the window; GetMessageW
    // returns 0 on WM_QUIT, which WM_DESTROY posts.
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // SAFETY: the window still exists until its procedure returns; this
    // unregister pairs with the WTSRegisterSessionNotification above.
    unsafe {
        let _ = WTSUnRegisterSessionNotification(hwnd);
    }

    slot.store(0, Ordering::SeqCst);
    tracing::info!("session watcher stopped");
}

fn register_class() -> Option<u16> {
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: w!("AiroSessionMonitor"),
        ..Default::default()
    };

    // SAFETY: every field is valid and the class name is a static NUL
    // terminated literal.
    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        tracing::error!("RegisterClassW failed");
        return None;
    }
    Some(atom)
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_WTSSESSION_CHANGE => {
            let event = match wparam.0 as u32 {
                WTS_SESSION_LOCK => Some(SessionEvent::Locked),
                WTS_SESSION_UNLOCK => Some(SessionEvent::Unlocked),
                WTS_SESSION_LOGON => Some(SessionEvent::Logon),
                WTS_SESSION_LOGOFF => Some(SessionEvent::Logoff),
                WTS_CONSOLE_CONNECT => Some(SessionEvent::ConsoleConnect),
                WTS_CONSOLE_DISCONNECT => Some(SessionEvent::ConsoleDisconnect),
                WTS_REMOTE_CONNECT => Some(SessionEvent::RemoteConnect),
                WTS_REMOTE_DISCONNECT => Some(SessionEvent::RemoteDisconnect),
                _ => None,
            };
            if let Some(event) = event {
                emit(event);
            }
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            emit(SessionEvent::DisplayChanged);
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: runs on the window's thread and ends its message loop.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: default handling for everything else; hwnd is valid.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn emit(event: SessionEvent) {
    SENDER.with(|sender| {
        if let Some(sender) = sender.borrow().as_ref() {
            // An error means the receiver is already gone, which is the normal
            // path during shutdown.
            let _ = sender.send(event);
        }
    });
}

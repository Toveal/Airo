//! Driving the `COLOR_HIGHLIGHT` / `COLOR_HOTLIGHT` system colours.
//!
//! `SetSysColors` affects the whole user session and broadcasts
//! `WM_SYSCOLORCHANGE` to every top-level window, so it must be called rarely
//! and always rolled back on exit.

use std::sync::Mutex;

use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{COLOR_HIGHLIGHT, COLOR_HOTLIGHT, GetSysColor, SetSysColors};

use crate::config::{ColorPair, Rgb};

const ELEMENTS: [i32; 2] = [COLOR_HIGHLIGHT.0, COLOR_HOTLIGHT.0];

pub const fn rgb_to_colorref(rgb: Rgb) -> COLORREF {
    COLORREF((rgb.r as u32) | ((rgb.g as u32) << 8) | ((rgb.b as u32) << 16))
}

pub const fn colorref_to_rgb(color: COLORREF) -> Rgb {
    Rgb::new(
        (color.0 & 0xFF) as u8,
        ((color.0 >> 8) & 0xFF) as u8,
        ((color.0 >> 16) & 0xFF) as u8,
    )
}

/// Reads the colours currently set in the system.
pub fn current_system_colors() -> ColorPair {
    // SAFETY: GetSysColor takes a system element index and has no side effects;
    // both indices are constants from windows-rs.
    unsafe {
        ColorPair::new(
            colorref_to_rgb(COLORREF(GetSysColor(COLOR_HIGHLIGHT))),
            colorref_to_rgb(COLORREF(GetSysColor(COLOR_HOTLIGHT))),
        )
    }
}

fn set_system_colors(pair: ColorPair) -> bool {
    let colors = [
        rgb_to_colorref(pair.highlight),
        rgb_to_colorref(pair.hotlight),
    ];

    // SAFETY: both arrays outlive the call and have the same length of 2, which
    // is what the first argument reports.
    let result = unsafe { SetSysColors(ELEMENTS.len() as i32, ELEMENTS.as_ptr(), colors.as_ptr()) };

    if let Err(err) = &result {
        tracing::error!(?err, "SetSysColors failed");
    }
    result.is_ok()
}

#[derive(Debug)]
struct Inner {
    originals: ColorPair,
    applied: Option<ColorPair>,
}

/// Remembers the user's original colours and knows how to roll back to them.
#[derive(Debug)]
pub struct SystemColorManager {
    inner: Mutex<Inner>,
}

impl SystemColorManager {
    /// Captures the current colours as the originals.
    ///
    /// `recovery` holds the colours saved by a previous run that ended badly.
    /// When present they are restored first: otherwise a colour we set
    /// ourselves would become the "original" one and the user could never get
    /// their own scheme back.
    pub fn capture(recovery: Option<ColorPair>) -> Self {
        if let Some(pair) = recovery {
            tracing::warn!(?pair, "previous run ended badly, restoring colours");
            set_system_colors(pair);
        }

        let originals = current_system_colors();
        tracing::info!(?originals, "original system colours saved");

        Self {
            inner: Mutex::new(Inner {
                originals,
                applied: None,
            }),
        }
    }

    pub fn originals(&self) -> ColorPair {
        self.lock().originals
    }

    /// The colours we have set right now, if they differ from the originals.
    pub fn applied(&self) -> Option<ColorPair> {
        self.lock().applied
    }

    /// Sets a colour pair. Calling it again with the same value is a no-op, to
    /// avoid broadcasting `WM_SYSCOLORCHANGE` for nothing.
    pub fn apply(&self, pair: ColorPair) -> bool {
        let mut inner = self.lock();
        if inner.applied == Some(pair) {
            return true;
        }

        if !set_system_colors(pair) {
            return false;
        }

        inner.applied = Some(pair);
        tracing::debug!(?pair, "system colours applied");
        true
    }

    /// Puts the original colours back. Idempotent.
    pub fn restore(&self) -> bool {
        let mut inner = self.lock();
        if inner.applied.is_none() {
            return true;
        }

        let originals = inner.originals;
        if !set_system_colors(originals) {
            return false;
        }

        inner.applied = None;
        tracing::info!(?originals, "original system colours restored");
        true
    }

    /// Re-applies whatever is already set.
    ///
    /// Needed after leaving the secure desktop and after unlocking the session:
    /// winlogon may reset the system colours to its own.
    pub fn reapply(&self) -> bool {
        let applied = self.lock().applied;
        match applied {
            Some(pair) => set_system_colors(pair),
            None => true,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // Nothing inside the critical section can panic except the Win32 calls
        // themselves, and the state stays consistent even after those.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorref_is_little_endian_bgr() {
        // A Win32 COLORREF is 0x00BBGGRR.
        assert_eq!(rgb_to_colorref(Rgb::new(0x12, 0x34, 0x56)).0, 0x0056_3412);
    }

    #[test]
    fn colorref_roundtrips() {
        for rgb in [
            Rgb::new(0, 0, 0),
            Rgb::new(255, 255, 255),
            Rgb::new(0, 120, 215),
            Rgb::new(1, 2, 3),
        ] {
            assert_eq!(colorref_to_rgb(rgb_to_colorref(rgb)), rgb);
        }
    }

    #[test]
    fn colorref_ignores_high_byte() {
        assert_eq!(
            colorref_to_rgb(COLORREF(0xFF00_0000)),
            Rgb::new(0, 0, 0),
            "the top COLORREF byte is reserved and must not leak into the RGB value"
        );
    }
}

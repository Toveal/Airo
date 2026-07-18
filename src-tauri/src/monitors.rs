//! Monitor enumeration with stable identifiers.
//!
//! An `HMONITOR` and a `\\.\DISPLAY1` name only live until the display
//! configuration changes next, so user settings cannot be keyed on them. The
//! stable key comes from `monitorDevicePath` (`QueryDisplayConfig`) instead: it
//! describes the physical panel and the port it hangs off, and survives both a
//! reboot and a cable swap.

use std::collections::HashMap;

use serde::Serialize;
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
    GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
};
use windows::Win32::Foundation::{ERROR_SUCCESS, LPARAM, RECT, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::core::BOOL;

/// `MONITORINFOF_PRIMARY` — windows-rs does not export it.
const MONITORINFOF_PRIMARY: u32 = 0x0000_0001;

/// A monitor as the frontend and the watcher see it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// Stable settings key.
    pub id: String,
    /// Ordinal used to sort the tabs in the UI, starting at 1.
    pub index: u32,
    /// System name, `\\.\DISPLAY1`.
    pub device_name: String,
    /// Human readable panel name, for example `DELL U2720Q`.
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_primary: bool,
    /// Diagnostics only: the device path [`Self::id`] was derived from.
    pub device_path: Option<String>,
    #[serde(skip)]
    pub handle: isize,
}

/// A consistent snapshot of the display configuration.
#[derive(Debug, Clone, Default)]
pub struct MonitorSnapshot {
    pub monitors: Vec<MonitorInfo>,
}

impl MonitorSnapshot {
    pub fn by_handle(&self, handle: HMONITOR) -> Option<&MonitorInfo> {
        let handle = handle.0 as isize;
        self.monitors.iter().find(|m| m.handle == handle)
    }

    pub fn by_id(&self, id: &str) -> Option<&MonitorInfo> {
        self.monitors.iter().find(|m| m.id == id)
    }
}

/// Collects the current display configuration.
pub fn enumerate() -> MonitorSnapshot {
    let descriptors = display_descriptors();
    let mut raw = enum_display_monitors();

    // Primary first, then left to right and top to bottom.
    raw.sort_by(|a, b| {
        b.is_primary
            .cmp(&a.is_primary)
            .then_with(|| a.rect.left.cmp(&b.rect.left))
            .then_with(|| a.rect.top.cmp(&b.rect.top))
    });

    let monitors = raw
        .into_iter()
        .enumerate()
        .map(|(index, m)| {
            let descriptor = descriptors.get(&m.device_name);
            let device_path = descriptor.and_then(|d| d.device_path.clone());
            let id = device_path
                .as_deref()
                .map_or_else(|| fnv1a_hex(&m.device_name), fnv1a_hex);
            let label = descriptor
                .and_then(|d| d.friendly_name.clone())
                .unwrap_or_else(|| m.device_name.clone());

            MonitorInfo {
                id,
                index: index as u32 + 1,
                label,
                device_path,
                x: m.rect.left,
                y: m.rect.top,
                width: m.rect.right - m.rect.left,
                height: m.rect.bottom - m.rect.top,
                is_primary: m.is_primary,
                handle: m.handle,
                device_name: m.device_name,
            }
        })
        .collect();

    MonitorSnapshot {
        monitors: disambiguate_labels(monitors),
    }
}

/// Appends a number to duplicate names.
///
/// Two panels of the same model report the same
/// `monitorFriendlyDeviceName`, which makes them indistinguishable in the list
/// — yet the user has to pick a specific one.
fn disambiguate_labels(mut monitors: Vec<MonitorInfo>) -> Vec<MonitorInfo> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for monitor in &monitors {
        *counts.entry(monitor.label.as_str()).or_default() += 1;
    }

    let duplicates: Vec<String> = counts
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|(label, _)| label.to_owned())
        .collect();

    for monitor in &mut monitors {
        if duplicates.contains(&monitor.label) {
            monitor.label = format!("{} #{}", monitor.label, monitor.index);
        }
    }

    monitors
}

// --- EnumDisplayMonitors -----------------------------------------------------

struct RawMonitor {
    handle: isize,
    device_name: String,
    rect: RECT,
    is_primary: bool,
}

extern "system" fn enum_proc(handle: HMONITOR, _hdc: HDC, _rect: *mut RECT, data: LPARAM) -> BOOL {
    // SAFETY: the pointer refers to the Vec<RawMonitor> created in
    // enum_display_monitors; EnumDisplayMonitors invokes the callback
    // synchronously while that Vec is alive, and only from the calling thread.
    let out = unsafe { &mut *(data.0 as *mut Vec<RawMonitor>) };

    let mut info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        szDevice: [0; 32],
    };

    // SAFETY: cbSize describes a MONITORINFOEXW and that is exactly the struct
    // we hand over — GetMonitorInfoW picks the variant based on cbSize.
    let ok = unsafe {
        GetMonitorInfoW(handle, std::ptr::from_mut(&mut info).cast::<MONITORINFO>()).as_bool()
    };

    if ok {
        out.push(RawMonitor {
            handle: handle.0 as isize,
            device_name: wide_to_string(&info.szDevice),
            rect: info.monitorInfo.rcMonitor,
            is_primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    } else {
        tracing::warn!("GetMonitorInfoW failed for one of the monitors");
    }

    BOOL::from(true)
}

fn enum_display_monitors() -> Vec<RawMonitor> {
    let mut out: Vec<RawMonitor> = Vec::new();
    let data = LPARAM(std::ptr::from_mut(&mut out) as isize);

    // SAFETY: the callback and the pointer to `out` stay valid for the whole
    // synchronous call; None/None means "every monitor, no clipping region".
    let ok = unsafe { EnumDisplayMonitors(None, None, Some(enum_proc), data).as_bool() };
    if !ok {
        tracing::error!("EnumDisplayMonitors failed");
    }

    out
}

// --- QueryDisplayConfig ------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct DisplayDescriptor {
    device_path: Option<String>,
    friendly_name: Option<String>,
}

/// Maps `\\.\DISPLAYn` to a device path and a panel name.
///
/// Returns an empty map on any failure — the caller then falls back to the
/// system names.
fn display_descriptors() -> HashMap<String, DisplayDescriptor> {
    let mut map = HashMap::new();

    let Some((paths, _modes)) = query_display_config() else {
        return map;
    };

    for path in paths {
        let Some(source_name) = source_device_name(&path) else {
            continue;
        };
        // In clone mode several paths share one source; the first one decides
        // which panel the UI shows.
        if map.contains_key(&source_name) {
            continue;
        }

        let target = target_device_name(&path);
        map.insert(source_name, target.unwrap_or_default());
    }

    map
}

fn query_display_config() -> Option<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    let mut path_count = 0u32;
    let mut mode_count = 0u32;

    // SAFETY: both pointers refer to locals that outlive the call.
    let status = unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
    };
    if let Err(err) = win32_result(status, "GetDisplayConfigBufferSizes") {
        tracing::warn!("{err}");
        return None;
    }

    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

    // SAFETY: the buffer lengths match the path_count/mode_count obtained
    // above, and QueryDisplayConfig writes no more than it is told.
    let status = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
    };
    if let Err(err) = win32_result(status, "QueryDisplayConfig") {
        tracing::warn!("{err}");
        return None;
    }

    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);
    Some((paths, modes))
}

fn source_device_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
    let mut request = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            size: size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
            adapterId: path.sourceInfo.adapterId,
            id: path.sourceInfo.id,
        },
        ..Default::default()
    };

    // SAFETY: header.size describes exactly the struct we pass a pointer to;
    // the API reads size to learn which packet type to expect.
    let status =
        unsafe { DisplayConfigGetDeviceInfo(std::ptr::from_mut(&mut request).cast()) } as u32;

    if status != ERROR_SUCCESS.0 {
        return None;
    }

    let name = wide_to_string(&request.viewGdiDeviceName);
    (!name.is_empty()).then_some(name)
}

fn target_device_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<DisplayDescriptor> {
    let mut request = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            size: size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
            adapterId: path.targetInfo.adapterId,
            id: path.targetInfo.id,
        },
        ..Default::default()
    };

    // SAFETY: same contract as source_device_name.
    let status =
        unsafe { DisplayConfigGetDeviceInfo(std::ptr::from_mut(&mut request).cast()) } as u32;

    if status != ERROR_SUCCESS.0 {
        return None;
    }

    let friendly = wide_to_string(&request.monitorFriendlyDeviceName);
    let path = wide_to_string(&request.monitorDevicePath);

    Some(DisplayDescriptor {
        device_path: (!path.is_empty()).then_some(path),
        friendly_name: (!friendly.is_empty()).then_some(friendly),
    })
}

fn win32_result(status: WIN32_ERROR, call: &str) -> Result<(), String> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("{call} returned code {}", status.0))
    }
}

// --- Helpers -----------------------------------------------------------------

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

/// FNV-1a: a short deterministic key for the config.
///
/// Hand rolled rather than `DefaultHasher`, because the latter offers no
/// stability guarantees across Rust versions — and these keys live in the
/// user's config file.
fn fnv1a_hex(data: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in data.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_to_string_stops_at_nul() {
        let buf = [b'A' as u16, b'B' as u16, 0, b'C' as u16];
        assert_eq!(wide_to_string(&buf), "AB");
    }

    #[test]
    fn wide_to_string_handles_full_buffer() {
        let buf = [b'A' as u16, b'B' as u16];
        assert_eq!(wide_to_string(&buf), "AB");
    }

    #[test]
    fn fnv1a_matches_reference_vectors() {
        // Reference values for 64-bit FNV-1a.
        assert_eq!(fnv1a_hex(""), "cbf29ce484222325");
        assert_eq!(fnv1a_hex("a"), "af63dc4c8601ec8c");
    }

    #[test]
    fn fnv1a_is_stable_and_distinct() {
        assert_eq!(fnv1a_hex(r"\\.\DISPLAY1"), fnv1a_hex(r"\\.\DISPLAY1"));
        assert_ne!(fnv1a_hex(r"\\.\DISPLAY1"), fnv1a_hex(r"\\.\DISPLAY2"));
    }

    fn monitor(id: &str, index: u32, label: &str) -> MonitorInfo {
        MonitorInfo {
            id: id.into(),
            index,
            device_name: format!(r"\\.\DISPLAY{index}"),
            label: label.into(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: index == 1,
            device_path: None,
            handle: 40 + index as isize,
        }
    }

    #[test]
    fn identical_labels_get_numbered() {
        let labels: Vec<String> = disambiguate_labels(vec![
            monitor("a", 1, "LS24C43xG"),
            monitor("b", 2, "LS24C43xG"),
            monitor("c", 3, "DELL U2720Q"),
        ])
        .into_iter()
        .map(|m| m.label)
        .collect();

        assert_eq!(labels, ["LS24C43xG #1", "LS24C43xG #2", "DELL U2720Q"]);
    }

    #[test]
    fn snapshot_lookup_by_id() {
        let snapshot = MonitorSnapshot {
            monitors: vec![monitor("abc", 1, "Test")],
        };

        assert!(snapshot.by_id("abc").is_some());
        assert!(snapshot.by_id("nope").is_none());
        assert!(snapshot.by_handle(HMONITOR(41 as *mut _)).is_some());
    }
}

//! The only unsafe code of the crate: the Win32 calls the icon library does not cover. Process
//! liveness, the uptime counter, the single-instance mutex, the message pump, dark popup menus
//! the balloon tip, the DPI of the screen and the size of a small icon. Every block says why it holds.
#![allow(unsafe_code)]

use std::ffi::c_void;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::{
    CreateMutexW, GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem, GetDpiForWindow,
    GetSystemMetricsForDpi, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Shell::{
    NIF_INFO, NIIF_INFO, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, MsgWaitForMultipleObjects, PM_REMOVE, PeekMessageW, QS_ALLINPUT,
    SM_CXSMICON, TranslateMessage, WM_QUIT,
};
use windows::core::{HSTRING, PCSTR, w};

use crate::tray::error::TrayError;

const STILL_ACTIVE: u32 = 259;
const SET_PREFERRED_APP_MODE: usize = 135;
const FLUSH_MENU_THEMES: usize = 136;
const APP_MODE_ALLOW_DARK: i32 = 1;

type SetPreferredAppMode = unsafe extern "system" fn(i32) -> i32;
type FlushMenuThemes = unsafe extern "system" fn();
type RawProc = unsafe extern "system" fn() -> isize;

/// True when `pid` is a running process. A process that denies the query still counts as alive.
#[must_use]
pub fn process_alive(pid: u32) -> bool {
    // SAFETY: OpenProcess takes plain values and returns an owned handle or an error.
    let opened = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
    let Ok(handle) = opened else {
        return opened.is_err_and(|e| e.code() == ERROR_ACCESS_DENIED.to_hresult());
    };
    let mut code = 0;
    // SAFETY: the handle was just opened and is ours; `code` is a valid u32 for the call.
    let queried = unsafe { GetExitCodeProcess(handle, &raw mut code) }.is_ok();
    // SAFETY: the handle is ours and not used again.
    let _closed = unsafe { CloseHandle(handle) };
    queried && code == STILL_ACTIVE
}

/// Milliseconds the machine has been up.
pub fn uptime_ms() -> u64 {
    // SAFETY: GetTickCount64 takes no arguments and only reads the system tick counter.
    unsafe { GetTickCount64() }
}

/// The single-instance mutex of one dashboard home, held until it is dropped.
pub struct InstanceGuard(HANDLE);

impl InstanceGuard {
    /// Takes the named mutex.
    ///
    /// # Errors
    ///
    /// `AlreadyRunning` when another process holds it, `Platform` when Windows refuses.
    pub fn acquire(name: &str) -> Result<Self, TrayError> {
        let wide = HSTRING::from(name);
        // SAFETY: `wide` is a valid string that outlives the call; no security attributes.
        let handle = unsafe { CreateMutexW(None, true, &wide) }
            .map_err(|e| TrayError::Platform(e.to_string()))?;
        // SAFETY: reads the last error of this thread, set by the CreateMutexW call above.
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // SAFETY: the handle is ours and not used again.
            let _closed = unsafe { CloseHandle(handle) };
            return Err(TrayError::AlreadyRunning);
        }
        Ok(Self(handle))
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        // SAFETY: the handle is ours and this is its last use.
        let _closed = unsafe { CloseHandle(self.0) };
    }
}

/// Waits up to `timeout_ms` for a message of this thread, then dispatches every message that
/// waits. False once the thread got `WM_QUIT`.
pub fn pump(timeout_ms: u32) -> bool {
    // SAFETY: no handles are waited on, only this thread's own message queue.
    unsafe { MsgWaitForMultipleObjects(None, false, timeout_ms, QS_ALLINPUT) };
    let mut message = MSG::default();
    // SAFETY: `message` is a valid MSG owned by this frame; the queue is this thread's.
    while unsafe { PeekMessageW(&raw mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
        if message.message == WM_QUIT {
            return false;
        }
        // SAFETY: `message` was filled by PeekMessageW just above.
        unsafe {
            let _translated = TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        }
    }
    true
}

/// Lets popup menus follow the system light or dark mode (the undocumented `uxtheme` entries
/// every dark-mode aware app uses). Call it before the first menu is built; it does nothing
/// when Windows lacks the entries.
pub fn allow_dark_menus() {
    // SAFETY: the library is the system's uxtheme, loaded for the life of the process.
    let Ok(library) = (unsafe { LoadLibraryW(w!("uxtheme.dll")) }) else {
        return;
    };
    let entry = |ordinal: usize| {
        let name = PCSTR(std::ptr::without_provenance::<u8>(ordinal));
        // SAFETY: an ordinal in the low word of the name pointer is how GetProcAddress takes one.
        unsafe { GetProcAddress(library, name) }
    };
    let (Some(set), Some(flush)) = (entry(SET_PREFERRED_APP_MODE), entry(FLUSH_MENU_THEMES)) else {
        return;
    };
    // SAFETY: these ordinals have these signatures on every Windows that has the entries.
    unsafe {
        let set = std::mem::transmute::<RawProc, SetPreferredAppMode>(set);
        let flush = std::mem::transmute::<RawProc, FlushMenuThemes>(flush);
        set(APP_MODE_ALLOW_DARK);
        flush();
    }
}

/// Makes this process per-monitor DPI aware (version 2), so Windows hands it the real size of a
/// small icon and never stretches the bitmaps it draws. Must run before the first window of the
/// process. Fails, and is ignored, when the awareness was already set.
pub fn make_dpi_aware() {
    // SAFETY: SetProcessDpiAwarenessContext takes a constant context value and reads no memory.
    let _set = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// The DPI of the screen the notification area is on: that of `window` (the tray window) when
/// there is one, else that of the system. 96 is 100 percent.
pub fn dpi(window: Option<*mut c_void>) -> u32 {
    // SAFETY: GetDpiForWindow takes a handle and returns 0 for one that is not valid.
    let own = window.map_or(0, |raw| unsafe { GetDpiForWindow(HWND(raw)) });
    if own != 0 {
        return own;
    }
    // SAFETY: GetDpiForSystem takes no arguments and reads no memory of ours.
    unsafe { GetDpiForSystem() }
}

/// The width in pixels of a small icon at `dpi`, which is what the notification area draws:
/// 16 at 96 dpi, 20 at 120, 24 at 144, 32 at 192.
pub fn small_icon_px(dpi: u32) -> u32 {
    // SAFETY: GetSystemMetricsForDpi takes an index and a DPI and reads no memory of ours.
    let width = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    u32::try_from(width).ok().filter(|px| *px > 0).unwrap_or(16)
}

/// Shows a balloon tip on the notification icon `id` of the window `window`.
pub fn show_balloon(window: *mut c_void, id: u32, title: &str, text: &str) -> bool {
    let mut data = NOTIFYICONDATAW {
        cbSize: u32::try_from(size_of::<NOTIFYICONDATAW>()).unwrap_or(0),
        hWnd: HWND(window),
        uID: id,
        uFlags: NIF_INFO,
        dwInfoFlags: NIIF_INFO,
        ..Default::default()
    };
    fill(&mut data.szInfoTitle, title);
    fill(&mut data.szInfo, text);
    // SAFETY: `data` is a fully initialised NOTIFYICONDATAW with its size set.
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &raw const data) }.as_bool()
}

fn fill(buffer: &mut [u16], value: &str) {
    let room = buffer.len().saturating_sub(1);
    for (slot, unit) in buffer.iter_mut().zip(value.encode_utf16().take(room)) {
        *slot = unit;
    }
}

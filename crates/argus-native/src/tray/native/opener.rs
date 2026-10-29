//! How the tray opens a page: it hands the foreground right the shell gave it for the click on to
//! the browser, then asks the shell to open the address, the way Explorer does. Both answer a
//! reason when they fail, so the tray can log it. Every unsafe block says why it holds.
#![allow(unsafe_code)]

use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow, SW_SHOWNORMAL};
use windows::core::{HSTRING, w};

/// `ShellExecuteW` answers a value above this on success, an error code at or below it.
const SHELL_EXECUTE_OK: isize = 32;

/// Lets the next process that asks take the foreground. The shell gives the tray that right for
/// a moment when the owner clicks the icon or its menu; handing it on lets the browser come to
/// the front instead of opening behind the other windows.
///
/// # Errors
///
/// The tray held no foreground right to hand on.
pub fn pass_foreground() -> Result<(), String> {
    // SAFETY: AllowSetForegroundWindow takes a plain process id and reads no memory of ours.
    unsafe { AllowSetForegroundWindow(ASFW_ANY) }.map_err(|e| e.to_string())
}

/// Opens an address with the program Windows has for it (the default browser for `http`).
/// Blocks until the shell has handed it over, so it runs off the async threads.
///
/// # Errors
///
/// The shell's error code when it could not open the address.
pub fn open_url(address: &str) -> Result<(), String> {
    let target = HSTRING::from(address);
    // SAFETY: sets up COM for this thread only; the shell may use it to reach the handler.
    let com = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
    // SAFETY: the strings are valid and outlive the call; no window, parameters or folder.
    let done = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    if com.is_ok() {
        // SAFETY: balances the successful CoInitializeEx of this thread above.
        unsafe { CoUninitialize() };
    }
    let code = done.0 as isize;
    if code > SHELL_EXECUTE_OK {
        Ok(())
    } else {
        Err(format!("the shell could not open it (error {code})"))
    }
}

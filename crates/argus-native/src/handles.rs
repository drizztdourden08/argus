//! Keeps the standard handles of this process out of the processes it starts. On Windows a child
//! inherits every inheritable handle, so a server started by `argus start` would hold the pipe
//! that a caller reads the output of `argus start` from, and the caller would wait for the
//! server to exit. Off Windows file descriptors are closed on exec already.
#![cfg_attr(windows, allow(unsafe_code))]

/// Makes this process's standard input, output and error not inheritable. The processes started
/// after this call still get the handles the caller gives them explicitly.
#[cfg(windows)]
pub fn keep_stdio_private() {
    use windows::Win32::Foundation::{HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation};
    use windows::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: GetStdHandle takes a constant and returns a handle this process owns or an
        // error; SetHandleInformation only clears a flag on it and is ignored when it fails.
        unsafe {
            if let Ok(handle) = GetStdHandle(which) {
                let _cleared = SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
            }
        }
    }
}

/// Nothing to do off Windows.
#[cfg(not(windows))]
pub fn keep_stdio_private() {}

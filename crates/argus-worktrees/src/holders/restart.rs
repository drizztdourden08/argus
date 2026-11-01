//! Windows' Restart Manager: the processes that have one of the given files open. Elsewhere it knows none.

#![cfg_attr(windows, allow(unsafe_code))]

/// A process that has one of the files open: its id and the name Windows gives the application.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct FileHolder {
    pub pid: u32,
    pub app: String,
}

#[cfg(not(windows))]
pub(crate) fn file_holders(_files: &[String]) -> Vec<FileHolder> {
    Vec::new()
}

#[cfg(windows)]
pub(crate) use win::file_holders;

#[cfg(windows)]
mod win {
    use super::FileHolder;
    use std::ptr::null;
    use windows_sys::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
    use windows_sys::Win32::System::RestartManager::{
        CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList, RmRegisterResources,
        RmStartSession,
    };
    use windows_sys::core::PCWSTR;

    /// How many times the list is asked again when it grew between two asks.
    const TRIES: usize = 3;

    /// A Restart Manager session, ended when dropped.
    struct Session(u32);

    impl Drop for Session {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful RmStartSession and is ended once.
            unsafe {
                RmEndSession(self.0);
            }
        }
    }

    fn wide(path: &str) -> Vec<u16> {
        path.replace('/', "\\")
            .encode_utf16()
            .chain(Some(0))
            .collect()
    }

    fn start() -> Option<Session> {
        let mut handle = 0u32;
        let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
        // SAFETY: both pointers are to live locals; the key buffer has room for CCH_RM_SESSION_KEY + 1 characters.
        let said = unsafe { RmStartSession(&raw mut handle, 0, key.as_mut_ptr()) };
        (said == ERROR_SUCCESS).then_some(Session(handle))
    }

    fn register(session: &Session, files: &[String]) -> bool {
        let names: Vec<Vec<u16>> = files.iter().map(String::as_str).map(wide).collect();
        let pointers: Vec<PCWSTR> = names.iter().map(Vec::as_ptr).collect();
        let Ok(count) = u32::try_from(pointers.len()) else {
            return false;
        };
        // SAFETY: `pointers` holds `count` null-terminated wide strings that `names` keeps alive for the call.
        let said = unsafe {
            RmRegisterResources(session.0, count, pointers.as_ptr(), 0, null(), 0, null())
        };
        said == ERROR_SUCCESS
    }

    fn holder_of(info: &RM_PROCESS_INFO) -> FileHolder {
        let name = &info.strAppName;
        let end = name.iter().position(|c| *c == 0).unwrap_or(name.len());
        FileHolder {
            pid: info.Process.dwProcessId,
            app: String::from_utf16_lossy(&name[..end]),
        }
    }

    fn listed(session: &Session) -> Vec<FileHolder> {
        let mut infos = vec![RM_PROCESS_INFO::default(); 16];
        for _ in 0..TRIES {
            let (mut needed, mut reasons) = (0u32, 0u32);
            let mut count = u32::try_from(infos.len()).unwrap_or(0);
            // SAFETY: `infos` has room for `count` entries; the other pointers are to live locals.
            let said = unsafe {
                RmGetList(
                    session.0,
                    &raw mut needed,
                    &raw mut count,
                    infos.as_mut_ptr(),
                    &raw mut reasons,
                )
            };
            if said == ERROR_SUCCESS {
                infos.truncate(usize::try_from(count).unwrap_or(0));
                return infos.iter().map(holder_of).collect();
            }
            if said != ERROR_MORE_DATA {
                return Vec::new();
            }
            let room = usize::try_from(needed).unwrap_or(0) + 4;
            infos = vec![RM_PROCESS_INFO::default(); room];
        }
        Vec::new()
    }

    /// The processes that have one of the files open; none when Windows cannot tell.
    pub(crate) fn file_holders(files: &[String]) -> Vec<FileHolder> {
        if files.is_empty() {
            return Vec::new();
        }
        let Some(session) = start() else {
            return Vec::new();
        };
        if !register(&session, files) {
            return Vec::new();
        }
        listed(&session)
    }
}

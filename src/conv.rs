use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use winapi::um::{winbase::lstrlenW, winnt::PCWSTR};

pub trait WStrExt {
    fn to_wstr(&self) -> PCWSTR;
}

impl<T> WStrExt for T
where
    T: AsRef<OsStr>,
{
    fn to_wstr(&self) -> PCWSTR {
        self.as_ref()
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<u16>>()
            .as_ptr()
    }
}

pub trait RawWStrExt {
    fn to_os(&self) -> OsString;
}

impl RawWStrExt for PCWSTR {
    fn to_os(&self) -> OsString {
        let length = unsafe { lstrlenW(*self) as usize };
        let wstr = unsafe { std::slice::from_raw_parts(*self, length) };
        OsString::from_wide(wstr)
    }
}

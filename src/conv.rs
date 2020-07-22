use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use winapi::um::{winbase::lstrlenW, winnt::PCWSTR};

pub struct WStr {
    data: Vec<u16>,
}

impl WStr {
    pub fn as_ptr(&self) -> PCWSTR {
        self.data.as_ptr()
    }
}

pub trait WStrExt {
    fn to_wstr(&self) -> WStr;
}

impl<T> WStrExt for T
where
    T: AsRef<OsStr>,
{
    fn to_wstr(&self) -> WStr {
        let data = self
            .as_ref()
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<u16>>();

        WStr { data }
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

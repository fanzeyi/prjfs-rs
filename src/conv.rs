use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::um::winnt::PCWSTR;

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

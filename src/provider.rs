use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::ptr::null_mut;
use winapi::um::projectedfslib as prjfs;

use crate::conv::WStrExt;
use crate::guid;

const GUID_FILE: &'static str = ".regfsId";

mod ffi {
    use winapi::shared::guiddef::GUID;
    use winapi::um::projectedfslib as prjfs;
    use winapi::um::winnt::{HRESULT, PCWSTR};

    pub unsafe extern "system" fn start_dir_enum_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        enumeration: *const GUID,
    ) -> HRESULT {
        unimplemented!()
    }

    pub unsafe extern "system" fn end_dir_enum_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        enumeration: *const GUID,
    ) -> HRESULT {
        unimplemented!()
    }

    pub unsafe extern "system" fn get_dir_enum_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        enumeration: *const GUID,
        search_expression: PCWSTR,
        dir_entry_buffer_handle: prjfs::PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> HRESULT {
        unimplemented!()
    }

    pub unsafe extern "system" fn get_placeholder_info_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
    ) -> HRESULT {
        unimplemented!()
    }

    pub unsafe extern "system" fn get_file_data_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        offset: u64,
        length: u32,
    ) -> HRESULT {
        unimplemented!()
    }
}
pub struct Provider {
    root_path: PathBuf,
    options: prjfs::PRJ_STARTVIRTUALIZING_OPTIONS,
}

impl Provider {
    pub fn new(
        root_path: PathBuf,
        options: prjfs::PRJ_STARTVIRTUALIZING_OPTIONS,
    ) -> Result<Provider> {
        let provider = Provider { root_path, options };
        provider.ensure_virtualization_root()?;
        let mut callbacks: prjfs::PRJ_CALLBACKS = Default::default();
        callbacks.StartDirectoryEnumerationCallback =
            Box::into_raw(Box::new(Some(ffi::start_dir_enum_callback_c)));
        callbacks.EndDirectoryEnumerationCallback =
            Box::into_raw(Box::new(Some(ffi::end_dir_enum_callback_c)));
        callbacks.GetDirectoryEnumerationCallback =
            Box::into_raw(Box::new(Some(ffi::get_dir_enum_callback_c)));
        callbacks.GetPlaceholderInfoCallback =
            Box::into_raw(Box::new(Some(ffi::get_placeholder_info_callback_c)));
        callbacks.GetFileDataCallback =
            Box::into_raw(Box::new(Some(ffi::get_file_data_callback_c)));
        Ok(provider)
    }

    fn ensure_virtualization_root(&self) -> Result<()> {
        let guid_file = self.root_path.join(GUID_FILE);

        if self.root_path.exists() && self.root_path.is_dir() {
            if !self.root_path.is_dir() {
                return Err(anyhow!(format!("{:?} is not a directory", self.root_path)));
            }
            // virtualization root is present, attempts to read guid
            let guid = std::fs::read(&guid_file)?;
            guid::guid_from_bytes(guid).map_err(|_| anyhow!("unable to read GUID"))?;
            Ok(())
        } else {
            let guid = guid::create_guid();
            std::fs::create_dir(&self.root_path)?;
            std::fs::write(&guid_file, guid::guid_to_bytes(guid))?;
            let hr = unsafe {
                prjfs::PrjMarkDirectoryAsPlaceholder(
                    self.root_path.clone().to_wstr(),
                    null_mut(),
                    null_mut(),
                    &guid,
                )
            };
            if hr < 0 {
                // failed, clean up
                let _ = std::fs::remove_file(&guid_file);
                let _ = std::fs::remove_dir(&self.root_path);
                return Err(anyhow!(format!("HRESULT: {}", hr)));
            }
            Ok(())
        }
    }
}

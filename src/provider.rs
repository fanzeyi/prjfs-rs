use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use winapi::shared::guiddef::GUID;
use winapi::um::projectedfslib as prjfs;
use winapi::{
    ctypes::c_void,
    um::winnt::{HRESULT, PCWSTR},
};

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
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).start_dir_enum(&*data, &*enumeration)
    }

    pub unsafe extern "system" fn end_dir_enum_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        enumeration: *const GUID,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).end_dir_enum(&*data, &*enumeration)
    }

    pub unsafe extern "system" fn get_dir_enum_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        enumeration: *const GUID,
        search_expression: PCWSTR,
        dir_entry_buffer_handle: prjfs::PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).get_dir_enum(
            &*data,
            &*enumeration,
            search_expression,
            dir_entry_buffer_handle,
        )
    }

    pub unsafe extern "system" fn get_placeholder_info_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).get_placeholder_info(&*data)
    }

    pub unsafe extern "system" fn get_file_data_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        offset: u64,
        length: u32,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).get_file_data(&*data, offset, length)
    }

    pub unsafe extern "system" fn notification_callback_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
        is_directory: bool,
        notification_type: prjfs::PRJ_NOTIFICATION,
        destination_file_name: PCWSTR,
        parameters: *mut prjfs::PRJ_NOTIFICATION_PARAMETERS,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).notify(
            &*data,
            is_directory,
            notification_type,
            destination_file_name,
            &*parameters,
        )
    }

    pub unsafe extern "system" fn query_file_name_c(
        data: *const prjfs::PRJ_CALLBACK_DATA,
    ) -> HRESULT {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).query_file_name(&*data)
    }

    pub unsafe extern "system" fn cancel_command_c(data: *const prjfs::PRJ_CALLBACK_DATA) {
        let provider = (*data).InstanceContext.cast::<super::Provider>();
        (*provider).cancel_command(&*data);
    }
}

pub trait ProviderT {
    fn set_context(&mut self, context: prjfs::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT);

    fn start_dir_enum(
        &self,
        callback_data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> Result<HRESULT>;
    fn end_dir_enum(
        &self,
        callback_data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> Result<HRESULT>;
    fn get_dir_enum(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration: &GUID,
        search_expression: PCWSTR,
        dir_entry_buffer_handle: prjfs::PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> Result<HRESULT>;
    fn get_placeholder_info(&self, data: &prjfs::PRJ_CALLBACK_DATA) -> Result<HRESULT>;
    fn get_file_data(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        offset: u64,
        length: u32,
    ) -> Result<HRESULT>;
    fn notify(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        is_directory: bool,
        notification_type: prjfs::PRJ_NOTIFICATION,
        destination_file_name: PCWSTR,
        parameters: &prjfs::PRJ_NOTIFICATION_PARAMETERS,
    ) -> Result<HRESULT>;
    fn query_file_name(&self, data: &prjfs::PRJ_CALLBACK_DATA) -> Result<HRESULT>;
    fn cancel_command(&self, data: &prjfs::PRJ_CALLBACK_DATA) -> Result<()>;
}

pub struct Provider {
    inner: *mut Box<dyn ProviderT>,
}

macro_rules! wintry {
    ($e: expr) => {
        match $e {
            Ok(result) => result,
            Err(_) => winapi::shared::winerror::S_FALSE,
        }
    };
}

impl Provider {
    pub fn new(
        root_path: PathBuf,
        options: prjfs::PRJ_STARTVIRTUALIZING_OPTIONS,
        inner: Box<dyn ProviderT>,
    ) -> Result<Provider> {
        Self::ensure_virtualization_root(&root_path)?;

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
        callbacks.QueryFileNameCallback = Box::into_raw(Box::new(Some(ffi::query_file_name_c)));
        callbacks.CancelCommandCallback = Box::into_raw(Box::new(Some(ffi::cancel_command_c)));

        let context: *mut prjfs::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT =
            unsafe { std::mem::zeroed() };

        let inner = Box::into_raw(Box::new(inner));
        let instance: *const c_void = unsafe { &*(inner as *mut c_void) };

        unsafe {
            // TODO: check HRESULT
            prjfs::PrjStartVirtualizing(
                root_path.into_os_string().to_wstr(),
                &callbacks,
                instance,
                &options,
                context,
            );
        }

        // is this.. UB?
        unsafe {
            (&mut *inner).set_context(*context);
        }

        let provider = Provider { inner };

        Ok(provider)
    }

    fn ensure_virtualization_root<T: AsRef<Path>>(root_path: T) -> Result<()> {
        let root_path = root_path.as_ref();
        let guid_file = root_path.join(GUID_FILE);

        if root_path.exists() && root_path.is_dir() {
            if !root_path.is_dir() {
                return Err(anyhow!(format!("{:?} is not a directory", root_path)));
            }
            // virtualization root is present, attempts to read guid
            let guid = std::fs::read(&guid_file)?;
            guid::guid_from_bytes(guid).map_err(|_| anyhow!("unable to read GUID"))?;
            Ok(())
        } else {
            let guid = guid::create_guid();
            std::fs::create_dir(&root_path)?;
            std::fs::write(&guid_file, guid::guid_to_bytes(&guid))?;
            let hr = unsafe {
                prjfs::PrjMarkDirectoryAsPlaceholder(
                    root_path.clone().to_wstr(),
                    null_mut(),
                    null_mut(),
                    &guid,
                )
            };
            if hr < 0 {
                // failed, clean up
                let _ = std::fs::remove_file(&guid_file);
                let _ = std::fs::remove_dir(&root_path);
                return Err(anyhow!(format!("HRESULT: {}", hr)));
            }
            Ok(())
        }
    }

    pub fn start_dir_enum(
        &self,
        callback_data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.start_dir_enum(callback_data, enumeration_id))
    }

    pub fn end_dir_enum(
        &self,
        callback_data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.end_dir_enum(callback_data, enumeration_id))
    }

    pub fn get_dir_enum(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        enumeration: &GUID,
        search_expression: PCWSTR,
        dir_entry_buffer_handle: prjfs::PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.get_dir_enum(
            data,
            enumeration,
            search_expression,
            dir_entry_buffer_handle,
        ))
    }

    pub fn get_placeholder_info(&self, data: &prjfs::PRJ_CALLBACK_DATA) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.get_placeholder_info(data))
    }

    pub fn get_file_data(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        offset: u64,
        length: u32,
    ) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.get_file_data(data, offset, length))
    }

    pub fn notify(
        &self,
        data: &prjfs::PRJ_CALLBACK_DATA,
        is_directory: bool,
        notification_type: prjfs::PRJ_NOTIFICATION,
        destination_file_name: PCWSTR,
        parameters: &prjfs::PRJ_NOTIFICATION_PARAMETERS,
    ) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.notify(
            data,
            is_directory,
            notification_type,
            destination_file_name,
            parameters,
        ))
    }

    pub fn query_file_name(&self, data: &prjfs::PRJ_CALLBACK_DATA) -> HRESULT {
        let inner = unsafe { &*self.inner };
        wintry!(inner.query_file_name(data))
    }

    pub fn cancel_command(&self, data: &prjfs::PRJ_CALLBACK_DATA) {
        let inner = unsafe { &*self.inner };
        let _ = inner.cancel_command(data);
    }
}

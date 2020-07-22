use anyhow::{anyhow, Result};
use log::{info, warn};
use std::{collections::HashMap, ffi::OsString, sync::Mutex};
use winapi::{
    shared::{
        guiddef::GUID,
        winerror::{self, HRESULT_FROM_WIN32, S_OK},
    },
    um::{
        projectedfslib::{
            self as prjfs, PRJ_CALLBACK_DATA, PRJ_DIR_ENTRY_BUFFER_HANDLE,
            PRJ_NOTIFICATION_PARAMETERS,
        },
        winnt::{HRESULT, LPCWSTR, PCWSTR},
    },
};

use crate::conv::RawWStrExt;
use crate::dirinfo::DirInfo;
use crate::guid::guid_to_bytes;
use crate::provider::ProviderT;

#[derive(Default)]
pub struct State {
    enum_sessions: HashMap<Vec<u8>, DirInfo>,
}

pub struct RegFs {
    state: Mutex<State>,
    readonly: bool,
    context: prjfs::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT,
}

impl RegFs {
    pub fn new() -> Self {
        RegFs {
            state: Mutex::new(Default::default()),
            readonly: true,
            context: std::ptr::null_mut(),
        }
    }
}

impl RegFs {
    fn write_placeholder_info(
        &self,
        filepath: LPCWSTR,
        info: prjfs::PRJ_PLACEHOLDER_INFO,
    ) -> HRESULT {
        unsafe {
            prjfs::PrjWritePlaceholderInfo(
                self.context,
                filepath,
                &info,
                std::mem::size_of_val(&info) as u32,
            )
        }
    }

    fn populate_dir_info_for_path(
        &self,
        path: OsString,
        dirinfo: &mut DirInfo,
        search_expression: OsString,
    ) {
    }
}

impl ProviderT for RegFs {
    fn get_context_space(&mut self) -> Option<*mut prjfs::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT> {
        Some(&mut self.context)
    }

    fn start_dir_enum(
        &self,
        callback_data: &PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> Result<HRESULT> {
        let filepath = callback_data.FilePathName.to_os();
        info!(
            "----> start_dir_enum: Path [{:?}] triggered by [{:?}]",
            filepath,
            callback_data.TriggeringProcessImageFileName.to_os()
        );

        let guid = guid_to_bytes(enumeration_id);
        self.state
            .lock()
            .unwrap()
            .enum_sessions
            .insert(guid, DirInfo::new(filepath));

        info!("<---- start_dir_enum: return 0x0");

        Ok(0)
    }

    fn end_dir_enum(
        &self,
        _callback_data: &PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
    ) -> Result<HRESULT> {
        info!("----> end_dir_enum");

        let guid = guid_to_bytes(enumeration_id);
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("unable to acquire state"))?;

        state.enum_sessions.remove(&guid);

        info!("<---- end_dir_enum: return 0x0");
        Ok(0)
    }

    fn get_dir_enum(
        &self,
        data: &PRJ_CALLBACK_DATA,
        enumeration_id: &GUID,
        search_expression: PCWSTR,
        handle: PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> Result<HRESULT> {
        let path = data.FilePathName.to_os();
        let search_expression = search_expression.to_os();
        info!(
            "----> get_dir_enum: Path [{:?}] SearchExpression: [{:?}]",
            path, search_expression
        );

        let guid = guid_to_bytes(enumeration_id);
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("unable to acquire state"))?;

        let dirinfo = match state.enum_sessions.get_mut(&guid) {
            Some(session) => session,
            None => return Ok(winerror::E_INVALIDARG),
        };

        if data.Flags & prjfs::PRJ_CB_DATA_FLAG_ENUM_RESTART_SCAN != 0 {
            dirinfo.reset();
        }

        if !dirinfo.filled() {
            self.populate_dir_info_for_path(path, dirinfo, search_expression);
        }

        while dirinfo.current_is_valid() {
            let result = unsafe {
                prjfs::PrjFillDirEntryBuffer(
                    dirinfo.current_file_name(),
                    &mut dirinfo.current_basic_info(),
                    handle,
                )
            };

            if result != S_OK {
                break;
            }

            dirinfo.move_next();
        }

        info!("<---- get_dir_enum: return {:08x}", 0);
        Ok(S_OK)
    }

    fn get_placeholder_info(&self, data: &PRJ_CALLBACK_DATA) -> Result<HRESULT> {
        let filepath = data.FilePathName.to_os();
        info!(
            "----> get_placeholder_info: Path [{:?}] triggered by {:?}]",
            filepath,
            data.TriggeringProcessImageFileName.to_os()
        );

        let iskey = if false { true } else { false };
        let size = 0;

        let mut placeholder = prjfs::PRJ_PLACEHOLDER_INFO::default();
        placeholder.FileBasicInfo.IsDirectory = iskey as u8;
        placeholder.FileBasicInfo.FileSize = size;

        let result = self.write_placeholder_info(data.FilePathName, placeholder);

        info!("<---- get_placeholder_info: {:08x}", result);

        Ok(result)
    }

    fn get_file_data(&self, data: &PRJ_CALLBACK_DATA, offset: u64, length: u32) -> Result<HRESULT> {
        let path = data.FilePathName.to_os();
        let process = data.TriggeringProcessImageFileName.to_os();
        info!(
            "----> get_file_data: Path[{:?}] triggered by [{:?}]",
            path, process
        );

        let instance_info = {
            let ptr = std::ptr::null_mut();
            let hr = unsafe { prjfs::PrjGetVirtualizationInstanceInfo(self.context, ptr) };
            if winerror::FAILED(hr) {
                warn!(
                    "<---- get_file_data: PrjGetVirtualizationInstanceInfo: {:08x}",
                    hr
                );
                return Ok(hr);
            }
            ptr
        };

        let buffer = unsafe { prjfs::PrjAllocateAlignedBuffer(self.context, length as usize) };
        if buffer.is_null() {
            warn!("<---- get_file_data: Could not allocate write buffer.");
            return Ok(winerror::E_OUTOFMEMORY);
        }

        unsafe {
            prjfs::PrjFreeAlignedBuffer(buffer);
        }

        info!("<---- get_file_data: return {:08x}", S_OK);
        Ok(S_OK)
    }

    fn notify(
        &self,
        data: &PRJ_CALLBACK_DATA,
        _is_directory: bool,
        notification_type: prjfs::PRJ_NOTIFICATION,
        destination_file_name: PCWSTR,
        _parameters: &PRJ_NOTIFICATION_PARAMETERS,
    ) -> Result<HRESULT> {
        let filepath = data.FilePathName.to_os();
        let process = data.TriggeringProcessImageFileName.to_os();
        info!(
            "---> notify: Path [{:?}] triggered by [{:?}]",
            filepath, process
        );
        info!("--- Notification: 0x{:08x}", notification_type);

        match notification_type {
            prjfs::PRJ_NOTIFICATION_FILE_OPENED => Ok(S_OK),
            prjfs::PRJ_NOTIFICATION_FILE_HANDLE_CLOSED_FILE_MODIFIED
            | prjfs::PRJ_NOTIFICATION_FILE_OVERWRITTEN => {
                info!(" ----- [{:?}] was modified", filepath);
                Ok(S_OK)
            }
            prjfs::PRJ_NOTIFY_NEW_FILE_CREATED => {
                info!(" ----- [{:?}] was created", filepath);
                Ok(S_OK)
            }
            prjfs::PRJ_NOTIFY_FILE_RENAMED => {
                info!(
                    " ----- [{:?}] -> [{:?}]",
                    filepath,
                    destination_file_name.to_os()
                );
                Ok(S_OK)
            }
            prjfs::PRJ_NOTIFY_FILE_HANDLE_CLOSED_FILE_DELETED => {
                info!(" ----- [{:?}] was deleted", filepath);
                Ok(S_OK)
            }
            prjfs::PRJ_NOTIFICATION_PRE_RENAME => {
                if self.readonly {
                    info!(" ----- rename request for [{:?}] was rejected", filepath);
                    Ok(HRESULT_FROM_WIN32(winerror::ERROR_ACCESS_DENIED))
                } else {
                    info!(" ----- rename request for [{:?}]", filepath);
                    Ok(S_OK)
                }
            }
            prjfs::PRJ_NOTIFICATION_PRE_DELETE => {
                if self.readonly {
                    info!(" ----- delete request for [{:?}] was rejected", filepath);
                    Ok(HRESULT_FROM_WIN32(winerror::ERROR_ACCESS_DENIED))
                } else {
                    info!(" ----- delete request for [{:?}]", filepath);
                    Ok(S_OK)
                }
            }
            prjfs::PRJ_NOTIFICATION_FILE_PRE_CONVERT_TO_FULL => Ok(S_OK),
            t => {
                warn!("notify: Unexpected notification: 0x{:08x}", t);
                Ok(S_OK)
            }
        }
    }

    fn query_file_name(&self, _data: &PRJ_CALLBACK_DATA) -> Result<HRESULT> {
        Ok(S_OK)
    }

    fn cancel_command(&self, _data: &PRJ_CALLBACK_DATA) -> Result<()> {
        Ok(())
    }
}

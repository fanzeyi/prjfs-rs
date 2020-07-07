use anyhow::Result;
use log::{info, warn};
use std::{collections::HashMap, path::Path, sync::Mutex};
use winapi::{
    shared::{
        guiddef::GUID,
        winerror::{self, HRESULT_FROM_WIN32, S_OK},
    },
    um::{
        projectedfslib::{
            self, PRJ_CALLBACK_DATA, PRJ_DIR_ENTRY_BUFFER_HANDLE, PRJ_NOTIFICATION_PARAMETERS,
        },
        winnt::{HRESULT, PCWSTR},
    },
};

use crate::conv::RawWStrExt;
use crate::guid::guid_to_bytes;
use crate::provider::ProviderT;

struct DirInfo;

impl DirInfo {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        DirInfo
    }
}

#[derive(Default)]
pub struct State {
    enum_sessions: HashMap<Vec<u8>, DirInfo>,
}

pub struct RegFs {
    state: Mutex<State>,
    readonly: bool,
}

impl RegFs {
    pub fn new() -> Self {
        RegFs {
            state: Mutex::new(Default::default()),
            readonly: true,
        }
    }
}

impl ProviderT for RegFs {
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
        self.state.lock().unwrap().enum_sessions.remove(&guid);

        info!("<---- end_dir_enum: return 0x0");
        Ok(0)
    }
    fn get_dir_enum(
        &self,
        data: &PRJ_CALLBACK_DATA,
        enumeration: &GUID,
        search_expression: PCWSTR,
        dir_entry_buffer_handle: PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> Result<HRESULT> {
        todo!()
    }
    fn get_placeholder_info(&self, data: &PRJ_CALLBACK_DATA) -> Result<HRESULT> {
        todo!()
    }
    fn get_file_data(&self, data: &PRJ_CALLBACK_DATA, offset: u64, length: u32) -> Result<HRESULT> {
        todo!()
    }
    fn notify(
        &self,
        data: &PRJ_CALLBACK_DATA,
        _is_directory: bool,
        notification_type: projectedfslib::PRJ_NOTIFICATION,
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
            projectedfslib::PRJ_NOTIFICATION_FILE_OPENED => Ok(S_OK),
            projectedfslib::PRJ_NOTIFICATION_FILE_HANDLE_CLOSED_FILE_MODIFIED
            | projectedfslib::PRJ_NOTIFICATION_FILE_OVERWRITTEN => {
                info!(" ----- [{:?}] was modified", filepath);
                Ok(S_OK)
            }
            projectedfslib::PRJ_NOTIFY_NEW_FILE_CREATED => {
                info!(" ----- [{:?}] was created", filepath);
                Ok(S_OK)
            }
            projectedfslib::PRJ_NOTIFY_FILE_RENAMED => {
                info!(
                    " ----- [{:?}] -> [{:?}]",
                    filepath,
                    destination_file_name.to_os()
                );
                Ok(S_OK)
            }
            projectedfslib::PRJ_NOTIFY_FILE_HANDLE_CLOSED_FILE_DELETED => {
                info!(" ----- [{:?}] was deleted", filepath);
                Ok(S_OK)
            }
            projectedfslib::PRJ_NOTIFICATION_PRE_RENAME => {
                if self.readonly {
                    info!(" ----- rename request for [{:?}] was rejected", filepath);
                    Ok(HRESULT_FROM_WIN32(winerror::ERROR_ACCESS_DENIED))
                } else {
                    info!(" ----- rename request for [{:?}]", filepath);
                    Ok(S_OK)
                }
            }
            projectedfslib::PRJ_NOTIFICATION_PRE_DELETE => {
                if self.readonly {
                    info!(" ----- delete request for [{:?}] was rejected", filepath);
                    Ok(HRESULT_FROM_WIN32(winerror::ERROR_ACCESS_DENIED))
                } else {
                    info!(" ----- delete request for [{:?}]", filepath);
                    Ok(S_OK)
                }
            }
            projectedfslib::PRJ_NOTIFICATION_FILE_PRE_CONVERT_TO_FULL => Ok(S_OK),
            t => {
                warn!("notify: Unexpected notification: 0x{:08x}", t);
                Ok(S_OK)
            }
        }
    }
    fn query_file_name(&self, _data: &PRJ_CALLBACK_DATA) -> Result<HRESULT> {
        todo!()
    }
    fn cancel_command(&self, _data: &PRJ_CALLBACK_DATA) -> Result<()> {
        todo!()
    }
}

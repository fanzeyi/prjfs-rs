use crate::conv::{WStr, WStrExt};
use std::path::PathBuf;

bitflags::bitflags! {
    pub struct NotificationType: u32 {
        const NONE = 0b0000_0000_0000_0000;
        const SUPPRESS_NOTIFICATIONS = 0b0000_0000_0000_0001;
        const FILE_OPENED = 0b0000_0000_0000_0010;
        const NEW_FILE_CREATED = 0b0000_0000_0000_0100;
        const FILE_OVERWRITTEN = 0b0000_0000_0000_1000;
        const PRE_DELETE = 0b0000_0000_0001_0000;
        const PRE_RENAME = 0b0000_0000_0010_0000;
        const PRE_SET_HARDLINK = 0b0000_0000_0100_0000;
        const FILE_RENAMED = 0b0000_0000_1000_0000;
        const HARDLINK_CREATED  = 0b0000_0001_0000_0000;
        const FILE_HANDLE_CLOSED_NO_MODIFICATION = 0b0000_0010_0000_0000;
        const FILE_HANDLE_CLOSED_FILE_MODIFIED = 0b0000_0100_0000_0000;
        const FILE_HANDLE_CLOSED_FILE_DELETED = 0b0000_1000_0000_0000;
        const FILE_PRE_CONVERT_TO_FULL = 0b0001_0000_0000_0000;
        const USE_EXISTING_MASK = 0b0010_0000_0000_0000;
    }
}

impl NotificationType {
    fn into_raw(self) -> crate::sys::PRJ_NOTIFY_TYPES {
        let mut raw = crate::sys::PRJ_NOTIFY_TYPES::default();

        if self.contains(NotificationType::NONE) {
            raw = raw | crate::sys::PRJ_NOTIFY_NONE;
        }
        if self.contains(NotificationType::SUPPRESS_NOTIFICATIONS) {
            raw = raw | crate::sys::PRJ_NOTIFY_SUPPRESS_NOTIFICATIONS;
        }
        if self.contains(NotificationType::FILE_OPENED) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_OPENED;
        }
        if self.contains(NotificationType::NEW_FILE_CREATED) {
            raw = raw | crate::sys::PRJ_NOTIFY_NEW_FILE_CREATED;
        }
        if self.contains(NotificationType::FILE_OVERWRITTEN) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_OVERWRITTEN;
        }
        if self.contains(NotificationType::PRE_DELETE) {
            raw = raw | crate::sys::PRJ_NOTIFY_PRE_DELETE;
        }
        if self.contains(NotificationType::PRE_RENAME) {
            raw = raw | crate::sys::PRJ_NOTIFY_PRE_RENAME;
        }
        if self.contains(NotificationType::PRE_SET_HARDLINK) {
            raw = raw | crate::sys::PRJ_NOTIFY_PRE_SET_HARDLINK;
        }
        if self.contains(NotificationType::FILE_RENAMED) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_RENAMED;
        }
        if self.contains(NotificationType::HARDLINK_CREATED) {
            raw = raw | crate::sys::PRJ_NOTIFY_HARDLINK_CREATED;
        }
        if self.contains(NotificationType::FILE_HANDLE_CLOSED_NO_MODIFICATION) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_HANDLE_CLOSED_NO_MODIFICATION;
        }
        if self.contains(NotificationType::FILE_HANDLE_CLOSED_FILE_MODIFIED) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_HANDLE_CLOSED_FILE_MODIFIED;
        }
        if self.contains(NotificationType::FILE_HANDLE_CLOSED_FILE_DELETED) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_HANDLE_CLOSED_FILE_DELETED;
        }
        if self.contains(NotificationType::FILE_PRE_CONVERT_TO_FULL) {
            raw = raw | crate::sys::PRJ_NOTIFY_FILE_PRE_CONVERT_TO_FULL;
        }
        if self.contains(NotificationType::USE_EXISTING_MASK) {
            raw = raw | crate::sys::PRJ_NOTIFY_USE_EXISTING_MASK;
        }

        raw
    }
}

#[derive(Default)]
pub struct OptionBuilder {
    use_negative_path_cache: bool,
    pool_thread_count: Option<u32>,
    concurrent_thread_count: Option<u32>,
    notifications: Vec<(NotificationType, WStr)>,
}

impl OptionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_negative_path_cache(mut self) -> Self {
        self.use_negative_path_cache = true;
        self
    }

    pub fn pool_thread_count(mut self, count: u32) -> Self {
        self.pool_thread_count = Some(count);
        self
    }

    pub fn concurrent_thread_count(mut self, count: u32) -> Self {
        self.concurrent_thread_count = Some(count);
        self
    }

    pub fn add_root_notification(self, notification: NotificationType) -> Self {
        self.add_notification(notification, "".into())
    }

    pub fn add_notification(mut self, notification: NotificationType, path: PathBuf) -> Self {
        self.notifications.push((notification, path.to_wstr()));
        self
    }

    pub(crate) fn build(&self) -> crate::sys::PRJ_STARTVIRTUALIZING_OPTIONS {
        let mut options = crate::sys::PRJ_STARTVIRTUALIZING_OPTIONS::default();

        if self.use_negative_path_cache {
            options.Flags = crate::sys::PRJ_FLAG_USE_NEGATIVE_PATH_CACHE;
        }

        if let Some(count) = self.pool_thread_count {
            options.PoolThreadCount = count;
        }

        if let Some(count) = self.concurrent_thread_count {
            options.ConcurrentThreadCount = count;
        }

        options.NotificationMappingsCount = self.notifications.len() as u32;
        let mut raw_notifications = self
            .notifications
            .iter()
            .map(|(notify, path)| crate::sys::PRJ_NOTIFICATION_MAPPING {
                NotificationBitMask: notify.into_raw(),
                NotificationRoot: path.as_ptr(),
            })
            .collect::<Vec<_>>();
        options.NotificationMappings = raw_notifications.as_mut_ptr();

        options
    }
}

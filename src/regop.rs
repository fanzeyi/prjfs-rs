use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};
use winapi::{
    shared::minwindef::{HKEY, PBYTE},
    um::winreg,
};

struct RegEntry {
    name: OsString,
    size: u64,
}

struct RegEntires {
    subkeys: Vec<RegEntry>,
    values: Vec<RegEntry>,
}

pub struct RegOps {
    keymap: HashMap<OsString, HKEY>,
}

impl RegOps {
    pub fn new() -> RegOps {
        let mut keymap: HashMap<OsString, HKEY> = HashMap::new();
        keymap.insert("HKEY_CLASSES_ROOT".into(), winreg::HKEY_CLASSES_ROOT);
        keymap.insert("HKEY_CURRENT_USER".into(), winreg::HKEY_CURRENT_USER);
        keymap.insert("HKEY_LOCAL_MACHINE".into(), winreg::HKEY_LOCAL_MACHINE);
        keymap.insert("HKEY_USERS".into(), winreg::HKEY_USERS);
        keymap.insert("HKEY_CURRENT_CONFIG".into(), winreg::HKEY_CURRENT_CONFIG);
        RegOps { keymap }
    }

    pub fn enumerate_key(&self, path: OsString) -> RegEntires {
        let path: &Path = path.as_ref();
        todo!()
    }

    pub fn read_value(&self, path: OsString, data: PBYTE, len: u32) -> bool {
        todo!()
    }

    pub fn does_key_exist(&self, path: OsString) -> bool {
        todo!()
    }

    pub fn does_value_exist(&self, path: OsString, size: i64) -> bool {
        todo!()
    }
}

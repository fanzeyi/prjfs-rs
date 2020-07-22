use log::warn;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Component, Path, PathBuf},
};
use winapi::{
    shared::{
        minwindef::{HKEY, PBYTE},
        winerror,
    },
    um::{winnt, winreg},
};

use crate::conv::WStrExt;

mod utils {
    use std::path::{Component, Path};

    pub fn is_virtualization_root(path: &Path) -> bool {
        if let Some(comp) = path.components().next() {
            // some component, must be \ to be root
            comp == Component::RootDir
        } else {
            // no components, still root
            true
        }
    }
}

#[derive(Default)]
pub struct RegEntry {
    name: OsString,
    size: u64,
}

#[derive(Default)]
pub struct RegEntires {
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
        let mut result = RegEntires::default();
        if utils::is_virtualization_root(path.as_ref()) {
            for (name, _entry) in &self.keymap {
                result.subkeys.push(RegEntry {
                    name: name.clone(),
                    size: 0,
                });
            }
        } else {
            // if let Some(subkey) = self.open_key_by_path(path.as_ref()) {
            //     result = self.enumerate_key();
            //     unsafe { winreg::RegCloseKey(subkey) };
            // }
        }

        result
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

    fn open_key_by_path(&self, path: &Path) -> Option<HKEY> {
        if path.components().count() == 1 {
            if let Some(hkey) = self.keymap.get(path.as_os_str()) {
                Some(*hkey)
            } else {
                warn!("open_key_by_path: root key [{:?}] doesn't exist", path);
                None
            }
        } else {
            let mut parts = path.components();
            let rootkey = match parts.next() {
                Some(Component::RootDir) => parts.next().map(|x| x.as_os_str()),
                Some(Component::Normal(part)) => Some(part),
                _ => None,
            }?;
            let subkey = parts.collect::<PathBuf>();
            let root = self.keymap.get(rootkey)?;
            let mut hkey = std::ptr::null_mut();

            let result = unsafe {
                winreg::RegOpenKeyExW(
                    *root,
                    subkey.as_os_str().to_wstr(),
                    0,
                    winnt::KEY_READ,
                    &mut hkey,
                )
            };

            if result != winerror::ERROR_SUCCESS as i32 {
                warn!(
                    "open_key_by_path: failed to open key [{:?}]: {}",
                    path, result
                );
                None
            } else {
                Some(hkey)
            }
        }
    }
}

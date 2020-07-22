#![feature(try_trait, slice_concat_trait)]
use anyhow::Result;
use std::path::PathBuf;
use winapi::um::projectedfslib as prjfs;

mod conv;
mod dirinfo;
mod guid;
mod provider;
mod regfs;
mod regop;

use crate::conv::WStrExt;
use crate::provider::{Provider, ProviderT};
use crate::regfs::RegFs;

fn main() -> Result<()> {
    env_logger::init();

    let path = PathBuf::from("./test");
    let mut mappings: [prjfs::PRJ_NOTIFICATION_MAPPING; 1] = [prjfs::PRJ_NOTIFICATION_MAPPING {
        NotificationBitMask: prjfs::PRJ_NOTIFY_FILE_OPENED
            | prjfs::PRJ_NOTIFY_PRE_RENAME
            | prjfs::PRJ_NOTIFY_PRE_DELETE,
        NotificationRoot: "".to_wstr().as_ptr(),
    }];
    let opts = prjfs::PRJ_STARTVIRTUALIZING_OPTIONS {
        NotificationMappings: mappings.as_mut_ptr(),
        NotificationMappingsCount: 1,
        ..Default::default()
    };
    let regfs: Box<dyn ProviderT> = Box::new(RegFs::new());
    let _provider = Provider::new(path, opts, regfs)?;

    loop {}
}

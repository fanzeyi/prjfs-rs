#![feature(try_trait, slice_concat_trait)]
use anyhow::Result;
use prjfs::conv::WStrExt;
use prjfs::provider::{Provider, ProviderT};
use std::path::PathBuf;

mod dirinfo;
mod regfs;
mod regop;

use crate::regfs::RegFs;

fn main() -> Result<()> {
    env_logger::init();

    let path = PathBuf::from("./test");
    let mut mappings: [prjfs::sys::PRJ_NOTIFICATION_MAPPING; 1] =
        [prjfs::sys::PRJ_NOTIFICATION_MAPPING {
            NotificationBitMask: prjfs::sys::PRJ_NOTIFY_FILE_OPENED
                | prjfs::sys::PRJ_NOTIFY_PRE_RENAME
                | prjfs::sys::PRJ_NOTIFY_PRE_DELETE,
            NotificationRoot: "".to_wstr().as_ptr(),
        }];
    let opts = prjfs::sys::PRJ_STARTVIRTUALIZING_OPTIONS {
        NotificationMappings: mappings.as_mut_ptr(),
        NotificationMappingsCount: 1,
        ..Default::default()
    };
    let regfs: Box<dyn ProviderT> = Box::new(RegFs::new());
    let _provider = Provider::new(path, opts, regfs)?;

    loop {}
}

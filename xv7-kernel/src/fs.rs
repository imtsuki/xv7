pub mod dev;
pub mod file;
pub mod vfs;

pub type Result<T> = core::result::Result<T, FsError>;

#[derive(Debug, Eq, PartialEq)]
pub enum FsError {
    NotSupported,
    FileError,
}

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::Mutex;
use vfs::Inode;

lazy_static! {
    pub static ref FILE_SYSTEM: Mutex<BTreeMap<String, Arc<dyn Inode>>> =
        Mutex::new(BTreeMap::new());
}

use crate::fs::{FsError, Result};
use core::any::Any;

pub trait Inode: Any + Sync + Send {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize>;
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize>;
    fn metadata(&self) -> Result<Metadata> {
        Err(FsError::NotSupported)
    }
    fn set_metadata(&self, _metadata: &Metadata) -> Result<()> {
        Err(FsError::NotSupported)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Metadata {
    /// Device ID
    pub dev: usize, // (major << 8) | minor
    /// Inode number
    pub inode: usize,
    /// Size in bytes
    ///
    /// SFS Note: for normal file size is the actuate file size
    /// for directory this is count of dirent.
    pub size: usize,
    /// Type of file
    pub type_: FileType,
    /// Permission
    pub mode: u16,
    /// Number of hard links
    pub nlinks: usize,
    /// Raw device id
    /// e.g. /dev/null: makedev(0x1, 0x3)
    pub rdev: usize, // (major << 8) | minor
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileType {
    File,
    Dir,
    SymLink,
    CharDevice,
    BlockDevice,
    NamedPipe,
    Socket,
}

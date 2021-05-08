use crate::fs::vfs::Inode;
use crate::fs::Result;
use alloc::sync::Arc;

use super::vfs::Metadata;

pub struct File {
    inode: Arc<dyn Inode>,
    offset: usize,
    readable: bool,
    writable: bool,
}

impl File {
    pub fn new(inode: Arc<dyn Inode>, readable: bool, writable: bool) -> Self {
        File {
            inode,
            offset: 0,
            readable,
            writable,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        assert!(self.readable);
        let len = self.inode.read_at(self.offset, buf)?;
        self.offset += len;
        Ok(len)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        assert!(self.writable);
        let len = self.inode.write_at(self.offset, buf)?;
        self.offset += len;
        Ok(len)
    }

    pub fn info(&self) -> Result<Metadata> {
        self.inode.metadata()
    }
}

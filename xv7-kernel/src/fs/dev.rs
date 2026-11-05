use super::vfs::Inode;
use crate::arch::interrupt::without_interrupts;

pub struct Console;

impl Inode for Console {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> super::Result<usize> {
        Ok(crate::device::tty::read(buf))
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> super::Result<usize> {
        without_interrupts(|| crate::device::console::CONSOLE_DRIVERS.lock().write(buf));
        Ok(buf.len())
    }
}

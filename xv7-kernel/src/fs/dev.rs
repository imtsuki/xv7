use super::vfs::Inode;
use crate::arch::interrupt::without_interrupts;
pub struct Console;

impl Inode for Console {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> super::Result<usize> {
        let mut cnt = 0;
        while let Ok(b) = crate::device::console::KEYBOARD_BUFFER.pop() {
            if cnt >= buf.len() {
                break;
            }
            buf[cnt] = b;
            cnt += 1;
        }
        Ok(cnt)
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> super::Result<usize> {
        without_interrupts(|| crate::device::console::CONSOLE_DRIVERS.lock().write(buf));
        Ok(buf.len())
    }
}

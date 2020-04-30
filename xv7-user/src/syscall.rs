use crate::arch::*;
use crate::number::*;

pub fn exit(code: isize) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as usize);
    }
    unreachable!()
}

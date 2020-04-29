#[macro_use]
mod macros;

pub mod allocator;
pub mod config;
mod console;
mod cpuid;
pub mod device;
mod gdt;
pub mod interrupt;
mod paging;
mod start;
pub mod syscall;

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            llvm_asm!("hlt");
        }
    }
}

#[macro_use]
mod macros;

pub mod allocator;
pub mod config;
mod console;
pub mod context;
mod cpuid;
pub mod device;
pub mod gdt;
pub mod interrupt;
pub mod paging;
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

pub mod apic;
pub mod config;
mod gdt;
pub mod interrupt;
mod paging;
mod pic;
mod start;

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            llvm_asm!("hlt");
        }
    }
}

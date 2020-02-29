pub mod config;
mod gdt;
pub mod interrupt;
mod paging;
mod start;

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

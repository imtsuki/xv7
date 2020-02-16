pub mod config;
mod gdt;
mod interrupt;
mod start;

#[inline(always)]
pub fn halt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

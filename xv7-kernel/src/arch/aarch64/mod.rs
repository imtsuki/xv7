mod start;

#[inline(always)]
pub fn halt_loop() -> ! {
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}

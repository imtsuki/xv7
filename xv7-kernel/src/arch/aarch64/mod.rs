mod start;

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}

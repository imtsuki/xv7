use crate::hlt_loop;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}

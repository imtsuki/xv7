use crate::hlt_loop;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    hlt_loop();
}

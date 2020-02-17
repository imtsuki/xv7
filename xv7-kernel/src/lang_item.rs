use crate::arch::idle;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    idle();
}

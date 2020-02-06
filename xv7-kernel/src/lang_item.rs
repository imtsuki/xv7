use crate::halt_loop;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    halt_loop();
}

#[no_mangle]
extern "C" fn _start() -> ! {
    crate::main();
}

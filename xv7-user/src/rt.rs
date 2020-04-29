#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    crate::syscall::exit(-1);
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    main();
    crate::syscall::exit(0);
}

extern "Rust" {
    fn main();
}

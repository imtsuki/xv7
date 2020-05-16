#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    crate::syscall::exit(-1);
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }
    // TODO: setup argc and argv
    let exit_code = main(0, 0 as *const *const u8);
    crate::syscall::exit(exit_code);
}

#[cfg(not(test))]
#[lang = "start"]
extern "C" fn lang_start<T: crate::process::Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    let exit_code = main().report();
    exit_code as isize
}

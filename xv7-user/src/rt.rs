#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    crate::syscall::exit(-1);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    unsafe extern "C" {
        fn main(argc: isize, argv: *const *const u8) -> isize;
    }
    // TODO: setup argc and argv
    let exit_code = unsafe { main(0, core::ptr::null()) };
    crate::syscall::exit(exit_code);
}

#[cfg(not(test))]
#[lang = "start"]
fn lang_start<T: crate::process::Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    let exit_code = main().report();
    exit_code as isize
}

use crate::halt_loop;
use bootinfo::KernelArgs;
use bootinfo::KERNEL_ARGS_MAGIC;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    halt_loop();
}

#[no_mangle]
extern "sysv64" fn _start(args: &'static KernelArgs) -> ! {
    assert_eq!(
        args.magic, KERNEL_ARGS_MAGIC,
        "KernelArgs magic number check failed"
    );
    crate::kmain(args);
}

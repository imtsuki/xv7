use crate::halt_loop;
use bootinfo::{KernelArgs, KernelEntryFn, KERNEL_ARGS_MAGIC};
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

#[doc(hidden)]
#[allow(unused)]
const KERNEL_ENTRY_SIGNATURE_TYPE_CHECK: KernelEntryFn = _start;

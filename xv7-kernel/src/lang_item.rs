use crate::halt_loop;
use boot::{KernelArgs, KernelEntryFn, KERNEL_ARGS_MAGIC};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    halt_loop();
}

#[no_mangle]
extern "sysv64" fn _start(args: &KernelArgs) -> ! {
    assert_eq!(BSS_ZERO_CHECK, 0);
    assert_eq!(DATA_NONZERO_CHECK, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(
        args.magic, KERNEL_ARGS_MAGIC,
        "KernelArgs magic number check failed"
    );
    crate::kmain(args);
}

#[used]
static BSS_ZERO_CHECK: u64 = 0;
#[used]
static DATA_NONZERO_CHECK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[doc(hidden)]
#[allow(unused)]
const KERNEL_ENTRY_SIGNATURE_TYPE_CHECK: KernelEntryFn = _start;

use super::*;
use crate::ansi::{CtrlSeq, EraseParam};
use boot::{KernelArgs, KernelEntryFn, KERNEL_ARGS_MAGIC};

#[used]
static BSS_ZERO_CHECK: u64 = 0;
#[used]
static DATA_NONZERO_CHECK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[doc(hidden)]
#[allow(unused)]
const KERNEL_ENTRY_SIGNATURE_TYPE_CHECK: KernelEntryFn = _start;

#[no_mangle]
extern "sysv64" fn _start(args: &KernelArgs) -> ! {
    assert_eq!(BSS_ZERO_CHECK, 0);
    assert_eq!(DATA_NONZERO_CHECK, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(
        args.magic, KERNEL_ARGS_MAGIC,
        "KernelArgs magic number check failed"
    );

    interrupt::without_interrupts(|| {
        print!(
            "{}{}{}",
            CtrlSeq::EraseDisplay(Some(EraseParam::Entire)),
            CtrlSeq::CursorPosition(None, None),
            CtrlSeq::SelectGraphicRendition(None),
        );

        dbg!(unsafe { x86_64::registers::model_specific::Msr::new(0x1b).read() });

        gdt::init();
        interrupt::init();
    });

    crate::kmain(args);
}

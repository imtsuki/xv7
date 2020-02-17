use super::*;
use crate::ansi::{CtrlSeq, EraseParam};
use boot::{BootArgs, KernelEntryFn, BOOT_ARGS_MAGIC};

#[used]
static BSS_ZERO_CHECK: u64 = 0;
#[used]
static DATA_NONZERO_CHECK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[doc(hidden)]
#[allow(unused)]
const KERNEL_ENTRY_SIGNATURE_TYPE_CHECK: KernelEntryFn = _start;

#[no_mangle]
extern "sysv64" fn _start(args: &BootArgs) -> ! {
    assert_eq!(BSS_ZERO_CHECK, 0);
    assert_eq!(DATA_NONZERO_CHECK, 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(
        args.magic, BOOT_ARGS_MAGIC,
        "BootArgs magic number check failed"
    );
    paging::disable_identity_mapping();

    interrupt::without_interrupts(|| {
        print!(
            "{}{}{}",
            CtrlSeq::EraseDisplay(Some(EraseParam::Entire)),
            CtrlSeq::CursorPosition(None, None),
            CtrlSeq::SelectGraphicRendition(None),
        );

        for descriptor in args.memory_map.clone().iter {
            dbg!(descriptor);
        }

        dbg!(args);

        dbg!(unsafe { x86_64::registers::model_specific::Msr::new(0x1b).read() });

        gdt::init();
        interrupt::init();
    });

    crate::kmain();
}

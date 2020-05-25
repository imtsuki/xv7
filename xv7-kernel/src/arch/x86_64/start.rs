use super::*;
use boot::{BootArgs, KernelEntryFn, BOOT_ARGS_MAGIC, MemoryMap};
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};

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
    assert_eq!(args.magic, BOOT_ARGS_MAGIC);

    crate::video::init(args);

    paging::disable_identity_mapping();

    paging::init_frame_allocator(args);

    // After this point, we can allocate memory
    crate::allocator::init_heap();

    // We wanna see outputs
    console::init();

    dbg!(args);

    print_memory_map(&args.memory_map);

    cpuid::init();

    gdt::init();

    interrupt::init();

    interrupt::controller::init();

    syscall::init();

    interrupt::enable();

    // crate::video::fun_things();

    crate::kmain();
}

fn print_memory_map(mmap:&MemoryMap) {
    println!("Mem phys map:");
    for descriptor in mmap.clone().iter {
        println!(
            "[mem {:#016x}-{:#016x} {:>8}] type {}",
            descriptor.phys_start,
            descriptor.phys_start + descriptor.page_count * Size4KiB::SIZE - 1,
            descriptor.page_count,
            descriptor.ty.0
        );
    }
}

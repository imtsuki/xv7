use boot::BootArgs;
use uefi::table::boot::MemoryType;
use x86_64::structures::paging::{page_table::PageTableEntry, PageTable};
use x86_64::PhysAddr;

pub fn disable_identity_mapping() {
    let page_table = unsafe { &mut *(0xFFFF_FFFF_FFFF_F000 as *mut PageTable) };

    for i in 0..256 {
        page_table[i] = PageTableEntry::new();
    }

    x86_64::instructions::tlb::flush_all();
}

pub fn init_frame_allocator(args: &BootArgs) {
    let mut allocator = crate::memory::FRAME_ALLOCATOR.lock();

    for descriptor in args.memory_map.clone().iter {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            dbg!(descriptor);
            allocator.insert_memory_region(
                PhysAddr::new(descriptor.phys_start),
                descriptor.page_count as usize,
            );
        }
    }

    allocator.print_statistics();
}

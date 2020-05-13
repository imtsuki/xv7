use crate::config::*;
use boot::BootArgs;
use uefi::table::boot::MemoryType;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{page_table::PageTableEntry, PageTable, PageTableFlags};
pub use x86_64::{PhysAddr, VirtAddr};

pub fn disable_identity_mapping() {
    let page_table = unsafe { active_page_table() };

    for i in 0..256 {
        page_table[i] = PageTableEntry::new();
    }

    x86_64::instructions::tlb::flush_all();
}

pub fn init_frame_allocator(args: &BootArgs) {
    let mut allocator = crate::memory::FRAME_ALLOCATOR.lock();

    for descriptor in args.memory_map.clone().iter {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            allocator.install_memory_region(
                PhysAddr::new(descriptor.phys_start),
                descriptor.page_count as usize,
            );
        }
    }
}

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = PAGE_OFFSET_BASE + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt as *mut _;

    &mut *page_table_ptr
}

/// HACK: x86_64 crate currently doesn't set bits in L4~L2 page table entries.
///
/// Can be removed once rust-osdev/x86_64#114 is merged.
pub fn override_user_accessible_bits() {
    let l4_page_table = unsafe { active_page_table() };
    override_walker(l4_page_table, 4);
}

fn override_walker(table: &mut PageTable, level: usize) {
    let top_index = if level == 4 { 256 } else { 512 };
    for i in 0..top_index {
        if !table[i].is_unused() {
            let mut flags = table[i].flags();
            flags |= PageTableFlags::USER_ACCESSIBLE;
            table[i].set_flags(flags);
            if level > 1 {
                match table[i].frame() {
                    Ok(frame) => {
                        let next_table = unsafe {
                            &mut *((frame.start_address().as_u64() + PAGE_OFFSET_BASE)
                                as *mut PageTable)
                        };
                        override_walker(next_table, level - 1);
                    }
                    Err(_) => (),
                }
            }
        }
    }
}

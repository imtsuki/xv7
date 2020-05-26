use crate::{config::*, memory::FrameAllocator};
use boot::BootArgs;
use uefi::table::boot::MemoryType;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};
pub use x86_64::{PhysAddr, VirtAddr};

pub fn disable_identity_mapping() {
    let page_table = unsafe { active_page_table() };

    for i in 0..256 {
        page_table[i].set_unused();
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

pub struct AddressSpace {
    pub cr3: PhysFrame,
}

impl AddressSpace {
    pub fn new() -> AddressSpace {
        let table_frame = crate::memory::FRAME_ALLOCATOR
            .lock()
            .allocate_frame()
            .unwrap();
        let table = unsafe {
            &mut *{ (PAGE_OFFSET_BASE + table_frame.start_address().as_u64()) as *mut PageTable }
        };
        let current_table = unsafe { active_page_table() };
        for i in 0..256 {
            table[i].set_unused();
        }
        for i in 256..512 {
            table[i] = current_table[i].clone();
        }
        table[511].set_frame(
            table_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
        );
        AddressSpace { cr3: table_frame }
    }

    pub unsafe fn page_table(&mut self) -> &'static mut PageTable {
        let phys = self.cr3.start_address();
        let virt = PAGE_OFFSET_BASE + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt as *mut _;

        &mut *page_table_ptr
    }
}

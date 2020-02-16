use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryType};
use x86_64::registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTable, PageTableFlags, PhysFrame, RecursivePageTable,
    Size2MiB, Size4KiB, UnusedPhysFrame,
};
use x86_64::{PhysAddr, VirtAddr};

/// UEFI allows us to introduce new memory types
/// in the 0x70000000..0xFFFFFFFF range.
pub const MEMORY_TYPE_KERNEL: u32 = 0x70000000;

/// This frame allocator marks frames as `MEMORY_TYPE_KERNEL`.
pub struct KernelFrameAllocator<'a>(&'a BootServices);

impl<'a> KernelFrameAllocator<'a> {
    pub fn new(services: &'a BootServices) -> Self {
        Self(services)
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for KernelFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        let phys_addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType(MEMORY_TYPE_KERNEL), 1)
            .expect_success("Failed to allocate physical frame");
        let phys_addr = PhysAddr::new(phys_addr);
        let phys_frame = PhysFrame::containing_address(phys_addr);
        Some(unsafe { UnusedPhysFrame::new(phys_frame) })
    }
}

/// Set up a basic recursive page table.
pub fn init_recursive(
    allocator: &mut impl FrameAllocator<Size4KiB>,
) -> RecursivePageTable<'static> {
    // First we do a copy for the level 4 table here, because the old table
    // has memory type `BOOT_SERVICES_DATA`. Level 3 ~ level 1 tables will
    // be discarded eventually so we can ignore them.
    let old_l4_table_addr = Cr3::read().0.start_address().as_u64();
    let l4_table_frame = allocator.allocate_frame().unwrap().frame();
    let l4_table_addr = l4_table_frame.start_address().as_u64();

    // Safety: newly allocated frame is guaranteed to be valid and unused
    unsafe {
        core::ptr::copy(
            old_l4_table_addr as *const u8,
            l4_table_addr as *mut u8,
            l4_table_frame.size() as usize,
        )
    };

    // Safety: same as above
    let l4_table = unsafe { &mut *(l4_table_addr as *mut PageTable) };

    // Recursive mapping
    l4_table[0b111_111_111].set_frame(
        l4_table_frame,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
    );

    // Enable all CPU extensions we need.
    unsafe {
        Cr4::update(|cr4| {
            cr4.insert(
                Cr4Flags::PAGE_SIZE_EXTENSION
                    | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
                    | Cr4Flags::PAGE_GLOBAL
                    | Cr4Flags::OSFXSR
                    | Cr4Flags::OSXSAVE,
            )
        })
    };

    // Switch to the new page table...
    unsafe { Cr3::write(l4_table_frame, Cr3Flags::empty()) };

    // And we have it!
    let l4_table = unsafe { &mut *(0xFFFF_FFFF_FFFF_F000 as *mut PageTable) };

    RecursivePageTable::new(l4_table).unwrap()
}

/// Map complete pyhsical memory to `offset`, which is `PAGE_OFFSET_BASE`.
pub fn map_physical_memory(
    offset: VirtAddr,
    max_addr: PhysAddr,
    page_table: &mut impl Mapper<Size2MiB>,
    allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let start_frame = PhysFrame::containing_address(PhysAddr::new(0));
    let end_frame = PhysFrame::containing_address(max_addr);
    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        let page = Page::containing_address(offset + frame.start_address().as_u64());
        let frame = unsafe { UnusedPhysFrame::new(frame) };
        page_table
            .map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                allocator,
            )
            .expect("Error occured while mapping complete pyhsical memory")
            .flush();
    }
}

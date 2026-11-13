use uefi::boot::{AllocateType, MemoryType};
use x86_64::registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    Size2MiB, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr, align_up};

/// Allocates `count` contiguous 4 KiB physical pages from UEFI (`LOADER_DATA`) and zeroes them.
pub fn alloc_zeroed_pages(count: usize) -> PhysAddr {
    let ptr = uefi::boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, count)
        .expect("Failed to allocate physical pages")
        .as_ptr();
    unsafe {
        core::ptr::write_bytes(ptr, 0, count * Size4KiB::SIZE as usize);
    }
    PhysAddr::new(ptr as u64)
}

struct BootFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        Some(PhysFrame::containing_address(alloc_zeroed_pages(1)))
    }
}

/// Offline 4-level page table builder backed by UEFI's identity mapping (`offset = 0`).
pub struct BootPageTable {
    p4_frame: PhysFrame,
    mapper: OffsetPageTable<'static>,
}

impl BootPageTable {
    pub fn new() -> Self {
        let p4_addr = alloc_zeroed_pages(1);
        let p4_frame = PhysFrame::containing_address(p4_addr);
        let p4_table = unsafe { &mut *(p4_addr.as_u64() as *mut PageTable) };
        let mapper = unsafe { OffsetPageTable::new(p4_table, VirtAddr::zero()) };
        Self { p4_frame, mapper }
    }

    pub fn map_4k(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags) {
        unsafe {
            self.mapper
                .map_to(page, frame, flags, &mut BootFrameAllocator)
                .expect("Failed to map 4 KiB page")
                .ignore();
        }
    }

    /// Maps `[0, max_addr]` to `offset` (`PAGE_OFFSET_BASE`) using 2 MiB huge pages.
    pub fn map_physical_memory(&mut self, offset: VirtAddr, max_addr: PhysAddr) {
        let start = PhysFrame::<Size2MiB>::containing_address(PhysAddr::zero());
        let end = PhysFrame::<Size2MiB>::containing_address(max_addr);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

        for frame in PhysFrame::range_inclusive(start, end) {
            let page = Page::containing_address(offset + frame.start_address().as_u64());
            unsafe {
                self.mapper
                    .map_to(page, frame, flags, &mut BootFrameAllocator)
                    .expect("Failed to map physical memory")
                    .ignore();
            }
        }
    }

    /// Allocates and maps a zeroed initial kernel stack under `stack_top`.
    pub fn map_stack(&mut self, stack_top: VirtAddr, size: usize) {
        let page_count = align_up(size as u64, Size4KiB::SIZE) / Size4KiB::SIZE;
        let top_page = Page::<Size4KiB>::containing_address(stack_top);
        let bottom_page = top_page - page_count;
        let phys_base = alloc_zeroed_pages(page_count as usize);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

        for (i, page) in Page::range(bottom_page, top_page).enumerate() {
            let frame = PhysFrame::containing_address(phys_base + i as u64 * Size4KiB::SIZE);
            self.map_4k(page, frame, flags);
        }
    }

    pub fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        self.mapper.translate_addr(addr)
    }

    /// Copies UEFI's lower-half identity mappings (`PML4[0..256]`), enables required
    /// CPU paging/SSE features, and loads this page table into `CR3`.
    ///
    /// # Safety
    /// Must be called *after* `exit_boot_services()`.
    pub unsafe fn activate(self) {
        let p4_frame = self.p4_frame;
        drop(self.mapper);

        let (uefi_l4_frame, _) = Cr3::read();
        unsafe {
            let uefi_l4 = &*(uefi_l4_frame.start_address().as_u64() as *const PageTable);
            let p4 = &mut *(p4_frame.start_address().as_u64() as *mut PageTable);
            for i in 0..256 {
                p4[i] = uefi_l4[i].clone();
            }

            Cr4::update(|cr4| {
                cr4.insert(
                    Cr4Flags::PAGE_SIZE_EXTENSION
                        | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
                        | Cr4Flags::PAGE_GLOBAL
                        | Cr4Flags::OSFXSR,
                )
            });
            Efer::update(|efer| efer.insert(EferFlags::NO_EXECUTE_ENABLE));
            Cr3::write(p4_frame, Cr3Flags::empty());
        }
    }
}

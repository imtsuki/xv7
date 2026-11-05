use crate::{
    config::*,
    memory::{FRAME_ALLOCATOR, FrameAllocator},
};
use boot::{BootArgs, MemoryType};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
pub use x86_64::{PhysAddr, VirtAddr};
use zeroize::Zeroize;

pub fn disable_identity_mapping() {
    let page_table = unsafe { active_page_table() };

    for i in 0..256 {
        page_table[i].set_unused();
    }

    x86_64::instructions::tlb::flush_all();
}

pub fn init_frame_allocator(args: &BootArgs) {
    let mut allocator = FRAME_ALLOCATOR.lock();
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

    unsafe { &mut *page_table_ptr }
}

/// Allocates a single 4 KiB physical frame and returns its kernel virtual address
/// in the direct physical mapping (`PAGE_OFFSET_BASE + phys`).
pub fn alloc_kernel_page() -> VirtAddr {
    let frame = FRAME_ALLOCATOR
        .lock()
        .allocate_frame()
        .expect("Failed to allocate kernel page");
    VirtAddr::new(PAGE_OFFSET_BASE + frame.start_address().as_u64())
}

pub struct AddressSpace {
    pub cr3: PhysFrame,
}

impl AddressSpace {
    pub fn new() -> AddressSpace {
        let table_frame = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
        let table = unsafe {
            &mut *((PAGE_OFFSET_BASE + table_frame.start_address().as_u64()) as *mut PageTable)
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

    #[inline]
    pub fn cr3_u64(&self) -> u64 {
        self.cr3.start_address().as_u64()
    }

    pub unsafe fn page_table(&mut self) -> &'static mut PageTable {
        let phys = self.cr3.start_address();
        let virt = PAGE_OFFSET_BASE + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt as *mut _;

        unsafe { &mut *page_table_ptr }
    }

    fn mapper(&mut self) -> OffsetPageTable<'static> {
        let page_table = unsafe { self.page_table() };
        unsafe { OffsetPageTable::new(page_table, VirtAddr::new(PAGE_OFFSET_BASE)) }
    }

    /// Allocates zeroed physical frames and maps them to `[vaddr, vaddr + len)` in this user address space.
    pub fn map_user_range(
        &mut self,
        vaddr: VirtAddr,
        len: usize,
        _writable: bool,
        executable: bool,
    ) {
        if len == 0 {
            return;
        }
        let start_addr = Page::<Size4KiB>::containing_address(vaddr)
            .start_address()
            .as_u64();
        let end_addr = Page::<Size4KiB>::containing_address(vaddr + (len as u64 - 1))
            .start_address()
            .as_u64();

        let mut flags =
            PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
        if !executable {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        let mut mapper = self.mapper();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();

        let mut addr = start_addr;
        loop {
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr));
            if mapper.translate_page(page).is_err() {
                let frame = frame_allocator
                    .allocate_frame()
                    .expect("Failed to allocate user frame");
                let kva = (PAGE_OFFSET_BASE + frame.start_address().as_u64()) as *mut u8;
                let slice =
                    unsafe { core::slice::from_raw_parts_mut(kva, Size4KiB::SIZE as usize) };
                slice.zeroize();

                unsafe {
                    let flusher = mapper
                        .map_to(page, frame, flags, &mut *frame_allocator)
                        .expect("Failed to map user page");
                    if Cr3::read().0 == self.cr3 {
                        flusher.flush();
                    } else {
                        flusher.ignore();
                    }
                }
            }
            if addr == end_addr {
                break;
            }
            addr += Size4KiB::SIZE;
        }
    }

    /// Copies `src` bytes into user virtual address `dst_vaddr` in this `AddressSpace`
    /// via the kernel direct physical mapping (safe even when `self` is not the active `cr3`).
    pub fn copy_to_user(&mut self, dst_vaddr: VirtAddr, src: &[u8]) {
        let mapper = self.mapper();
        let mut offset = 0usize;
        while offset < src.len() {
            let cur_vaddr = dst_vaddr + offset as u64;
            let phys = mapper
                .translate_addr(cur_vaddr)
                .expect("copy_to_user: unmapped user address");
            let page_offset = (cur_vaddr.as_u64() & 0xFFF) as usize;
            let chunk = (Size4KiB::SIZE as usize - page_offset).min(src.len() - offset);
            let dst_ptr = (PAGE_OFFSET_BASE + phys.as_u64()) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr().add(offset), dst_ptr, chunk);
            }
            offset += chunk;
        }
    }

    /// Writes a `u64` value to user virtual address `dst_vaddr` in this `AddressSpace`.
    pub fn write_user_u64(&mut self, dst_vaddr: VirtAddr, value: u64) {
        self.copy_to_user(dst_vaddr, &value.to_ne_bytes());
    }
}

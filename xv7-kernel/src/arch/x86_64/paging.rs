use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use boot::{BootArgs, MemoryType};
use spin::{Mutex, Once};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
pub use x86_64::{PhysAddr, VirtAddr};
use zeroize::Zeroize;

use crate::{
    config::*,
    memory::{FRAME_ALLOCATOR, FrameAllocator},
};

/// Kernel PML4 physical frame recorded during early boot.
static KERNEL_CR3: Once<PhysFrame> = Once::new();

/// Virtual address region for per-process kernel stacks (within PML4[416],
/// sharing the same Level-3 PDPT initialized by `init_heap` at `KERNEL_HEAP_BASE`).
const KSTACK_REGION_BASE: u64 = KERNEL_HEAP_BASE as u64 + 0x1000_0000;
const KSTACK_GUARD_SIZE: u64 = Size4KiB::SIZE;
pub const KSTACK_SIZE: u64 = 4 * Size4KiB::SIZE;
const KSTACK_SLOT_STRIDE: u64 = KSTACK_GUARD_SIZE + KSTACK_SIZE;

static NEXT_KSTACK_SLOT: AtomicU64 = AtomicU64::new(0);
static FREE_KSTACK_TOPS: Mutex<Vec<VirtAddr>> = Mutex::new(Vec::new());

pub fn disable_identity_mapping() {
    let (kernel_frame, _) = Cr3::read();
    KERNEL_CR3.call_once(|| kernel_frame);

    let page_table = unsafe { active_page_table() };
    for i in 0..256 {
        page_table[i].set_unused();
    }

    x86_64::instructions::tlb::flush_all();
}

pub fn activate_kernel_page_table() {
    let target = *KERNEL_CR3.get().expect("KERNEL_CR3 not initialized");
    let (current, flags) = Cr3::read();
    if current != target {
        unsafe {
            Cr3::write(target, flags);
        }
    }
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

unsafe fn page_table_from_frame(frame: PhysFrame) -> &'static mut PageTable {
    let virt = PAGE_OFFSET_BASE + frame.start_address().as_u64();
    unsafe { &mut *(virt as *mut PageTable) }
}

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    unsafe { page_table_from_frame(level_4_table_frame) }
}

/// A 16 KiB kernel stack preceded by an unmapped 4 KiB guard page.
/// Recycled automatically on `Drop`.
pub struct KernelStack {
    top: VirtAddr,
}

impl KernelStack {
    pub fn alloc() -> Self {
        if let Some(top) = FREE_KSTACK_TOPS.lock().pop() {
            return Self { top };
        }

        let slot = NEXT_KSTACK_SLOT.fetch_add(1, Ordering::Relaxed);
        let slot_base = KSTACK_REGION_BASE + slot * KSTACK_SLOT_STRIDE;
        let stack_bottom = slot_base + KSTACK_GUARD_SIZE;
        let stack_top = stack_bottom + KSTACK_SIZE;

        let kernel_frame = *KERNEL_CR3.get().expect("KERNEL_CR3 not initialized");
        let page_table = unsafe { page_table_from_frame(kernel_frame) };
        let mut mapper =
            unsafe { OffsetPageTable::new(page_table, VirtAddr::new(PAGE_OFFSET_BASE)) };
        let mut frame_allocator = FRAME_ALLOCATOR.lock();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        let mut addr = stack_bottom;
        while addr < stack_top {
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr));
            let frame = frame_allocator
                .allocate_frame()
                .expect("Failed to allocate kernel stack frame");
            unsafe {
                mapper
                    .map_to(page, frame, flags, &mut *frame_allocator)
                    .expect("Failed to map kernel stack page")
                    .flush();
            }
            addr += Size4KiB::SIZE;
        }

        Self {
            top: VirtAddr::new(stack_top),
        }
    }

    #[inline]
    pub fn top(&self) -> VirtAddr {
        self.top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        FREE_KSTACK_TOPS.lock().push(self.top);
    }
}

pub struct AddressSpace {
    pub cr3: PhysFrame,
}

impl AddressSpace {
    pub fn new() -> AddressSpace {
        let table_frame = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
        let table = unsafe { page_table_from_frame(table_frame) };
        let kernel_frame = *KERNEL_CR3.get().expect("KERNEL_CR3 not initialized");
        let kernel_table = unsafe { page_table_from_frame(kernel_frame) };

        for i in 0..256 {
            table[i].set_unused();
        }
        for i in 256..512 {
            table[i] = kernel_table[i].clone();
        }
        table[511].set_frame(
            table_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
        );
        AddressSpace { cr3: table_frame }
    }

    pub fn activate(&self) {
        let (current, flags) = Cr3::read();
        if current != self.cr3 {
            unsafe {
                Cr3::write(self.cr3, flags);
            }
        }
    }

    pub unsafe fn page_table(&mut self) -> &'static mut PageTable {
        unsafe { page_table_from_frame(self.cr3) }
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

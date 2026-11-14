use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::{Mutex, Once};
use x86_64::instructions::tlb;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};
pub use x86_64::{PhysAddr, VirtAddr};

use crate::config::{KERNEL_HEAP_BASE, PAGE_OFFSET_BASE};
use crate::memory::{MapPerm, OwnedFrame, PAGE_SIZE};

/// Kernel root PML4 physical address recorded during early boot.
static KERNEL_CR3: Once<PhysAddr> = Once::new();

/// Virtual address region for per-process kernel stacks (within `PML4[416]`).
const KSTACK_REGION_BASE: u64 = KERNEL_HEAP_BASE + 0x1000_0000;
const KSTACK_GUARD_SIZE: u64 = PAGE_SIZE as u64;
pub const KSTACK_SIZE: u64 = 4 * PAGE_SIZE as u64;
const KSTACK_SLOT_STRIDE: u64 = KSTACK_GUARD_SIZE + KSTACK_SIZE;

/// Kernel PML4 range (`0xFFFF_C000_0000_0000 .. 0xFFFF_F000_0000_0000`) pre-populated at boot
/// so dynamic kernel mappings (heap, stacks, MMIO) are immediately shared across all processes.
const KERNEL_DYN_PML4_START: usize = 384;
const KERNEL_DYN_PML4_END: usize = 480;

static NEXT_KSTACK_SLOT: AtomicU64 = AtomicU64::new(0);
static FREE_KSTACK_TOPS: Mutex<Vec<VirtAddr>> = Mutex::new(Vec::new());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    OutOfMemory,
    AlreadyMapped,
}

#[inline]
fn phys_to_table_ptr(paddr: PhysAddr) -> *mut PageTable {
    (PAGE_OFFSET_BASE + paddr.as_u64()) as *mut PageTable
}

#[inline]
fn kernel_cr3() -> PhysAddr {
    *KERNEL_CR3.get().expect("KERNEL_CR3 not initialized")
}

#[inline]
fn perm_to_leaf_flags(perm: MapPerm) -> PageTableFlags {
    let mut flags = PageTableFlags::PRESENT;
    if perm.contains(MapPerm::W) {
        flags |= PageTableFlags::WRITABLE;
    }
    if perm.contains(MapPerm::U) {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    } else {
        flags |= PageTableFlags::GLOBAL;
    }
    if !perm.contains(MapPerm::X) {
        flags |= PageTableFlags::NO_EXECUTE;
    }
    flags
}

unsafe fn next_table_mut(
    entry: &mut PageTableEntry,
    table_flags: PageTableFlags,
) -> Option<&mut PageTable> {
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let frame = OwnedFrame::alloc_zeroed()?;
        entry.set_addr(frame.into_raw(), table_flags);
    } else if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        return None;
    }
    Some(unsafe { &mut *phys_to_table_ptr(entry.addr()) })
}

unsafe fn map_4k_in_root(
    root_paddr: PhysAddr,
    vaddr: VirtAddr,
    frame: OwnedFrame,
    perm: MapPerm,
) -> Result<(), MapError> {
    debug_assert!(vaddr.is_aligned(PAGE_SIZE as u64));
    let is_user = u16::from(vaddr.p4_index()) < 256;
    let table_flags = if is_user {
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
    } else {
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE
    };

    unsafe {
        let p4 = &mut *phys_to_table_ptr(root_paddr);
        let p3 =
            next_table_mut(&mut p4[vaddr.p4_index()], table_flags).ok_or(MapError::OutOfMemory)?;
        let p2 =
            next_table_mut(&mut p3[vaddr.p3_index()], table_flags).ok_or(MapError::OutOfMemory)?;
        let p1 =
            next_table_mut(&mut p2[vaddr.p2_index()], table_flags).ok_or(MapError::OutOfMemory)?;

        let entry = &mut p1[vaddr.p1_index()];
        if entry.flags().contains(PageTableFlags::PRESENT) {
            return Err(MapError::AlreadyMapped);
        }

        entry.set_addr(frame.into_raw(), perm_to_leaf_flags(perm));

        if !is_user || Cr3::read().0.start_address() == root_paddr {
            tlb::flush(vaddr);
        }
    }
    Ok(())
}

/// Initializes kernel paging: records `KERNEL_CR3`, removes boot identity mappings (`0..256`),
/// and pre-populates shared higher-half kernel PML4 slots (`384..480`).
pub fn init() {
    let (kernel_frame, _) = Cr3::read();
    let root_paddr = kernel_frame.start_address();
    KERNEL_CR3.call_once(|| root_paddr);

    unsafe {
        let p4 = &mut *phys_to_table_ptr(root_paddr);
        for i in 0..256 {
            p4[i].set_unused();
        }
        for i in KERNEL_DYN_PML4_START..KERNEL_DYN_PML4_END {
            if !p4[i].flags().contains(PageTableFlags::PRESENT) {
                let l3 = OwnedFrame::alloc_zeroed().expect("Failed to pre-allocate kernel L3");
                p4[i].set_addr(
                    l3.into_raw(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                );
            }
        }
    }

    tlb::flush_all();
}

pub fn activate_kernel_page_table() {
    let target = PhysFrame::containing_address(kernel_cr3());
    let (current, flags) = Cr3::read();
    if current != target {
        unsafe {
            Cr3::write(target, flags);
        }
    }
}

/// Allocates zeroed frames and maps `[vaddr, vaddr + size)` into the shared kernel address space.
pub fn map_kernel_range(vaddr: VirtAddr, size: usize, perm: MapPerm) -> Result<(), MapError> {
    assert!(vaddr.is_aligned(PAGE_SIZE as u64));
    assert_eq!(size % PAGE_SIZE, 0);
    assert!(!perm.contains(MapPerm::U));

    let root = kernel_cr3();
    let mut offset = 0usize;
    while offset < size {
        let frame = OwnedFrame::alloc_zeroed().ok_or(MapError::OutOfMemory)?;
        unsafe {
            map_4k_in_root(root, vaddr + offset as u64, frame, perm)?;
        }
        offset += PAGE_SIZE;
    }
    Ok(())
}

/// Hardware x86_64 4-level page table backing a user [`crate::memory::AddressSpace`].
pub struct ArchPageTable {
    root_paddr: PhysAddr,
}

impl ArchPageTable {
    pub fn new_user() -> Option<Self> {
        let root_paddr = OwnedFrame::alloc_zeroed()?.into_raw();
        let kernel_root = kernel_cr3();
        unsafe {
            let dst_p4 = &mut *phys_to_table_ptr(root_paddr);
            let src_p4 = &*phys_to_table_ptr(kernel_root);
            for i in 256..512 {
                dst_p4[i] = src_p4[i].clone();
            }
        }
        Some(Self { root_paddr })
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        Cr3::read().0.start_address() == self.root_paddr
    }

    #[inline]
    pub fn activate(&self) {
        let target = PhysFrame::containing_address(self.root_paddr);
        let (current, _) = Cr3::read();
        if current != target {
            unsafe {
                Cr3::write(target, Cr3Flags::empty());
            }
        }
    }

    pub fn map_4k(
        &mut self,
        vaddr: VirtAddr,
        frame: OwnedFrame,
        perm: MapPerm,
    ) -> Result<(), MapError> {
        unsafe { map_4k_in_root(self.root_paddr, vaddr, frame, perm) }
    }

    pub fn unmap_4k(&mut self, vaddr: VirtAddr) -> Option<OwnedFrame> {
        debug_assert!(vaddr.is_aligned(PAGE_SIZE as u64));
        unsafe {
            let p4 = &*phys_to_table_ptr(self.root_paddr);
            let e4 = &p4[vaddr.p4_index()];
            if !e4.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }
            let p3 = &*phys_to_table_ptr(e4.addr());
            let e3 = &p3[vaddr.p3_index()];
            if !e3.flags().contains(PageTableFlags::PRESENT)
                || e3.flags().contains(PageTableFlags::HUGE_PAGE)
            {
                return None;
            }
            let p2 = &*phys_to_table_ptr(e3.addr());
            let e2 = &p2[vaddr.p2_index()];
            if !e2.flags().contains(PageTableFlags::PRESENT)
                || e2.flags().contains(PageTableFlags::HUGE_PAGE)
            {
                return None;
            }
            let p1 = &mut *phys_to_table_ptr(e2.addr());
            let e1 = &mut p1[vaddr.p1_index()];
            if !e1.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }

            let paddr = e1.addr();
            e1.set_unused();
            if self.is_active() {
                tlb::flush(vaddr);
            }
            Some(OwnedFrame::from_raw(paddr))
        }
    }

    /// Queries the exact physical address and effective hardware permissions of `vaddr`.
    pub fn query(&self, vaddr: VirtAddr) -> Option<(PhysAddr, MapPerm)> {
        let mut writable = true;
        let mut user = true;
        let mut executable = true;
        let mut accumulate = |flags: PageTableFlags| {
            writable &= flags.contains(PageTableFlags::WRITABLE);
            user &= flags.contains(PageTableFlags::USER_ACCESSIBLE);
            executable &= !flags.contains(PageTableFlags::NO_EXECUTE);
        };
        let build_perm = |w: bool, u: bool, x: bool| {
            let mut perm = MapPerm::R;
            if w {
                perm |= MapPerm::W;
            }
            if x {
                perm |= MapPerm::X;
            }
            if u {
                perm |= MapPerm::U;
            }
            perm
        };

        unsafe {
            let p4 = &*phys_to_table_ptr(self.root_paddr);
            let e4 = &p4[vaddr.p4_index()];
            if !e4.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }
            accumulate(e4.flags());

            let p3 = &*phys_to_table_ptr(e4.addr());
            let e3 = &p3[vaddr.p3_index()];
            if !e3.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }
            accumulate(e3.flags());
            if e3.flags().contains(PageTableFlags::HUGE_PAGE) {
                let paddr = e3.addr() + (vaddr.as_u64() & 0x3FFF_FFFF);
                return Some((paddr, build_perm(writable, user, executable)));
            }

            let p2 = &*phys_to_table_ptr(e3.addr());
            let e2 = &p2[vaddr.p2_index()];
            if !e2.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }
            accumulate(e2.flags());
            if e2.flags().contains(PageTableFlags::HUGE_PAGE) {
                let paddr = e2.addr() + (vaddr.as_u64() & 0x1F_FFFF);
                return Some((paddr, build_perm(writable, user, executable)));
            }

            let p1 = &*phys_to_table_ptr(e2.addr());
            let e1 = &p1[vaddr.p1_index()];
            if !e1.flags().contains(PageTableFlags::PRESENT) {
                return None;
            }
            accumulate(e1.flags());
            let paddr = e1.addr() + u64::from(vaddr.page_offset());
            Some((paddr, build_perm(writable, user, executable)))
        }
    }
}

impl Drop for ArchPageTable {
    fn drop(&mut self) {
        debug_assert!(!self.is_active(), "cannot drop active ArchPageTable");
        unsafe {
            let p4 = &mut *phys_to_table_ptr(self.root_paddr);
            // Only lower-half entries [0..256] belong to user space; [256..512] are shared kernel entries.
            for i4 in 0..256 {
                let e4 = &mut p4[i4];
                if !e4.flags().contains(PageTableFlags::PRESENT) {
                    continue;
                }
                let p3_paddr = e4.addr();
                let p3 = &mut *phys_to_table_ptr(p3_paddr);
                for i3 in 0..512 {
                    let e3 = &mut p3[i3];
                    if !e3.flags().contains(PageTableFlags::PRESENT)
                        || e3.flags().contains(PageTableFlags::HUGE_PAGE)
                    {
                        continue;
                    }
                    let p2_paddr = e3.addr();
                    let p2 = &mut *phys_to_table_ptr(p2_paddr);
                    for i2 in 0..512 {
                        let e2 = &mut p2[i2];
                        if !e2.flags().contains(PageTableFlags::PRESENT)
                            || e2.flags().contains(PageTableFlags::HUGE_PAGE)
                        {
                            continue;
                        }
                        let p1_paddr = e2.addr();
                        let p1 = &mut *phys_to_table_ptr(p1_paddr);
                        for i1 in 0..512 {
                            let e1 = &mut p1[i1];
                            if e1.flags().contains(PageTableFlags::PRESENT) {
                                let leaf_paddr = e1.addr();
                                e1.set_unused();
                                drop(OwnedFrame::from_raw(leaf_paddr));
                            }
                        }
                        e2.set_unused();
                        drop(OwnedFrame::from_raw(p1_paddr));
                    }
                    e3.set_unused();
                    drop(OwnedFrame::from_raw(p2_paddr));
                }
                e4.set_unused();
                drop(OwnedFrame::from_raw(p3_paddr));
            }
            drop(OwnedFrame::from_raw(self.root_paddr));
        }
    }
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

        map_kernel_range(
            VirtAddr::new(stack_bottom),
            KSTACK_SIZE as usize,
            MapPerm::R | MapPerm::W,
        )
        .expect("Failed to map kernel stack");

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

use uefi::boot::{AllocateType, MemoryType};
use x86_64::registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr, align_up};

pub const PAGE_SIZE_4K: u64 = 4096;
pub const PAGE_SIZE_2M: u64 = 2 * 1024 * 1024;

const INTERMEDIATE_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

/// Allocates `count` contiguous 4 KiB physical pages from UEFI (`LOADER_DATA`) and zeroes them.
pub fn alloc_zeroed_pages(count: usize) -> PhysAddr {
    let ptr = uefi::boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, count)
        .expect("Failed to allocate physical pages")
        .as_ptr();
    unsafe {
        core::ptr::write_bytes(ptr, 0, count * PAGE_SIZE_4K as usize);
    }
    PhysAddr::new(ptr as u64)
}

unsafe fn next_table_mut(entry: &mut PageTableEntry) -> &mut PageTable {
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let frame_addr = alloc_zeroed_pages(1);
        entry.set_addr(frame_addr, INTERMEDIATE_FLAGS);
    } else {
        assert!(
            !entry.flags().contains(PageTableFlags::HUGE_PAGE),
            "Huge page conflict while walking page table"
        );
    }
    unsafe { &mut *(entry.addr().as_u64() as *mut PageTable) }
}

/// Offline 4-level page table builder backed by UEFI's identity-mapped physical memory.
pub struct BootPageTable {
    p4_addr: PhysAddr,
}

impl BootPageTable {
    pub fn new() -> Self {
        Self {
            p4_addr: alloc_zeroed_pages(1),
        }
    }

    fn p4(&self) -> &PageTable {
        unsafe { &*(self.p4_addr.as_u64() as *const PageTable) }
    }

    fn p4_mut(&mut self) -> &mut PageTable {
        unsafe { &mut *(self.p4_addr.as_u64() as *mut PageTable) }
    }

    /// Maps a single 4 KiB page (`vaddr` -> `paddr`) with `flags`.
    pub fn map_4k(&mut self, vaddr: VirtAddr, paddr: PhysAddr, flags: PageTableFlags) {
        assert!(vaddr.is_aligned(PAGE_SIZE_4K));
        assert!(paddr.is_aligned(PAGE_SIZE_4K));
        assert!(!flags.contains(PageTableFlags::HUGE_PAGE));

        unsafe {
            let p4 = self.p4_mut();
            let p3 = next_table_mut(&mut p4[vaddr.p4_index()]);
            let p2 = next_table_mut(&mut p3[vaddr.p3_index()]);
            let p1 = next_table_mut(&mut p2[vaddr.p2_index()]);
            let entry = &mut p1[vaddr.p1_index()];
            assert!(
                !entry.flags().contains(PageTableFlags::PRESENT),
                "4 KiB page already mapped: {vaddr:?}"
            );
            entry.set_addr(paddr, flags | PageTableFlags::PRESENT);
        }
    }

    /// Maps a single 2 MiB huge page (`vaddr` -> `paddr`) with `flags`.
    pub fn map_2m(&mut self, vaddr: VirtAddr, paddr: PhysAddr, flags: PageTableFlags) {
        assert!(vaddr.is_aligned(PAGE_SIZE_2M));
        assert!(paddr.is_aligned(PAGE_SIZE_2M));

        unsafe {
            let p4 = self.p4_mut();
            let p3 = next_table_mut(&mut p4[vaddr.p4_index()]);
            let p2 = next_table_mut(&mut p3[vaddr.p3_index()]);
            let entry = &mut p2[vaddr.p2_index()];
            assert!(
                !entry.flags().contains(PageTableFlags::PRESENT),
                "2 MiB huge page already mapped: {vaddr:?}"
            );
            entry.set_addr(
                paddr,
                flags | PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE,
            );
        }
    }

    /// Maps `[0, max_addr]` to `offset` (`PAGE_OFFSET_BASE`) using 2 MiB huge pages.
    pub fn map_physical_memory(&mut self, offset: VirtAddr, max_addr: PhysAddr) {
        let end_excl = align_up(max_addr.as_u64().saturating_add(1), PAGE_SIZE_2M);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        let mut phys = 0u64;
        while phys < end_excl {
            self.map_2m(offset + phys, PhysAddr::new(phys), flags);
            phys += PAGE_SIZE_2M;
        }
    }

    /// Allocates and maps a zeroed initial kernel stack under `stack_top`.
    pub fn map_stack(&mut self, stack_top: VirtAddr, size: usize) {
        let page_count = (align_up(size as u64, PAGE_SIZE_4K) / PAGE_SIZE_4K) as usize;
        let stack_bottom = stack_top - (page_count as u64 * PAGE_SIZE_4K);
        let phys_base = alloc_zeroed_pages(page_count);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

        for i in 0..page_count as u64 {
            self.map_4k(
                stack_bottom + i * PAGE_SIZE_4K,
                phys_base + i * PAGE_SIZE_4K,
                flags,
            );
        }
    }

    /// Translates a mapped virtual address in this table to its physical address.
    pub fn translate(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        let p4 = self.p4();
        let e4 = &p4[vaddr.p4_index()];
        if !e4.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        let p3 = unsafe { &*(e4.addr().as_u64() as *const PageTable) };
        let e3 = &p3[vaddr.p3_index()];
        if !e3.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if e3.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Some(e3.addr() + (vaddr.as_u64() & 0x3FFF_FFFF));
        }
        let p2 = unsafe { &*(e3.addr().as_u64() as *const PageTable) };
        let e2 = &p2[vaddr.p2_index()];
        if !e2.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        if e2.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Some(e2.addr() + (vaddr.as_u64() & 0x1F_FFFF));
        }
        let p1 = unsafe { &*(e2.addr().as_u64() as *const PageTable) };
        let e1 = &p1[vaddr.p1_index()];
        if !e1.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }
        Some(e1.addr() + u64::from(vaddr.page_offset()))
    }

    /// Writes `src` into virtual address `dst_vaddr` via UEFI's identity mapping.
    pub fn write_bytes(&mut self, dst_vaddr: VirtAddr, src: &[u8]) {
        let mut offset = 0usize;
        while offset < src.len() {
            let cur_vaddr = dst_vaddr + offset as u64;
            let phys = self
                .translate(cur_vaddr)
                .expect("write_bytes: unmapped target virtual address");
            let page_offset = usize::from(cur_vaddr.page_offset());
            let chunk = (PAGE_SIZE_4K as usize - page_offset).min(src.len() - offset);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    src.as_ptr().add(offset),
                    phys.as_u64() as *mut u8,
                    chunk,
                );
            }
            offset += chunk;
        }
    }

    /// Writes a `u64` value into virtual address `dst_vaddr`.
    pub fn write_u64(&mut self, dst_vaddr: VirtAddr, value: u64) {
        self.write_bytes(dst_vaddr, &value.to_ne_bytes());
    }

    /// Copies UEFI's lower-half identity mappings (`PML4[0..256]`) so the trampoline
    /// instructions remain mapped across the `CR3` switch, enables required CPU
    /// paging/SSE features, and loads this page table into `CR3`.
    ///
    /// # Safety
    /// Must be called *after* `exit_boot_services()`.
    pub unsafe fn activate(mut self) {
        let (uefi_l4_frame, _) = Cr3::read();
        let uefi_l4 = unsafe { &*(uefi_l4_frame.start_address().as_u64() as *const PageTable) };
        let p4 = self.p4_mut();
        for i in 0..256 {
            p4[i] = uefi_l4[i].clone();
        }

        unsafe {
            Cr4::update(|cr4| {
                cr4.insert(
                    Cr4Flags::PAGE_SIZE_EXTENSION
                        | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
                        | Cr4Flags::PAGE_GLOBAL
                        | Cr4Flags::OSFXSR,
                )
            });
            Efer::update(|efer| efer.insert(EferFlags::NO_EXECUTE_ENABLE));
            Cr3::write(
                PhysFrame::containing_address(self.p4_addr),
                Cr3Flags::empty(),
            );
        }
    }
}

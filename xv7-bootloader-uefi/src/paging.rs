use crate::config::L4_PAGE_TABLE;
use x86_64::registers::control::{Cr3, Cr3Flags, Cr4, Cr4Flags};
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame};
use x86_64::PhysAddr;

/// Create a temporary page table for kernel's early booting process.
/// First 4GiB memory is mapped to both lower and higher half address space.
///
/// This page table is considered flawed but should be enough. After kernel
/// sets up frame allocator, it will immediately switch to a new page table.
///
/// TODO: Switch to `x86_64::structures::paging::Mapper` for better readability.
pub unsafe fn paging() {
    let mut base = L4_PAGE_TABLE;

    // L4 table is located at 0x70000
    let l4_table = &mut *(base as *mut PageTable);
    l4_table.zero();

    // Map to L3 table, to both lower-half and higher-half
    l4_table[0b000_000_000].set_addr(
        PhysAddr::new(base + 0x1000),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    l4_table[0b100_000_000].set_addr(
        PhysAddr::new(base + 0x1000),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    // Recursive mapping
    l4_table[0b111_111_111].set_addr(
        PhysAddr::new(base),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    // Move to L3 table
    base += 0x1000;

    // L3 table is located at 0x71000
    let l3_table = &mut *(base as *mut PageTable);
    l3_table.zero();

    base += 0x1000;

    // Map 0..4GiB to higher-half.
    // L2 tables are: 0x72000, 0x73000, 0x74000, 0x75000.
    for i in 0..4 {
        let l2_table_addr = base + 0x1000 + 0x1000 * i as u64;
        l3_table[i].set_addr(
            PhysAddr::new(l2_table_addr),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
        let l2_table = &mut *(l2_table_addr as *mut PageTable);
        // Map each 1GiB address space.
        for (offset, entry) in l2_table.iter_mut().enumerate() {
            entry.set_addr(
                PhysAddr::new(0x40000000 * i as u64 + 0x200000 * offset as u64),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
            );
        }
    }

    let mut cr4 = Cr4::read();

    // Enable all CPU extensions we need.
    cr4 |= Cr4Flags::PAGE_SIZE_EXTENSION
        | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
        | Cr4Flags::PAGE_GLOBAL
        | Cr4Flags::OSFXSR
        | Cr4Flags::OSXSAVE;

    Cr4::write(cr4);

    // Switch page table.
    Cr3::write(
        PhysFrame::containing_address(PhysAddr::new(L4_PAGE_TABLE)),
        Cr3Flags::empty(),
    );
}

use x86_64::structures::paging::{page_table::PageTableEntry, PageTable};

pub fn disable_identity_mapping() {
    let page_table = unsafe { &mut *(0xFFFF_FFFF_FFFF_F000 as *mut PageTable) };

    for i in 0..256 {
        page_table[i] = PageTableEntry::new();
    }

    x86_64::instructions::tlb::flush_all();
}

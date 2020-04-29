use lazy_static::lazy_static;
use x86_64::instructions::segmentation::{load_ds, load_es, load_gs, load_ss, set_cs};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::*;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code = {
            let flags = DescriptorFlags::USER_SEGMENT
                | DescriptorFlags::PRESENT
                | DescriptorFlags::EXECUTABLE
                | DescriptorFlags::LONG_MODE;
            Descriptor::UserSegment(flags.bits())
        };

        let kernel_data = {
            let flags = DescriptorFlags::USER_SEGMENT
                | DescriptorFlags::PRESENT
                | DescriptorFlags::WRITABLE;
            Descriptor::UserSegment(flags.bits())
        };

        let user_code = {
            let flags = DescriptorFlags::USER_SEGMENT
                | DescriptorFlags::PRESENT
                | DescriptorFlags::EXECUTABLE
                | DescriptorFlags::LONG_MODE
                | DescriptorFlags::DPL_RING_3;
            Descriptor::UserSegment(flags.bits())
        };

        let user_data = {
            let flags = DescriptorFlags::USER_SEGMENT
                | DescriptorFlags::PRESENT
                | DescriptorFlags::WRITABLE
                | DescriptorFlags::DPL_RING_3;
            Descriptor::UserSegment(flags.bits())
        };

        // The order is required.
        let kernel_code_selector = gdt.add_entry(kernel_code);

        let kernel_data_selector = gdt.add_entry(kernel_data);

        let mut user_data_selector = gdt.add_entry(user_data);
        user_data_selector.set_rpl(PrivilegeLevel::Ring3);

        let mut user_code_selector = gdt.add_entry(user_code);
        user_code_selector.set_rpl(PrivilegeLevel::Ring3);

        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                user_code_selector,
                user_data_selector,
                tss_selector,
            },
        )
    };
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };
        tss
    };
}

pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        load_ss(GDT.1.kernel_data_selector);
        load_ds(GDT.1.kernel_data_selector);
        load_es(GDT.1.kernel_data_selector);
        load_gs(GDT.1.kernel_data_selector);

        set_cs(GDT.1.kernel_code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

use core::cell::SyncUnsafeCell;
use lazy_static::lazy_static;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, DS, ES, GS, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::*;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

static TSS: SyncUnsafeCell<TaskStateSegment> = SyncUnsafeCell::new(TaskStateSegment::new());

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

        // Order required by `STAR` MSR (`sysret`).
        let kernel_code_selector = gdt.append(kernel_code);
        let kernel_data_selector = gdt.append(kernel_data);

        let mut user_data_selector = gdt.append(user_data);
        user_data_selector.set_rpl(PrivilegeLevel::Ring3);

        let mut user_code_selector = gdt.append(user_code);
        user_code_selector.set_rpl(PrivilegeLevel::Ring3);

        let tss = unsafe {
            let tss = &mut *TSS.get();
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096;
                static STACK: SyncUnsafeCell<[u8; STACK_SIZE]> =
                    SyncUnsafeCell::new([0; STACK_SIZE]);

                let stack_start = VirtAddr::from_ptr(STACK.get());
                stack_start + STACK_SIZE as u64
            };
            tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096;
                static STACK: SyncUnsafeCell<[u8; STACK_SIZE]> =
                    SyncUnsafeCell::new([0; STACK_SIZE]);

                let stack_start = VirtAddr::from_ptr(STACK.get());
                stack_start + STACK_SIZE as u64
            };
            &*TSS.get()
        };

        let tss_selector = gdt.append(Descriptor::tss_segment(tss));

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
        SS::set_reg(GDT.1.kernel_data_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        GS::set_reg(GDT.1.kernel_data_selector);

        CS::set_reg(GDT.1.kernel_code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

/// Updates `TSS.privilege_stack_table[0]` (`RSP0`) for hardware Ring 3 -> Ring 0 transitions.
pub fn set_tss_rsp0(kstack_top: VirtAddr) {
    unsafe {
        (*TSS.get()).privilege_stack_table[0] = kstack_top;
    }
}

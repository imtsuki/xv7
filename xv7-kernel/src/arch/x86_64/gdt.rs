use core::cell::SyncUnsafeCell;

use x86_64::instructions::segmentation::{CS, DS, ES, GS, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

/// Segment selectors matching the `IA32_STAR` MSR layout required by `syscall`/`sysret`.
pub const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
pub const KERNEL_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
pub const USER_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring3);
pub const USER_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring3);
pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring0);

const IST_STACK_SIZE: usize = 4096;

#[repr(C, align(16))]
struct IstStack([u8; IST_STACK_SIZE]);

static DOUBLE_FAULT_STACK: SyncUnsafeCell<IstStack> =
    SyncUnsafeCell::new(IstStack([0; IST_STACK_SIZE]));
static PAGE_FAULT_STACK: SyncUnsafeCell<IstStack> =
    SyncUnsafeCell::new(IstStack([0; IST_STACK_SIZE]));

static TSS: SyncUnsafeCell<TaskStateSegment> = SyncUnsafeCell::new(TaskStateSegment::new());

static GDT: SyncUnsafeCell<GlobalDescriptorTable> = SyncUnsafeCell::new(const {
    let mut gdt = GlobalDescriptorTable::new();
    assert!(gdt.append(Descriptor::kernel_code_segment()).0 == KERNEL_CODE_SELECTOR.0);
    assert!(gdt.append(Descriptor::kernel_data_segment()).0 == KERNEL_DATA_SELECTOR.0);
    assert!(gdt.append(Descriptor::user_data_segment()).0 == USER_DATA_SELECTOR.0);
    assert!(gdt.append(Descriptor::user_code_segment()).0 == USER_CODE_SELECTOR.0);
    gdt
});

pub fn init() {
    unsafe {
        let tss = &mut *TSS.get();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
            VirtAddr::from_ptr(DOUBLE_FAULT_STACK.get()) + IST_STACK_SIZE as u64;
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] =
            VirtAddr::from_ptr(PAGE_FAULT_STACK.get()) + IST_STACK_SIZE as u64;

        let gdt = &mut *GDT.get();
        let tss_sel = gdt.append(Descriptor::tss_segment(&*TSS.get()));
        assert_eq!(tss_sel, TSS_SELECTOR);

        (&*GDT.get()).load();

        SS::set_reg(KERNEL_DATA_SELECTOR);
        DS::set_reg(KERNEL_DATA_SELECTOR);
        ES::set_reg(KERNEL_DATA_SELECTOR);
        GS::set_reg(KERNEL_DATA_SELECTOR);
        CS::set_reg(KERNEL_CODE_SELECTOR);
        load_tss(TSS_SELECTOR);
    }
}

/// Updates `TSS.privilege_stack_table[0]` (`RSP0`) for hardware Ring 3 -> Ring 0 transitions.
pub fn set_tss_rsp0(kstack_top: VirtAddr) {
    unsafe {
        (*TSS.get()).privilege_stack_table[0] = kstack_top;
    }
}

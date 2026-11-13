use core::cell::SyncUnsafeCell;

use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const IST_STACK_SIZE: usize = 16 * 1024;

#[derive(Clone, Copy)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
}

/// Builds the base GDT in `IA32_STAR` order (`kernel_code`, `kernel_data`, `user_data`, `user_code`).
const fn base_gdt() -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());
    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());
    (
        gdt,
        Selectors {
            kernel_code,
            kernel_data,
            user_data,
            user_code,
        },
    )
}

pub const SELECTORS: Selectors = base_gdt().1;

#[repr(C, align(16))]
struct IstStack([u8; IST_STACK_SIZE]);

struct CpuGdt {
    table: GlobalDescriptorTable,
    tss: TaskStateSegment,
    df_stack: IstStack,
}

impl CpuGdt {
    const fn new() -> Self {
        Self {
            table: base_gdt().0,
            tss: TaskStateSegment::new(),
            df_stack: IstStack([0; IST_STACK_SIZE]),
        }
    }
}

/// TODO: Make this an array indexed by APIC ID when SMP is enabled.
static CPU_GDT: SyncUnsafeCell<CpuGdt> = SyncUnsafeCell::new(CpuGdt::new());

pub fn init() {
    unsafe {
        let cpu = CPU_GDT.get();
        let df_stack_top = VirtAddr::from_ptr(&raw const (*cpu).df_stack) + IST_STACK_SIZE as u64;
        (*cpu).tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = df_stack_top;

        let tss_sel = (*cpu)
            .table
            .append(Descriptor::tss_segment_unchecked(&raw const (*cpu).tss));
        (*cpu).table.load_unsafe();

        CS::set_reg(SELECTORS.kernel_code);
        SS::set_reg(SELECTORS.kernel_data);
        DS::set_reg(SegmentSelector::NULL);
        ES::set_reg(SegmentSelector::NULL);
        FS::set_reg(SegmentSelector::NULL);
        GS::set_reg(SegmentSelector::NULL);
        load_tss(tss_sel);
    }
}

/// Updates `TSS.privilege_stack_table[0]` (`RSP0`) for hardware Ring 3 -> Ring 0 transitions.
pub fn set_tss_rsp0(kstack_top: VirtAddr) {
    unsafe {
        (*CPU_GDT.get()).tss.privilege_stack_table[0] = kstack_top;
    }
}

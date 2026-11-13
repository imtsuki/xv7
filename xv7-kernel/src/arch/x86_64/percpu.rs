use core::cell::SyncUnsafeCell;

use x86_64::VirtAddr;
use x86_64::registers::model_specific::KernelGsBase;

use super::gdt;

/// Per-CPU state accessed via `%gs` on `syscall` entry (after `swapgs`).
#[repr(C)]
pub struct PerCpu {
    /// Top of the currently running process's kernel stack.
    pub kernel_rsp: u64,
    /// Scratch slot used by `syscall_entry` to stash user `%rsp` while switching stacks.
    pub user_rsp_scratch: u64,
}

impl PerCpu {
    const fn new() -> Self {
        Self {
            kernel_rsp: 0,
            user_rsp_scratch: 0,
        }
    }
}

/// TODO: Make this an array indexed by APIC ID when SMP is enabled.
static PER_CPU: SyncUnsafeCell<PerCpu> = SyncUnsafeCell::new(PerCpu::new());

pub fn init() {
    KernelGsBase::write(VirtAddr::from_ptr(PER_CPU.get()));
}

/// Registers `kstack_top` as the Ring 3 -> Ring 0 entry stack for both
/// `syscall` (`PerCpu.kernel_rsp`) and hardware interrupts/exceptions (`TSS.rsp0`).
pub fn set_kernel_stack(kstack_top: VirtAddr) {
    unsafe {
        (*PER_CPU.get()).kernel_rsp = kstack_top.as_u64();
    }
    gdt::set_tss_rsp0(kstack_top);
}

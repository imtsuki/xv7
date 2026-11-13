use core::mem::{offset_of, size_of};

use x86_64::VirtAddr;
use x86_64::registers::rflags::RFlags;

use super::gdt;

const USER_RFLAGS: u64 = RFlags::INTERRUPT_FLAG.bits() | 0x2;

/// Complete user-mode register state saved at the top of a process's kernel stack.
///
/// Shared uniformly across initial Ring 3 entry, `syscall` entry/exit, and future
/// `fork` / `exec` / interrupt trap handling.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct TrapFrame {
    // Scratch / syscall argument registers
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub rax: u64,

    // Callee-saved registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,

    // Hardware `iretq` frame
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

const _: () = {
    assert!(offset_of!(TrapFrame, rcx) == 0x00);
    assert!(offset_of!(TrapFrame, rax) == 0x40);
    assert!(offset_of!(TrapFrame, r15) == 0x48);
    assert!(offset_of!(TrapFrame, rbx) == 0x70);
    assert!(offset_of!(TrapFrame, rip) == 0x78);
    assert!(offset_of!(TrapFrame, ss) == 0x98);
    assert!(size_of::<TrapFrame>() == 0xA0);
};

impl TrapFrame {
    /// Resets the frame to cleanly enter Ring 3 at `(entry, user_sp)` with all
    /// general-purpose registers zeroed and interrupts enabled in user mode.
    pub fn init_user(&mut self, entry: VirtAddr, user_sp: VirtAddr) {
        *self = Self {
            rip: entry.as_u64(),
            cs: u64::from(gdt::USER_CODE_SELECTOR.0),
            rflags: USER_RFLAGS,
            rsp: user_sp.as_u64(),
            ss: u64::from(gdt::USER_DATA_SELECTOR.0),
            ..Self::default()
        };
    }
}

/// Unified Ring 0 -> Ring 3 exit trampoline.
///
/// Expects `%rsp` to point to a `TrapFrame` at `kstack_top - size_of::<TrapFrame>()`
/// with interrupts disabled (`IF = 0`). Pops all general-purpose registers and
/// executes `iretq` into user space.
#[unsafe(naked)]
pub unsafe extern "C" fn trap_return() -> ! {
    core::arch::naked_asm!(
        "pop rcx",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop r8",
        "pop r9",
        "pop r10",
        "pop r11",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "iretq",
    );
}

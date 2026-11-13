use core::mem::{offset_of, size_of};

use super::trapframe::{TrapFrame, trap_return};
use crate::paging::VirtAddr;

/// Pure kernel-mode coroutine context (callee-saved registers + `%rsp`).
///
/// Address space (`CR3`), privilege entry stack (`PerCpu` / `TSS.rsp0`), and
/// interrupt masking (`IF == 0` across `switch`) are managed orthogonally by
/// the scheduler and `TrapFrame`.
#[derive(Debug, Default)]
#[repr(C)]
pub struct Context {
    pub rsp: usize,
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            rsp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
        }
    }

    /// Initializes a kernel context at `kstack_top` that reserves a `TrapFrame`
    /// at the top of the stack and returns directly into `trap_return` when first scheduled.
    pub fn init_user(kstack_top: VirtAddr) -> Self {
        let tf_addr = kstack_top - size_of::<TrapFrame>() as u64;
        let ret_slot = tf_addr - size_of::<usize>() as u64;
        unsafe {
            tf_addr.as_mut_ptr::<TrapFrame>().write(TrapFrame::default());
            ret_slot
                .as_mut_ptr::<usize>()
                .write(trap_return as *const () as usize);
        }
        Self {
            rsp: ret_slot.as_u64() as usize,
            ..Self::new()
        }
    }

    #[unsafe(naked)]
    pub unsafe extern "C" fn switch(from: *mut Context, to: *const Context) {
        core::arch::naked_asm!(
            "mov [rdi + {r15}], r15",
            "mov [rdi + {r14}], r14",
            "mov [rdi + {r13}], r13",
            "mov [rdi + {r12}], r12",
            "mov [rdi + {rbp}], rbp",
            "mov [rdi + {rbx}], rbx",
            "mov [rdi + {rsp}], rsp",

            "mov r15, [rsi + {r15}]",
            "mov r14, [rsi + {r14}]",
            "mov r13, [rsi + {r13}]",
            "mov r12, [rsi + {r12}]",
            "mov rbp, [rsi + {rbp}]",
            "mov rbx, [rsi + {rbx}]",
            "mov rsp, [rsi + {rsp}]",
            "ret",
            rsp = const offset_of!(Context, rsp),
            r15 = const offset_of!(Context, r15),
            r14 = const offset_of!(Context, r14),
            r13 = const offset_of!(Context, r13),
            r12 = const offset_of!(Context, r12),
            rbp = const offset_of!(Context, rbp),
            rbx = const offset_of!(Context, rbx),
        );
    }
}

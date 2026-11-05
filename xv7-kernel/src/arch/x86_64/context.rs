use core::mem::size_of;

use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::InterruptStackFrameValue;

use super::gdt::GDT;
use crate::paging::VirtAddr;

const USER_RFLAGS: u64 = RFlags::INTERRUPT_FLAG.bits() | 0x2;

#[derive(Debug)]
#[repr(C)]
pub struct Context {
    pub cr3: usize,
    pub rsp: usize,
    pub rflags: usize,
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

impl Context {
    pub const fn new() -> Context {
        Context {
            cr3: 0,
            rflags: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: 0,
            rsp: 0,
        }
    }

    /// Initializes a new user-process kernel stack at `kstack_top` with a reserved
    /// `InterruptStackFrameValue` and `interrupt_return` (`iretq`) trampoline.
    pub fn init_user(kstack_top: VirtAddr, cr3: u64) -> Context {
        let stack_pointer = kstack_top - size_of::<InterruptStackFrameValue>() as u64 - 8u64;
        unsafe {
            stack_pointer
                .as_mut_ptr::<u64>()
                .write(interrupt_return as *const u8 as u64);
        }
        Context {
            cr3: cr3 as usize,
            rsp: stack_pointer.as_u64() as usize,
            rflags: USER_RFLAGS as usize,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
        }
    }

    /// Populates the `InterruptStackFrameValue` at the top of `kstack_top` so that
    /// `interrupt_return` (`iretq`) transitions to Ring 3 at `(instruction_pointer, stack_pointer)`.
    pub fn set_user_return_frame(
        kstack_top: VirtAddr,
        instruction_pointer: VirtAddr,
        stack_pointer: VirtAddr,
    ) {
        let frame_addr = kstack_top - size_of::<InterruptStackFrameValue>() as u64;
        let frame = unsafe { &mut *frame_addr.as_mut_ptr::<InterruptStackFrameValue>() };
        frame.instruction_pointer = instruction_pointer;
        frame.stack_pointer = stack_pointer;
        frame.code_segment = GDT.1.user_code_selector;
        frame.stack_segment = GDT.1.user_data_selector;
        frame.cpu_flags = RFlags::from_bits_retain(USER_RFLAGS);
    }

    #[unsafe(naked)]
    pub unsafe extern "C" fn switch_to(prev: *mut Context, next: *const Context) {
        core::arch::naked_asm!(
            "
            mov rcx, cr3
            mov [rdi], rcx
            mov rax, [rsi]
            cmp rax, rcx
    
            je 2f
            mov cr3, rax
    
            2:
            pushfq
            pop QWORD PTR [rdi + 0x10]

            push QWORD PTR [rsi + 0x10]
            popfq

            mov [rdi + 0x18], r15
            mov r15, [rsi + 0x18]
    
            mov [rdi + 0x20], r14
            mov r14, [rsi + 0x20]
    
            mov [rdi + 0x28], r13
            mov r13, [rsi + 0x28]
    
            mov [rdi + 0x30], r12
            mov r12, [rsi + 0x30]
    
            mov [rdi + 0x38], rbp
            mov rbp, [rsi + 0x38]
    
            mov [rdi + 0x40], rbx
            mov rbx, [rsi + 0x40]
    
            mov [rdi + 0x08], rsp
            mov rsp, [rsi + 0x08]

            ret
            ",
        );
    }
}

#[unsafe(naked)]
unsafe extern "C" fn interrupt_return() {
    core::arch::naked_asm!("iretq");
}

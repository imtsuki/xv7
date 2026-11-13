use core::mem::offset_of;

use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::gdt::SegmentSelector;

use super::gdt;
use super::percpu::PerCpu;
use super::trapframe::{TrapFrame, trap_return};

#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        "swapgs",
        "mov QWORD PTR gs:[{u_rsp}], rsp",
        "mov rsp, QWORD PTR gs:[{k_rsp}]",
        // Build hardware `iretq` portion of `TrapFrame`
        "push {u_ss}",
        "push QWORD PTR gs:[{u_rsp}]",
        "mov QWORD PTR gs:[{u_rsp}], 0",
        "push r11",
        "push {u_cs}",
        "push rcx",
        "swapgs",
        // Save callee-saved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // Save scratch / syscall argument registers
        "push rax",
        "push r11",
        "push r10",
        "push r9",
        "push r8",
        "push rdi",
        "push rsi",
        "push rdx",
        "push rcx",
        // Invoke `syscall_inner(&mut TrapFrame)` (`rsp` is 16-byte aligned)
        "mov rdi, rsp",
        "call {syscall_inner}",
        "jmp {trap_return}",
        k_rsp = const offset_of!(PerCpu, kernel_rsp),
        u_rsp = const offset_of!(PerCpu, user_rsp_scratch),
        u_cs = const gdt::USER_CODE_SELECTOR as usize,
        u_ss = const gdt::USER_DATA_SELECTOR as usize,
        syscall_inner = sym syscall_inner,
        trap_return = sym trap_return,
    );
}

extern "C" fn syscall_inner(tf: &mut TrapFrame) {
    tf.rax = crate::syscall::syscall(
        tf.rax as usize,
        tf.rdi as usize,
        tf.rsi as usize,
        tf.rdx as usize,
        tf.r10 as usize,
        tf.r8 as usize,
    ) as u64;
}

pub fn init() {
    Star::write(
        SegmentSelector(gdt::USER_CODE_SELECTOR),
        SegmentSelector(gdt::USER_DATA_SELECTOR),
        SegmentSelector(gdt::KERNEL_CODE_SELECTOR),
        SegmentSelector(gdt::KERNEL_DATA_SELECTOR),
    )
    .unwrap();

    LStar::write(VirtAddr::from_ptr(syscall_entry as *const u8));

    let mask = RFlags::TRAP_FLAG
        | RFlags::DIRECTION_FLAG
        | RFlags::INTERRUPT_FLAG
        | RFlags::IOPL_HIGH
        | RFlags::IOPL_LOW
        | RFlags::ALIGNMENT_CHECK
        | RFlags::NESTED_TASK;

    SFMask::write(mask);

    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }
}

#[repr(packed)]
pub struct ScratchRegisters {
    pub rcx: usize,
    pub rdx: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub rax: usize,
}

#[repr(packed)]
pub struct PreservedRegisters {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

#[repr(packed)]
pub struct IretRegisters {
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}

#[repr(packed)]
pub struct SyscallStackFrame {
    pub scratch: ScratchRegisters,
    pub preserved: PreservedRegisters,
    pub iret: IretRegisters,
}

macro_rules! scratch_push {
    () => (llvm_asm!(
    "
        pushq %rax
        pushq %r11
        pushq %r10
        pushq %r9
        pushq %r8
        pushq %rdi
        pushq %rsi
        pushq %rdx
        pushq %rcx
    " : : : : "volatile"
    ));
}

macro_rules! scratch_pop {
    () => (llvm_asm!(
    "
        popq %rcx
        popq %rdx
        popq %rsi
        popq %rdi
        popq %r8
        popq %r9
        popq %r10
        popq %r11
        popq %rax
    " : : : : "volatile"
    ));
}

macro_rules! preserved_push {
    () => (llvm_asm!(
    "
        pushq %rbx
        pushq %rbp
        pushq %r12
        pushq %r13
        pushq %r14
        pushq %r15
    " : : : : "volatile"
    ));
}

macro_rules! preserved_pop {
    () => (llvm_asm!(
    "
        popq %r15
        popq %r14
        popq %r13
        popq %r12
        popq %rbp
        popq %rbx
    " : : : : "volatile"
    ));
}

macro_rules! interrupt_return {
    () => (llvm_asm!(
        "iretq": : : : "volatile"
    ));
}

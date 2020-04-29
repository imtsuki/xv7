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

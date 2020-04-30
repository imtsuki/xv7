pub unsafe fn syscall0(mut a: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

pub unsafe fn syscall1(mut a: usize, b: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a), "{rdi}"(b)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

pub unsafe fn syscall2(mut a: usize, b: usize, c: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a), "{rdi}"(b), "{rsi}"(c)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

pub unsafe fn syscall3(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a), "{rdi}"(b), "{rsi}"(c), "{rdx}"(d)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

pub unsafe fn syscall4(mut a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a), "{rdi}"(b), "{rsi}"(c), "{rdx}"(d), "{r10}"(e)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

pub unsafe fn syscall5(mut a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    llvm_asm!(
    "syscall"
    : "={rax}"(a)
    : "{rax}"(a), "{rdi}"(b), "{rsi}"(c), "{rdx}"(d), "{r10}"(e), "{r8}"(f)
    : "rcx", "r11", "memory"
    : "volatile"
    );
    a
}

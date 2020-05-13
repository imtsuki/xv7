use crate::paging::VirtAddr;

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

    pub const fn user(stack_pointer: VirtAddr) -> Context {
        Context {
            cr3: 0,
            rflags: 0x282,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rbp: 0,
            rsp: stack_pointer.as_u64() as usize,
        }
    }

    #[inline(never)]
    #[naked]
    pub unsafe fn switch_to(&mut self, next: &Context) {
        llvm_asm!("mov $0, cr3" : "=r"(self.cr3) : : "memory" : "intel", "volatile");
        if next.cr3 != self.cr3 {
            llvm_asm!("mov cr3, $0" : : "r"(next.cr3) : "memory" : "intel", "volatile");
        }

        llvm_asm!("pushfq ; pop $0" : "=r"(self.rflags) : : "memory" : "intel", "volatile");
        llvm_asm!("push $0 ; popfq" : : "r"(next.rflags) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, rbx" : "=r"(self.rbx) : : "memory" : "intel", "volatile");
        llvm_asm!("mov rbx, $0" : : "r"(next.rbx) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, r12" : "=r"(self.r12) : : "memory" : "intel", "volatile");
        llvm_asm!("mov r12, $0" : : "r"(next.r12) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, r13" : "=r"(self.r13) : : "memory" : "intel", "volatile");
        llvm_asm!("mov r13, $0" : : "r"(next.r13) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, r14" : "=r"(self.r14) : : "memory" : "intel", "volatile");
        llvm_asm!("mov r14, $0" : : "r"(next.r14) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, r15" : "=r"(self.r15) : : "memory" : "intel", "volatile");
        llvm_asm!("mov r15, $0" : : "r"(next.r15) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, rsp" : "=r"(self.rsp) : : "memory" : "intel", "volatile");
        llvm_asm!("mov rsp, $0" : : "r"(next.rsp) : "memory" : "intel", "volatile");

        llvm_asm!("mov $0, rbp" : "=r"(self.rbp) : : "memory" : "intel", "volatile");
        llvm_asm!("mov rbp, $0" : : "r"(next.rbp) : "memory" : "intel", "volatile");
    }
}

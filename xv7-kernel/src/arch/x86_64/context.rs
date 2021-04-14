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
    pub unsafe extern "C" fn switch_to(&mut self, _next: &Context) {
        asm!(
            "
            mov rcx, cr3
            mov [rdi], rcx
            mov rax, [rsi]
            cmp rax, rcx
    
            je switch_to.same_cr3
            mov cr3, rax
    
            switch_to.same_cr3:
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
            options(noreturn)
        );
    }
}

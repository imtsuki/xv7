use super::gdt;
use super::macros::SyscallStackFrame;
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

#[naked]
#[inline(never)]
pub unsafe extern "C" fn syscall_entry() {
    asm!(
        "swapgs",                     // Load kernel TSS pointer
        "mov gs:[0x1c], rsp",         // Save userspace %rsp
        "mov rsp, gs:[0x4]",          // Load TSS %rsp
        "push 0x1b",                  // Push userspace data segment
        "push gs:[0x1c]",             // Push userspace %rsp
        "mov QWORD PTR gs:[0x1c], 0", // Clear userspace %rsp
        "push r11",                   // Push rflags
        "push 0x23",                  // Push userspace code segment
        "push rcx",                   // Push userspace return pointer
        "swapgs",                     // Restore %gs
        // push preserved
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // push scratch
        "push rax",
        "push r11",
        "push r10",
        "push r9",
        "push r8",
        "push rdi",
        "push rsi",
        "push rdx",
        "push rcx",
        // call syscall_inner
        "mov rdi, rsp",
        "call {}",
        // pop scratch
        "pop rcx",
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop r8",
        "pop r9",
        "pop r10",
        "pop r11",
        "pop rax",
        // pop preserved
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        // interrupt return
        "iretq",
        sym syscall_inner,
        options(noreturn),
    );
}

extern "C" fn syscall_inner(stack: *mut SyscallStackFrame) {
    let stack = unsafe { &mut *stack };
    let scratch = &stack.scratch;
    stack.scratch.rax = crate::syscall::syscall(
        scratch.rax,
        scratch.rdi,
        scratch.rsi,
        scratch.rdx,
        scratch.r10,
        scratch.r8,
    );
}

pub fn init() {
    // Setup syscall/sysret cs/ss
    Star::write(
        gdt::GDT.1.user_code_selector,
        gdt::GDT.1.user_data_selector,
        gdt::GDT.1.kernel_code_selector,
        gdt::GDT.1.kernel_data_selector,
    )
    .unwrap();

    // Setup syscall target rip
    LStar::write(VirtAddr::from_ptr(syscall_entry as *const u8));

    // Setup flags to clear
    let mask = RFlags::TRAP_FLAG
        | RFlags::DIRECTION_FLAG
        | RFlags::INTERRUPT_FLAG
        | RFlags::IOPL_HIGH
        | RFlags::IOPL_LOW
        | RFlags::ALIGNMENT_CHECK
        | RFlags::NESTED_TASK;

    SFMask::write(mask);

    KernelGsBase::write(VirtAddr::from_ptr(&*gdt::TSS as *const _));

    // Enable syscall extensions
    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }

    dbg!(&*gdt::TSS);

    dbg!(Efer::read());
    dbg!(Star::read());
    dbg!(LStar::read());
    dbg!(SFMask::read());
    dbg!(KernelGsBase::read());
}

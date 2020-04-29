use super::gdt;
use super::macros::SyscallStackFrame;
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

#[naked]
pub unsafe extern "C" fn syscall_entry() {
    llvm_asm!(
    "
        swapgs              // Load kernel TSS pointer
        movq %rsp, %gs:28   // Save userspace %rsp
        movq %gs:4, %rsp    // Load TSS %rsp
        pushq $$0x1b        // Push userspace data segment
        pushq %gs:28        // Push userspace %rsp
        movq $$0, %gs:28    // Clear userspace %rsp
        pushq %r11          // Push rflags
        pushq $$0x23        // Push userspace code segment
        pushq %rcx          // Push userspace return pointer
        swapgs              // Restore %gs
    " : : : : "volatile"
    );

    scratch_push!();
    preserved_push!();

    // FIXME: overrides stack.preserved.r15?
    let rsp: usize;
    llvm_asm!("" : "={rsp}"(rsp) : : : "volatile");

    syscall_inner(rsp as *mut SyscallStackFrame);

    preserved_pop!();
    scratch_pop!();

    interrupt_return!();
}

fn syscall_inner(stack: *mut SyscallStackFrame) {
    println!("Ready to go back to userspace and trigger a #PF...");
    unsafe {
        println!("{:x}", &(*stack).iret.rip);
    }
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

    unsafe { syscall_entry() };
}

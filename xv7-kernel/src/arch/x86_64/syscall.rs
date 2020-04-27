use super::gdt::GDT;
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

#[naked]
pub unsafe extern "C" fn syscall_entry() {
    /* TODO */
}

pub fn init() {
    // Setup syscall/sysret cs/ss
    Star::write(
        GDT.1.user_code_selector,
        GDT.1.user_data_selector,
        GDT.1.kernel_code_selector,
        GDT.1.kernel_data_selector,
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

    // Enable syscall extensions
    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }

    dbg!(Efer::read());
    dbg!(Star::read());
    dbg!(LStar::read());
    dbg!(SFMask::read());
}

use super::gdt::GDT;
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star};
use x86_64::VirtAddr;

extern "C" fn __syscall() {
    /* TODO */
}

pub fn init() {
    Star::write(
        GDT.1.user_code_selector,
        GDT.1.user_data_selector,
        GDT.1.kernel_code_selector,
        GDT.1.kernel_data_selector,
    )
    .unwrap();

    LStar::write(VirtAddr::from_ptr(__syscall as *const u8));

    unsafe {
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));
    }

    dbg!(Efer::read());
    dbg!(Star::read());
    dbg!(LStar::read());
}

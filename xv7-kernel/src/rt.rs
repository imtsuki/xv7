use alloc::alloc::Layout;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::arch::interrupt::disable();
    crate::println!("kernel {}", info);
    crate::arch::idle();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Failed to allocate memory: {:?}", layout)
}

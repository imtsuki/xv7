pub mod allocator;
pub mod config;
mod console;
pub mod context;
mod cpuid;
pub mod device;
pub mod gdt;
pub mod interrupt;
pub mod paging;
pub mod percpu;
mod start;
pub mod syscall;
pub mod trapframe;

pub use interrupt::{disable as disable_interrupts, without_interrupts};
pub use percpu::set_kernel_stack;

/// Atomically enables interrupts, halts until the next hardware interrupt,
/// and disables interrupts again upon wakeup.
#[inline]
pub fn wait_for_interrupt() {
    interrupt::enable_and_hlt();
    interrupt::disable();
}

#[inline(always)]
pub fn idle() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

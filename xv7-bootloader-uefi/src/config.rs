/// Kernel image path in ESP.
pub const KERNEL_IMAGE_PATH: &'static str = r"\EFI\xv7\kernel";

/// Higher half address sapce offset.
pub const VIRTUAL_OFFSET: usize = 0xFFFF_8000_0000_0000;

/// Base address where kernel is loaded.
pub const KERNEL_VIRTUAL_BASE: usize = KERNEL_PHYSICAL_BASE + VIRTUAL_OFFSET;
pub const KERNEL_PHYSICAL_BASE: usize = 0x10_0000;

/// Temporary kernel stack.
pub const STACK_VIRTUAL: usize = STACK_PHYSICAL + VIRTUAL_OFFSET;
pub const STACK_PHYSICAL: usize = 0x8_0000;

// FIXME: stack pointer and size are arbitrary
pub const STACK_SIZE: usize = 0x1_0000;

/// Temporary page table used for kernel booting.
pub const L4_PAGE_TABLE: usize = 0x7_0000;

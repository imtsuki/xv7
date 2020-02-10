/// Kernel image path in ESP.
pub const KERNEL_IMAGE_PATH: &'static str = r"\EFI\xv7\kernel";

/// Higher half address sapce offset.
pub const VIRTUAL_OFFSET: u64 = PAGE_OFFSET_BASE;

pub const PAGE_OFFSET_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Base address where kernel is loaded.
pub const KERNEL_VIRTUAL_BASE: u64 = KERNEL_PHYSICAL_BASE + VIRTUAL_OFFSET;
pub const KERNEL_PHYSICAL_BASE: u64 = 0x10_0000;

/// Temporary kernel stack.
pub const STACK_VIRTUAL: u64 = STACK_PHYSICAL + VIRTUAL_OFFSET;
pub const STACK_PHYSICAL: u64 = 0x8_0000;

// FIXME: stack pointer and size are arbitrary
pub const STACK_SIZE: usize = 0x1_0000;

/// Temporary page table used for kernel booting.
pub const L4_PAGE_TABLE: u64 = 0x7_0000;

/// Where MemoryMaps is placed
#[allow(unused)]
pub const MEMORY_MAPS_VIRTUAL_BASE: u64 = MEMORY_MAPS_PHYSICAL_BASE + VIRTUAL_OFFSET;
pub const MEMORY_MAPS_PHYSICAL_BASE: u64 = 0x6_0000;

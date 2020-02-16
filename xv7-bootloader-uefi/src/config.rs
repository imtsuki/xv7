//! # Configuration Constants
//!
//! ## Virtual memory map of xv7
//!
//! | Range                                           | Size     | Description                             | Constant              |
//! |-------------------------------------------------|----------|-----------------------------------------|-----------------------|
//! | `0x0000_0000_0000_0000 ~ 0x0000_7FFF_FFFF_FFFF` |  128 TiB | User space                              |                       |
//! | `0xFFFF_8000_0000_0000 ~ 0xFFFF_8FFF_FFFF_FFFF` |   16 TiB | Direct physical memory mapping          | [`PAGE_OFFSET_BASE`]  |
//! | `0xFFFF_9000_0000_0000 ~ 0xFFFF_9FFF_FFFF_FFFF` |   16 TiB | Kernel `.text` `.data` `.rodata` `.bss` | [`KERNEL_BASE`]       |
//! | `0xFFFF_A000_0000_0000 ~ 0xFFFF_CFFF_FFFF_FFFF` |   48 TiB | Kernel heap and stack                   | [`KERNEL_HEAP_BASE`]  |
//! | `0xFFFF_D000_0000_0000 ~ 0xFFFF_DFFF_FFFF_FFFF` |   16 TiB | UEFI mapping                            | [`UEFI_MAPPING_BASE`] |
//! | `0xFFFF_E000_0000_0000 ~ 0xFFFF_EFFF_FFFF_FFFF` |   16 TiB | MMIO, Device space                      | [`DEVICE_BASE`]       |
//! | `0xFFFF_F000_0000_0000 ~ 0xFFFF_FF7F_FFFF_FFFF` | 15.5 TiB | Guard hole (Unused)                     |                       |
//! | `0xFFFF_FF80_0000_0000 ~ 0xFFFF_FFFF_FFFF_FFFF` |  0.5 TiB | Recursive page table                    |                       |
//!
//! ### Kernel stack layout
//!
//! TODO
//!

/// Kernel image path in ESP.
pub const KERNEL_IMAGE_PATH: &'static str = r"\EFI\xv7\kernel";

/// Direct physical memory mapping.
pub const PAGE_OFFSET_BASE: u64 = 0xFFFF_8000_0000_0000;
/// Base address where kernel is loaded. `.text` `.data` `.rodata` `.bss`
pub const KERNEL_BASE: u64 = 0xFFFF_9000_0000_0000;
/// Kernel heap
#[allow(unused)]
pub const KERNEL_HEAP_BASE: u64 = 0xFFFF_A000_0000_0000;
#[allow(unused)]
pub const KERNEL_STACK_TOP: u64 = 0xFFFF_D000_0000_0000;
/// UEFI mapping
#[allow(unused)]
pub const UEFI_MAPPING_BASE: u64 = 0xFFFF_D000_0000_0000;
/// MMIO, Device space
#[allow(unused)]
pub const DEVICE_BASE: u64 = 0xFFFF_E000_0000_0000;

/// Higher half address sapce offset.
pub const VIRTUAL_OFFSET: u64 = PAGE_OFFSET_BASE;

/// Temporary kernel stack.
pub const STACK_VIRTUAL: u64 = STACK_PHYSICAL + VIRTUAL_OFFSET;
pub const STACK_PHYSICAL: u64 = 0x8_0000;

/// FIXME: stack pointer and size are arbitrary
pub const STACK_SIZE: usize = 0x1_0000;

/// Where MemoryMaps is placed
#[allow(unused)]
pub const MEMORY_MAPS_VIRTUAL_BASE: u64 = MEMORY_MAPS_PHYSICAL_BASE + VIRTUAL_OFFSET;
pub const MEMORY_MAPS_PHYSICAL_BASE: u64 = 0x6_0000;

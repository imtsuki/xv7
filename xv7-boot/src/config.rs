//! # Configuration Constants
//!
//! ## Virtual memory map of xv7
//!
//! | Range                                           | Size     | Description                             | Constant              |
//! |-------------------------------------------------|----------|-----------------------------------------|-----------------------|
//! | `0x0000_0000_0000_0000 ~ 0x0000_7FFF_FFFF_FFFF` |  128 TiB | User space                              |                       |
//! | `0xFFFF_8000_0000_0000 ~ 0xFFFF_BFFF_FFFF_FFFF` |   64 TiB | Direct physical memory mapping          | [`PAGE_OFFSET_BASE`]  |
//! | `0xFFFF_C000_0000_0000 ~ 0xFFFF_CFFF_FFFF_FFFF` |   16 TiB | Kernel `.text` `.data` `.rodata` `.bss` | [`KERNEL_BASE`]       |
//! | `0xFFFF_D000_0000_0000 ~ 0xFFFF_DFFF_FFFF_FFFF` |   16 TiB | Kernel heap and stack                   | [`KERNEL_HEAP_BASE`]  |
//! | `0xFFFF_E000_0000_0000 ~ 0xFFFF_E7FF_FFFF_FFFF` |    8 TiB | UEFI mapping                            | [`UEFI_MAPPING_BASE`] |
//! | `0xFFFF_E800_0000_0000 ~ 0xFFFF_EFFF_FFFF_FFFF` |    8 TiB | MMIO, Device space                      | [`DEVICE_BASE`]       |
//! | `0xFFFF_F000_0000_0000 ~ 0xFFFF_FF7F_FFFF_FFFF` | 15.5 TiB | Guard hole (Unused)                     |                       |
//! | `0xFFFF_FF80_0000_0000 ~ 0xFFFF_FFFF_FFFF_FFFF` |  0.5 TiB | Recursive page table                    |                       |

/// Direct physical memory mapping.
pub const PAGE_OFFSET_BASE: u64 = 0xFFFF_8000_0000_0000;
/// Base address where kernel is loaded. `.text` `.data` `.rodata` `.bss`
pub const KERNEL_BASE: u64 = 0xFFFF_C000_0000_0000;
/// Kernel heap.
pub const KERNEL_HEAP_BASE: u64 = 0xFFFF_D000_0000_0000;
/// Size of kernel heap.
pub const KERNEL_HEAP_SIZE: usize = 128 * 1024;
/// Top address of kernel stack.
pub const KERNEL_STACK_TOP: u64 = 0xFFFF_E000_0000_0000;
/// UEFI mapping
#[allow(unused)]
pub const UEFI_MAPPING_BASE: u64 = 0xFFFF_E000_0000_0000;
/// MMIO, Device space
#[allow(unused)]
pub const DEVICE_BASE: u64 = 0xFFFF_E800_0000_0000;

/// Initial kernel stack size.
pub const KERNEL_STACK_SIZE: usize = 0x1_0000;

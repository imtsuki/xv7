//! FIXME: Add missing docs

#![no_std]
#![deny(missing_docs)]

pub mod config;

use core::fmt;
pub use x86_64::{PhysAddr, VirtAddr};

/// Function signature for kernel entry point.
#[cfg(target_arch = "x86_64")]
pub type KernelEntryFn = extern "sysv64" fn(args: &BootArgs) -> !;

/// Function signature for kernel entry point.
///
/// FIXME: Do we really need a bootloader for aarch64?
#[cfg(target_arch = "aarch64")]
pub type KernelEntryFn = extern "C" fn(args: &BootArgs) -> !;

// For other platforms, no bootloader is needed AFAIK.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub type KernelEntryFn = extern "C" fn() -> !;

/// Kernel entry's virtual address.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct KernelEntry(pub VirtAddr);

impl Into<VirtAddr> for KernelEntry {
    fn into(self) -> VirtAddr {
        self.0
    }
}

impl From<VirtAddr> for KernelEntry {
    fn from(addr: VirtAddr) -> Self {
        Self(addr)
    }
}

/// Bootloader passes `BootArgs` to the kernel entry,
/// containing boot information.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct BootArgs {
    /// Magic number for checking whether `BootArgs` is passed correctly
    pub magic: u64,
    /// Video frame buffer
    pub frame_buffer: FrameBufferDescriptor,
    /// Memory map
    pub memory_map: MemoryMap,
}

/// Type of a memory region in the memory map.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MemoryType(pub u32);

impl MemoryType {
    /// Not usable.
    pub const RESERVED: Self = Self(0);
    /// The code portions of a loaded UEFI application.
    pub const LOADER_CODE: Self = Self(1);
    /// The data portions of a loaded UEFI application.
    pub const LOADER_DATA: Self = Self(2);
    /// The code portions of a loaded UEFI Boot Service Driver.
    pub const BOOT_SERVICES_CODE: Self = Self(3);
    /// The data portions of a loaded UEFI Boot Services Driver.
    pub const BOOT_SERVICES_DATA: Self = Self(4);
    /// The code portions of a loaded UEFI Runtime Driver.
    pub const RUNTIME_SERVICES_CODE: Self = Self(5);
    /// The data portions of a loaded UEFI Runtime Driver.
    pub const RUNTIME_SERVICES_DATA: Self = Self(6);
    /// Free (unallocated) memory.
    pub const CONVENTIONAL: Self = Self(7);
    /// Memory in which errors have been detected.
    pub const UNUSABLE: Self = Self(8);
    /// Memory that holds the ACPI tables.
    pub const ACPI_RECLAIM: Self = Self(9);
    /// Address space reserved for use by the firmware.
    pub const ACPI_NON_VOLATILE: Self = Self(10);
    /// Memory-mapped IO region.
    pub const MMIO: Self = Self(11);
    /// System memory-mapped IO port space.
    pub const MMIO_PORT_SPACE: Self = Self(12);
    /// Address space reserved by the firmware for processor code.
    pub const PAL_CODE: Self = Self(13);
    /// Byte-addressable non-volatile memory.
    pub const PERSISTENT_MEMORY: Self = Self(14);
}

impl fmt::Debug for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            Self::RESERVED => "RESERVED",
            Self::LOADER_CODE => "LOADER_CODE",
            Self::LOADER_DATA => "LOADER_DATA",
            Self::BOOT_SERVICES_CODE => "BOOT_SERVICES_CODE",
            Self::BOOT_SERVICES_DATA => "BOOT_SERVICES_DATA",
            Self::RUNTIME_SERVICES_CODE => "RUNTIME_SERVICES_CODE",
            Self::RUNTIME_SERVICES_DATA => "RUNTIME_SERVICES_DATA",
            Self::CONVENTIONAL => "CONVENTIONAL",
            Self::UNUSABLE => "UNUSABLE",
            Self::ACPI_RECLAIM => "ACPI_RECLAIM",
            Self::ACPI_NON_VOLATILE => "ACPI_NON_VOLATILE",
            Self::MMIO => "MMIO",
            Self::MMIO_PORT_SPACE => "MMIO_PORT_SPACE",
            Self::PAL_CODE => "PAL_CODE",
            Self::PERSISTENT_MEMORY => "PERSISTENT_MEMORY",
            Self(other) => return write!(f, "CUSTOM({:#x})", other),
        };
        f.write_str(name)
    }
}

/// Memory descriptor compatible with the UEFI `EFI_MEMORY_DESCRIPTOR` layout.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of memory region.
    pub ty: MemoryType,
    /// Padding for 64-bit alignment.
    pub _pad: u32,
    /// Physical address of the first byte in the memory region.
    pub phys_start: u64,
    /// Virtual address of the first byte in the memory region.
    pub virt_start: u64,
    /// Number of 4 KiB pages in the memory region.
    pub page_count: u64,
    /// Attributes of the memory region.
    pub att: u64,
}

/// Iterator over memory descriptors in a raw memory map buffer.
#[derive(Debug, Clone)]
pub struct MemoryMapIter<'buf> {
    buffer: &'buf [u8],
    entry_size: usize,
    index: usize,
    len: usize,
}

impl<'buf> MemoryMapIter<'buf> {
    /// Creates a new `MemoryMapIter` over the given raw descriptor buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `buffer` is properly aligned for
    /// [`MemoryDescriptor`], `entry_size` is at least
    /// `size_of::<MemoryDescriptor>()` and a multiple of its alignment, and
    /// `buffer` contains at least `len` valid descriptors at `entry_size`
    /// byte intervals.
    pub unsafe fn new(buffer: &'buf [u8], entry_size: usize, len: usize) -> Self {
        assert!(entry_size >= core::mem::size_of::<MemoryDescriptor>());
        assert_eq!(entry_size % core::mem::align_of::<MemoryDescriptor>(), 0);
        assert_eq!(
            buffer
                .as_ptr()
                .align_offset(core::mem::align_of::<MemoryDescriptor>()),
            0
        );
        assert!(entry_size
            .checked_mul(len)
            .map_or(false, |total| total <= buffer.len()));

        Self {
            buffer,
            entry_size,
            index: 0,
            len,
        }
    }
}

impl<'buf> Iterator for MemoryMapIter<'buf> {
    type Item = &'buf MemoryDescriptor;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.len - self.index;

        (sz, Some(sz))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let ptr = self.buffer.as_ptr() as usize + self.entry_size * self.index;

            self.index += 1;

            let descriptor = unsafe { &*(ptr as *const MemoryDescriptor) };

            Some(descriptor)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {}

/// Memory map
#[repr(C)]
pub struct MemoryMap {
    /// Memory map iterator
    pub iter: MemoryMapIter<'static>,
}

impl Clone for MemoryMap {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MemoryMap")
            .field("len", &self.iter.len())
            .finish()
    }
}

/// `BootArgs` magic value.
pub const BOOT_ARGS_MAGIC: u64 = 0xcafe_beef;

/// Represents a range of pyhsical memory.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysMemoryDescriptor {
    /// Pyhsical Memory type
    pub memory_type: PhysMemoryType,
    /// Base address, should be aligned to `PAGE_SIZE`
    pub base: PhysAddr,
    /// number of pages in this range
    pub page_count: usize,
}

/// Types of `PhysMemoryDescriptor`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum PhysMemoryType {
    /// Conventional memory, can be used freely
    Usable,
    /// Occupied by kernel
    Kernel,
    /// ACPI related memory region.
    Acpi,
    /// Used for UEFI Runtime services.
    UefiRuntime,
    /// Reserved
    Reserved,
}

/// Describe video frame buffer.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FrameBufferDescriptor {
    /// Base address
    pub base: PhysAddr,
    /// buffer length
    pub len: usize,
    /// resolution
    pub resolution: (usize, usize),
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RsdpDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RsdpDescriptor20 {
    first_part: RsdpDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct SmbiosEntryPoint {
    /// This is `_SM_`
    entry_point_string: [u8; 4],
    /// This value summed with all the values of the table, should be 0 (overflow)
    checksum: u8,
    /// Length of the Entry Point Table. Since version 2.1 of SMBIOS, this is 0x1F
    length: u8,
    /// Major Version of SMBIOS
    major_version: u8,
    /// Minor Version of SMBIOS
    minor_version: u8,
    /// Maximum size of a SMBIOS Structure (we will se later)
    max_structure_size: u16,
    /// ...
    entry_point_revision: u8,
    /// ...
    formatted_area: [u8; 5],
    /// This is `_DMI_`
    entry_point_string2: [u8; 5],
    /// Checksum for values from EntryPointString2 to the end of table
    checksum2: u8,
    /// Length of the Table containing all the structures
    table_length: u16,
    /// Address of the Table
    table_address: u32,
    /// Number of structures in the table
    number_of_structures: u16,
    /// Unused
    bcd_revision: u8,
}

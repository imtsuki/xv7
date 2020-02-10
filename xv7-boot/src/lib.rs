//! FIXME: Add missing docs

#![no_std]
#![deny(missing_docs)]

pub use x86_64::{PhysAddr, VirtAddr};

/// Function signature for kernel entry point.
pub type KernelEntryFn = extern "sysv64" fn(args: &KernelArgs) -> !;

/// Kernel entry's virtual address.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct KernelEntry(pub VirtAddr);

impl Into<VirtAddr> for KernelEntry {
    fn into(self) -> VirtAddr {
        self.0
    }
}

/// Bootloader passes `KernelArgs` to the kernel entry,
/// containing some boot information.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct KernelArgs {
    /// Magic number for checking whether `KernelArgs` is passed correctly
    pub magic: u64,
    /// Video frame buffer
    pub frame_buffer: FrameBufferDescriptor,
    /* pub memory_map: &'static [MemoryDescriptor], */
}

/// `KernelArgs` magic value.
pub const KERNEL_ARGS_MAGIC: u64 = 0xcafe_beef_dead_babe;

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
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor20 {
    first_part: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct SMBIOSEntryPoint {
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

#![no_std]

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct KernelArgs {
    pub magic: u64,
}

pub const KERNEL_ARGS_MAGIC: u64 = 0xcafe_beef_dead_babe;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor20 {
    first_part: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct SMBIOSEntryPoint {
    entry_point_string: [u8; 4],  //This is _SM_
    checksum: u8, //This value summed with all the values of the table, should be 0 (overflow)
    length: u8,   //Length of the Entry Point Table. Since version 2.1 of SMBIOS, this is 0x1F
    major_version: u8, //Major Version of SMBIOS
    minor_version: u8, //Minor Version of SMBIOS
    max_structure_size: u16, //Maximum size of a SMBIOS Structure (we will se later)
    entry_point_revision: u8, //...
    formatted_area: [u8; 5], //...
    entry_point_string2: [u8; 5], //This is _DMI_
    checksum2: u8, //Checksum for values from EntryPointString2 to the end of table
    table_length: u16, //Length of the Table containing all the structures
    table_address: u32, //Address of the Table
    number_of_structures: u16, //Number of structures in the table
    bcd_revision: u8, //Unused
}

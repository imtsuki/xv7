#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BootInfo {}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct RSDPDescriptor20 {
    first_part: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

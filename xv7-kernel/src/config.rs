/// Maximum pyhsical memory that we decided to support.
pub const MAX_PHYSICAL_ADDRESS_SUPPORTED: usize = 32;
/// Maximum number of pages calculated from `MAX_PHYSICAL_ADDRESS_SUPPORTED`.
pub const MAX_PAGES_SUPPORTED: usize = 1 << MAX_PHYSICAL_ADDRESS_SUPPORTED >> 12;

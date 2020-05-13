pub use boot::config::*;

/// User stack.
pub const USER_STACK: u64 = 0x0000_7FFF_FFFF_F000;

/// User base
pub const USER_BASE: u64 = 0x0000_0001_0000_0000;

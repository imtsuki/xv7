pub use boot::config::*;

/// User stack.
pub const USER_STACK: u64 = 0x0000_8000_0000_0000 - USER_STACK_SIZE;

/// User stack size (1MB)
pub const USER_STACK_SIZE: u64 = 0x0000_0000_0010_0000;

/// User base
pub const USER_BASE: u64 = 0x0000_0001_0000_0000;

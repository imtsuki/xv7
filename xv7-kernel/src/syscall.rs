pub mod process;

use usyscall::number::*;

pub fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    match a {
        SYS_EXIT => println!("SYS_EXIT"),
        SYS_HELLO => println!("SYS_HELLO"),
        SYS_EXEC => println!("SYS_EXEC"),
        _ => println!("SYS_UNKNOWN"),
    }
    println!("b: {}, c: {}, d: {}, e: {}, f: {}", b, c, d, e, f);
    0
}

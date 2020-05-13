pub mod process;

#[allow(unused_variables)]
pub fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    println!("syscall: {}", a);
    println!("b: {}, c: {}, d: {}, e: {}, f: {}", b, c, d, e, f);
    0
}

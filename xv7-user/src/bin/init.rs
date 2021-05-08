//! user space init process (pid 0).
#![no_std]

#[macro_use]
extern crate xv7_user;

use xv7_user::io;
use xv7_user::syscall;
fn main() {
    syscall::mknod("console", 1).unwrap();
    syscall::open("console").unwrap();
    syscall::open("console").unwrap();

    syscall::r#yield().unwrap();
    println!(
        "[init] Hello from userspace! getpid() = {}",
        syscall::getpid().unwrap()
    );

    println!(
        r#"
       ___           ___        ___
      |\__\         /\__\      /\  \
      |:|  |       /:/  /      \:\  \
      |:|  |      /:/  /        \:\  \
      |:|__|__   /:/__/  ___     \:\  \
  ____/::::\__\  |:|  | /\__\     \:\__\
  \::::/~~/~     |:|  |/:/  /     /:/  /
   ~~|:|~~|      |:|__/:/  /     /:/  /
     |:|  |       \::::/__/     /:/  /
     |:|  |        ~~~~        /:/  /
      \|__|                    \/__/
"#
    );

    println!("Available commands (actually they are not): ls, cat, cp, mv, touch");
    let mut buf = [0; 128];
    loop {
        print!("$ ");
        let cmd = io::stdin().read_line(&mut buf);
        println!("sh: no such command `{}`", cmd);
    }
}

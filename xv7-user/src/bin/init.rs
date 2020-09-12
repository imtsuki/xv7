//! user space init process (pid 0).
#![no_std]

#[macro_use]
extern crate xv7_user;
use core::str;
use xv7_user::io::*;

fn main() {
    println!("Hello from userspace!");

    println!("Run some IO functions now ...");

    println!("create dir `/home`");
    mkdir("/home", 0);
    println!("create file `/home/text_file`");
    mknod("/home/text_file", 0);
    let fd = open("/home", FileMode::O_RDONLY);
    let mut dirs = [Direntory::default()];
    let ret = getdents(fd, &mut dirs);
    println!("list content of `/home`");
    for i in 0..ret {
        println!(
            "[{}] -> name: {} inode: {}",
            i,
            unsafe { str::from_utf8_unchecked(&dirs[i].name[0..dirs[i].name_len]) },
            dirs[i].ino
        );
    }
    close(fd);

    // w
    println!("let's write something to `/home/text_file`");
    let data = "bupt scs 2017";
    println!("write to `/home/text_file`: {}", data);
    let fd = open("/home/text_file", FileMode::O_WRONLY);
    write(fd, data.as_bytes());
    close(fd);

    // r
    let mut buf = [0u8; 20];
    let fd = open("/home/text_file", FileMode::O_RDONLY);
    let len = read(fd, &mut buf);
    println!("read from `/home/text_file`: {}", unsafe {
        str::from_utf8_unchecked(&buf[0..len])
    });
    close(fd);

    unlink("/home/text_file");
    unlink("/home");

    println!("init process exit");
}

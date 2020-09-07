use lazy_static::lazy_static;
use spin::Mutex;
use xv7_fs::vfs::*;
use xv7_fs_ramfs::ramfs;

pub fn init() {
    let mut fs = REGISTERED_FS.lock();
    fs.register_fs(FSType::RAMFS, ramfs::RamFS::mount);
    let (_rootfs, root_dentry) = fs.mount_fs(FSType::RAMFS, "".into());
    fs.set_root(&root_dentry);
    println!("[REGISTERED_FS]: {}", *fs);
    println!("[root]: {}", *fs.get_root().read());
}

lazy_static! {
    pub static ref REGISTERED_FS: Mutex<RegisteredFS> = {
        let fs = RegisteredFS::new();
        Mutex::new(fs)
    };
}

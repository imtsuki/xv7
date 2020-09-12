use super::*;
use lazy_static::lazy_static;
use spin::Mutex;
use usyscall::fs::*;
pub use xv7_fs::vfs::*;
use xv7_fs_ramfs::ramfs;

pub fn init() {
    {
        let mut fs = REGISTERED_FS.lock();
        fs.register_fs(FSType::RAMFS, ramfs::RamFS::mount);
        let (_rootfs, root_dentry) = fs.mount_fs(FSType::RAMFS, "".into());
        fs.set_root(&root_dentry);
        println!("[REGISTERED_FS]: {}", *fs);
        println!("[root]: {}", *fs.get_root().read());
    }
    test_vfs();
}

/// This function is used to temporarily demonstrate the availability of the file system
pub fn test_vfs() {
    println!("xv7-vfs test start");
    let root = REGISTERED_FS.lock().vfs_lookup("/").unwrap();
    println!("[vfs_lookup ({})] ret: {}", "/", *root.read());

    let dir = REGISTERED_FS.lock().vfs_mkdir("/dir").unwrap();
    println!("[vfs_mkdir ({})] ret: {}", "/dir", *dir.read());

    let file = REGISTERED_FS.lock().vfs_create("/dir/file").unwrap();
    println!("[vfs_create ({})] ret: {}", "/dir/file", *file.read());

    let mut stat = Stat::default();
    REGISTERED_FS
        .lock()
        .vfs_stat("/dir/file", &mut stat)
        .unwrap();
    println!("[vfs_stat ({} {:?})]", "/dir/file", stat);
    let mut opened_wo_file = REGISTERED_FS
        .lock()
        .vfs_open("/dir/file", FileMode::O_WRONLY)
        .unwrap();
    println!(
        "[vfs_open ({}, {:?})] ret: {}",
        "/dir/file",
        FileMode::O_WRONLY,
        *opened_wo_file.read()
    );

    let data1 = vec![1, 2, 3, 4, 5];
    let wr_len = REGISTERED_FS
        .lock()
        .vfs_write(&opened_wo_file, &data1)
        .unwrap();
    println!("[vfs_write ({} {:?})] ret: {}", *file.read(), data1, wr_len);

    REGISTERED_FS.lock().vfs_close(&opened_wo_file).unwrap();
    println!("[vfs_close ({})]", *opened_wo_file.read());

    let mut opened_ro_file = REGISTERED_FS
        .lock()
        .vfs_open("/dir/file", FileMode::O_RDONLY)
        .unwrap();

    let mut buf = vec![0u8; 20];
    let rd_len = REGISTERED_FS
        .lock()
        .vfs_read(&opened_ro_file, &mut buf)
        .unwrap();
    println!(
        "[vfs_read ({} {:?})] ret: {}",
        *file.read(),
        &buf[0..rd_len],
        rd_len
    );

    REGISTERED_FS.lock().vfs_close(&opened_ro_file).unwrap();
    println!("[vfs_close ({})]", *opened_ro_file.read());

    let opened_dir = REGISTERED_FS
        .lock()
        .vfs_open("/dir", FileMode::O_RDWR)
        .unwrap();
    println!(
        "[vfs_open ({}, {:?})] ret: {}",
        "/dir",
        FileMode::O_RDWR,
        *opened_dir.read()
    );

    let mut dirs = vec![Direntory::default(); 2];
    let ret = REGISTERED_FS
        .lock()
        .vfs_readdir(&opened_dir, &mut dirs)
        .unwrap();
    println!(
        "[vfs_readdir ({} {:?})] ret: {}",
        *opened_dir.read(),
        &dirs,
        ret
    );

    REGISTERED_FS.lock().vfs_close(&opened_dir).unwrap();
    println!("[vfs_close ({})]", *opened_dir.read());

    REGISTERED_FS.lock().vfs_unlink("/dir/file");
    println!("[vfs_unlink ({})]", "/dir/file",);

    REGISTERED_FS.lock().vfs_unlink("/dir");
    println!("[vfs_unlink ({})]", "/dir",);

    println!("xv7-vfs test finished");
}

lazy_static! {
    pub static ref REGISTERED_FS: Mutex<RegisteredFS> = {
        let fs = RegisteredFS::new();
        Mutex::new(fs)
    };
}

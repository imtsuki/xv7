use alloc::vec::Vec;
use uefi::prelude::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::Result;

pub fn read_file(services: &BootServices, path: &str) -> Result<(usize, Vec<u8>)> {
    let fatfs = services
        .locate_protocol::<SimpleFileSystem>()
        .log_warning()?;
    let fatfs = unsafe { &mut *fatfs.get() };

    let mut volume = fatfs.open_volume().log_warning()?;

    let file_handle = volume
        .open(path, FileMode::Read, FileAttribute::empty())
        .log_warning()?;

    let mut file = match file_handle.into_type().log_warning()? {
        FileType::Regular(file) => file,
        FileType::Dir(_) => unreachable!(),
    };

    // Use an empty buffer to retrieve the actual FileInfo size
    let mut empty_buf = Vec::new();
    let len = match *file
        .get_info::<FileInfo>(&mut empty_buf)
        .expect_error("passing an empty buffer will return the size of FileInfo")
        .data()
    {
        Some(len) => len,
        None => unreachable!(),
    };

    let mut file_info = vec![0u8; len];
    let file_info = file
        .get_info::<FileInfo>(&mut file_info)
        .discard_errdata()
        .log_warning()?;

    let mut buf = vec![0u8; file_info.file_size() as usize];
    let len = file.read(&mut buf).discard_errdata().log_warning()?;

    Ok((len, buf).into())
}

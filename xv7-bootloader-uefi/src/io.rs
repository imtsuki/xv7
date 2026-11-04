use alloc::vec::Vec;
use core::convert::TryFrom;
use uefi::prelude::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::{CString16, Result};

pub fn read_file(services: &BootServices, path: &str) -> Result<(usize, Vec<u8>)> {
    let handle = services.get_handle_for_protocol::<SimpleFileSystem>()?;
    let mut fatfs = services.open_protocol_exclusive::<SimpleFileSystem>(handle)?;

    let mut volume = fatfs.open_volume()?;

    let path = CString16::try_from(path).unwrap();
    let file_handle = volume.open(&path, FileMode::Read, FileAttribute::empty())?;

    let mut file = match file_handle.into_type()? {
        FileType::Regular(file) => file,
        FileType::Dir(_) => unreachable!(),
    };

    let file_info = file.get_boxed_info::<FileInfo>()?;

    let mut buf = vec![0u8; file_info.file_size() as usize];
    let len = file.read(&mut buf)?;

    Ok((len, buf))
}

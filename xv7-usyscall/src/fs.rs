use core::fmt;

const NAME_MAX_LEN: usize = 255;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum INodeType {
    IFIFO,
    IFCHR,
    IFDIR,
    IFBLK,
    IFREG,
    IFLNK,
    IFSOCK,
}

impl Default for INodeType {
    fn default() -> Self {
        INodeType::IFREG
    }
}

bitflags! {
pub struct FileMode:u32 {
    const O_RDONLY = 0b00000001;
    const O_WRONLY = 0b00000010;
    const O_RDWR = 0b00000100;
    const O_APPEND = 0b00001000;    // mark the target file can only be appended
    const O_CREAT = 0b00010000;     // (not currently implemented)
    const O_DIRECTORY = 0b00100000;
}
}

#[derive(Clone)]
pub struct Direntory {
    pub ino: usize,                   /* inode number */
    pub off: usize,                   /* offset to this dirent */
    pub name_len: usize,              /* length of this d_name */
    pub name: [u8; NAME_MAX_LEN + 1], /* filename (null-terminated) */
}

impl Default for Direntory {
    fn default() -> Direntory {
        Direntory {
            ino: 0,
            off: 0,
            name_len: 0,
            name: [0; NAME_MAX_LEN + 1],
        }
    }
}

impl fmt::Debug for Direntory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Direntory {{ino: {}, off: {}, name_len: {}, name: {:?}}}",
            self.ino,
            self.off,
            self.name_len,
            &self.name[0..self.name_len],
        )
    }
}

#[derive(Clone, Default, Debug)]
pub struct Stat {
    pub mode: INodeType,
    pub uid: usize,
    pub gid: usize,
    pub ino: usize,
    pub atime: usize,
    pub mtime: usize,
    pub ctime: usize,
    pub nlink: usize,
}

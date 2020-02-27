use core::fmt;

pub struct PrettyRef<'a, T>(&'a T);

pub trait Pretty<T> {
    fn pretty(&self) -> PrettyRef<T>;
}

impl<T> Pretty<T> for T {
    fn pretty(&self) -> PrettyRef<T> {
        PrettyRef(self)
    }
}

impl<'a> fmt::Display for PrettyRef<'a, usize> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self.0 >= 1 << 40 {
            write!(f, "{}TiB", self.0 >> 40)
        } else if *self.0 >= 1 << 30 {
            write!(f, "{}GiB", self.0 >> 30)
        } else if *self.0 >= 1 << 20 {
            write!(f, "{}MiB", self.0 >> 20)
        } else if *self.0 >= 1 << 10 {
            write!(f, "{}KiB", self.0 >> 10)
        } else {
            write!(f, "{}B", self.0)
        }
    }
}

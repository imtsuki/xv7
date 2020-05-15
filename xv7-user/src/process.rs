/// A trait for implementing arbitrary return types in the `main` function.
///
/// The C-main function only supports to return integers as return type.
/// So, every type implementing the `Termination` trait has to be converted
/// to an integer.
///
/// The default implementations are returning `libc::EXIT_SUCCESS` to indicate
/// a successful execution. In case of a failure, `libc::EXIT_FAILURE` is returned.
pub trait Termination {
    /// Is called to get the representation of the value as status code.
    /// This status code is returned to the operating system.
    fn report(self) -> i32;
}

impl Termination for () {
    #[inline]
    fn report(self) -> i32 {
        0
    }
}

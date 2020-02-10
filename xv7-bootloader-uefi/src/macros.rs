/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// Copied from standard library with slight modifications.
#[macro_export]
macro_rules! dbg {
    () => {
        info!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                info!("[{}:{}] {} = {:#x?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { $crate::dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

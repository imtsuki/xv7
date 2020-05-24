use crate::arch::interrupt::without_interrupts;
use core::fmt;
use core::fmt::Write;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print!("{}\n", format_args!($($arg)*));
    })
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    without_interrupts(|| {
        crate::device::console::CONSOLE_DRIVERS
            .lock()
            .write_fmt(args)
            .unwrap();
    })
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// Copied from standard library with slight modifications.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:?}",
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

#[macro_export]
macro_rules! dbg_by_eyes {
    ($args:expr) => (
        unsafe {
            use crate::config::PAGE_OFFSET_BASE;

            for i in $args.frame_buffer.base.as_u64()..$args.frame_buffer.base.as_u64()+(($args.frame_buffer.resolution.0 as u64)*200) {
                *((i+PAGE_OFFSET_BASE) as *mut u8) = 255;
            }
            loop{
                llvm_asm!("hlt");
            } 
        }
    );
}
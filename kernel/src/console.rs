//! The `console` module contains functions that interacts with the debug console.
//! It exports useful macros such as `print!` and `println!`.

use core::fmt::{self, Write};

/// The `Console` struct implements the [Write] trait, which invokes the
/// `sbi_rt::console_write_byte` function.
struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for char in string.bytes() {
            sbi_rt::console_write_byte(char);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

/// Print to the debug console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

/// Print to the debug console, with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

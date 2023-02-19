//! SBI console driver, for text output

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! hdebug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("[Hypervisor] ", $fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! hwarning {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("[Warning] ", $fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! htracking {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[1;32m[Tracking] ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! herror {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[1;31m[Error] ", $fmt, "\x1b[0m\n") $(, $($arg)+)?));
    }
}

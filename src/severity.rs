use log::LogLevel;

use meta::format::{Format, Formatter};

pub type Error = ::std::io::Error;

pub trait Severity {
    /// Returns an integer severity representation.
    fn as_i32(&self) -> i32;

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized;
}

impl Severity for i32 {
    fn as_i32(&self) -> i32 {
        *self
    }

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized
    {
        val.format(format)
    }
}

impl Severity for LogLevel {
    fn as_i32(&self) -> i32 {
        match *self {
            LogLevel::Error => 1,
            LogLevel::Warn  => 2,
            LogLevel::Info  => 3,
            LogLevel::Debug => 4,
            LogLevel::Trace => 5,
        }
    }

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized
    {
        match val {
            1 => format.write_str("Error"),
            2 => format.write_str("Warn"),
            3 => format.write_str("Info"),
            4 => format.write_str("Debug"),
            5 => format.write_str("Trace"),
            val => val.format(format),
        }
    }
}

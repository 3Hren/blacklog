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
            LogLevel::Error => 4,
            LogLevel::Warn  => 3,
            LogLevel::Info  => 2,
            LogLevel::Debug => 1,
            LogLevel::Trace => 0,
        }
    }

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized
    {
        match val {
            4 => format.write_str("Error"),
            3 => format.write_str("Warn"),
            2 => format.write_str("Info"),
            1 => format.write_str("Debug"),
            0 => format.write_str("Trace"),
            val => val.format(format),
        }
    }
}

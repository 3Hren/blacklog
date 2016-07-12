use meta::format::{Format, Formatter};

pub type Error = ::std::io::Error;

pub trait Severity {
    fn num(&self) -> i32;
    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized;
}

impl Severity for i32 {
    fn num(&self) -> i32 {
        *self
    }

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized
    {
        val.format(format)
    }
}

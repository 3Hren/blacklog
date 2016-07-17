use std::error;
use std::io::{Error, Write};

use {Config, Output, Record, Registry};

use factory::Factory;

/// A null output merely exists, it never outputs a message to any device.
///
/// This struct exists primarily for benchmarking reasons to measure the entire logging processing
/// pipeline. It never fails, because it does nothing.
///
/// ```
/// #[macro_use] extern crate blacklog;
///
/// use blacklog::Output;
/// use blacklog::output::NullOutput;
///
/// fn main() {
///     let out = NullOutput;
///
///     assert!(out.write(&record!(0), &[]).is_ok());
/// }
/// ```
pub struct NullOutput;

impl Output for NullOutput {
    #[allow(unused_variables)]
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

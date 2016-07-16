use std::error;
use std::io::{Error, Write};

use {Config, Output, Record, Registry};

use factory::Factory;

/// A null output merely exists, it never outputs a message to any device.
///
/// This struct exists primarily for benchmarking reasons for measuring the entire logging
/// processing pipeline.
/// It never fails, because it does nothing.
pub struct NullOutput;

impl Output for NullOutput {
    #[allow(unused_variables)]
    fn write(&self, rec: &Record, message: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

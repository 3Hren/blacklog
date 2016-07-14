//! This example demonstrates how to initialize the Blacklog from the JSON file and write some
//! messages using it with runtime formatting.
//! Also a custom severity binding is shown.
// TODO: Split.

extern crate serde_json;
#[macro_use] extern crate blacklog;

use std::env;
use std::fs::File;
use std::io::Error;

use blacklog::{Format, Formatter, Severity, Registry};

enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Severity for Level {
    fn num(&self) -> i32 {
        match *self {
            Level::Debug => 0,
            Level::Info => 1,
            Level::Warn => 2,
            Level::Error => 3,
        }
    }

    fn format(val: i32, format: &mut Formatter) -> Result<(), Error>
        where Self: Sized
    {
        match val {
            0 => format.write_str("DEBUG"),
            1 => format.write_str("INFO"),
            2 => format.write_str("WARN"),
            3 => format.write_str("ERROR"),
            val => val.format(format),
        }
    }
}

fn main() {
    let path = env::args()
        .skip(1)
        .next()
        .expect("USAGE: config FILENAME");

    let cfg = serde_json::from_reader(File::open(&path).unwrap())
        .unwrap();

    let logger = Registry::new()
        .logger(&cfg)
        .expect("expect logger to be properly created");

    log!(logger, Level::Debug, "{} {} HTTP/1.1 {} {}", "GET", "/static/image.png", 404, 347);
    log!(logger, Level::Info, "nginx/1.6 configured");
    log!(logger, Level::Warn, "client stopped connection before send body completed");
    log!(logger, Level::Error, "file does not exist: {}", "/var/www/favicon.ico");
}

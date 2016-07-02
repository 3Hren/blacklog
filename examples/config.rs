//! This example demonstrates how to initialize the Blacklog from the JSON file and write some
//! messages using it.

extern crate serde_json;
#[macro_use] extern crate blacklog;

use std::env;
use std::fs::File;

use blacklog::Registry;

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

    log!(logger, 0, "{} {} HTTP/1.1 {} {}", "GET", "/static/image.png", 404, 347);
    log!(logger, 1, "nginx/1.6 configured");
    log!(logger, 2, "client stopped connection before send body completed");
    log!(logger, 3, "file does not exist: {}", "/var/www/favicon.ico");
}

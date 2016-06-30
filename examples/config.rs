//! This example demonstrates how to initialize and use Blacklog from the JSON file.

extern crate serde_json;
extern crate blacklog;

use std::env;
use std::fs::File;

use blacklog::Registry;

fn main() {
    let path = env::args().next().expect("expect a filename");
    let cfg: blacklog::Config = serde_json::from_reader(File::open(&path).unwrap()).unwrap();

    // let logger = Registry::new().logger(&cfg);

    // TODO: Log something.
}

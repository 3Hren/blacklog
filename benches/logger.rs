#![feature(test)]

extern crate test;
#[macro_use] extern crate blacklog;

use test::Bencher;

use blacklog::Logger;
use blacklog::Meta;

#[bench]
fn log(b: &mut Bencher) {
    let logger = Logger::new();
    b.iter(|| {
        log!(logger, 0, "le message", {
            path: "/usr/bin/env",
        });
    });
}

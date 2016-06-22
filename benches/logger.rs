#![feature(test)]

extern crate test;
#[macro_use] extern crate blacklog;

use test::Bencher;

use blacklog::Logger;
use blacklog::{Meta, MetaList};

#[bench]
fn log(b: &mut Bencher) {
    let logger = Logger::new();
    b.iter(|| {
        log!(logger, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            flag: true,
            path1: "/home1",
            path2: "/home2",
            path3: "/home3",
            path4: "/home4",
            path5: "/home5",
        });
    });
}

#![feature(test)]

extern crate test;
#[macro_use] extern crate blacklog;

use test::Bencher;

use blacklog::Logger;

#[bench]
fn log(b: &mut Bencher) {
    let log = Logger::new();
    b.iter(|| {
        log.log(0, "le message");
        // println!("{name}", name="sads", value=42);
        //
        // log!(log, 0, "le message {1} {0} {path}", (42, "///"), {
        //     path: "/home"
        // });
    });
}

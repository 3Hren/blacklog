#![feature(test)]

extern crate test;
extern crate chrono;

use std::io::Write;
use test::Bencher;
use chrono::UTC;

#[bench]
fn interpreter(b: &mut Bencher) {
    let now = UTC::now();
    let mut buf = Vec::with_capacity(128);

    let pattern = "%Y-%m-%d %H:%M:%S.%.6f".to_owned();
    b.iter(|| {
        write!(&mut buf, "{}", now.format(&pattern)).unwrap();
        buf.clear();
    });
}

#[bench]
fn compiler(b: &mut Bencher) {
    let now = UTC::now();
    let mut buf = Vec::with_capacity(128);

    let pattern = "%Y-%m-%d %H:%M:%S.%.6f".to_owned();
    let format = now.format(&pattern);

    b.iter(|| {
        write!(&mut buf, "{}", format).unwrap();
        buf.clear();
    });
}

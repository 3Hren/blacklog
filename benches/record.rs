#![feature(test)]

extern crate test;
extern crate blackhole;

use test::Bencher;

use blackhole::Record;

#[bench]
fn new(b: &mut Bencher) {
    b.iter(|| {
        let rec = Record::new(42, "le value");
        test::black_box(rec);
    });
}

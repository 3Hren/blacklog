#![feature(test)]

extern crate blacklog;
extern crate chrono;
extern crate test;

use test::Bencher;

use chrono::{DateTime, UTC};
use chrono::naive::date::NaiveDate;
use chrono::naive::datetime::NaiveDateTime;
use chrono::naive::time::NaiveTime;

use blacklog::{Meta, MetaLink, Record};

#[bench]
fn chrono_from_timestamp(b: &mut Bencher) {
    b.iter(|| {
        let timestamp: DateTime<UTC> = DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), UTC);
        test::black_box(timestamp);
    });
}

#[bench]
fn chrono_new(b: &mut Bencher) {
    b.iter(|| {
        let timestamp: DateTime<UTC> = DateTime::from_utc(
            NaiveDateTime::new(NaiveDate::from_ymd(0000, 01, 01), NaiveTime::from_hms(0, 0, 0)),
            UTC
        );
        test::black_box(timestamp);
    });
}

/// This benchmark demonstrates, that creating an inactive record is very cheap.
#[bench]
fn new(b: &mut Bencher) {
    b.iter(|| {
        Record::new(0, line!(), module_path!(), &MetaLink::new(&[]));
    });
}

/// This benchmark demonstrates, that creating an inactive record is very cheap, even with meta
/// attributes.
#[bench]
fn new_with_format_and_meta6(b: &mut Bencher) {
    b.iter(|| {
        Record::new(0, line!(), module_path!(),
            &MetaLink::new(&[Meta::new("meta#1", &42),
            Meta::new("meta#1", &42),
            Meta::new("meta#1", &42),
            Meta::new("meta#1", &42),
            Meta::new("meta#1", &42),
            Meta::new("meta#1", &42)]));
    });
}

#![feature(test)]

#[macro_use] extern crate blacklog;
extern crate test;

use test::Bencher;

use blacklog::Logger;
use blacklog::logger::ActorLogger;

#[bench]
fn actor_log(b: &mut Bencher) {
    let log = ActorLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    });
}

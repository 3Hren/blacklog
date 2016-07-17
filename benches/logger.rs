#![feature(test)]

#[macro_use] extern crate blacklog;
extern crate test;

use test::Bencher;

use blacklog::Logger;
use blacklog::logger::{ActorLogger, SeverityFilteredLoggerAdapter, SyncLogger};

#[bench]
fn sync_log(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    });
}

#[bench]
fn sync_log_with_meta1(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: /var/www/favicon.ico", {
            path1: "/home1",
        });
    });
}

#[bench]
fn sync_log_with_format_and_meta1(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            path1: "/home1",
        });
    });
}

#[bench]
fn sync_log_with_format_and_meta6(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            flag: true,
            path1: "/home1",
            path2: "/home2",
            path3: "/home3",
            path4: "/home4",
            path5: "/home5",
        });
    });
}

#[bench]
fn actor_log(b: &mut Bencher) {
    let log = ActorLogger::new(vec![]);

    b.iter(|| {
        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    });
}

#[bench]
fn sync_log_with_sev_adapter_deny(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);
    let log = SeverityFilteredLoggerAdapter::new(log);
    log.filter(1);

    b.iter(|| {
        log!(log, 0, "file does not exist: /var/www/favicon.ico");
    });
}

#[bench]
fn sync_log_with_format_and_meta6_with_sev_adapter_deny(b: &mut Bencher) {
    let log = SyncLogger::new(vec![]);
    let log = SeverityFilteredLoggerAdapter::new(log);
    log.filter(1);

    b.iter(|| {
        log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
            flag: true,
            path1: "/home1",
            path2: "/home2",
            path3: "/home3",
            path4: "/home4",
            path5: "/home5",
        });
    });
}

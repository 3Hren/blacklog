#[macro_use] extern crate blacklog;

use blacklog::Logger;
use blacklog::logger::SyncLogger;

#[test]
fn log_only_message() {
    let log = SyncLogger::new(vec![]);

    log!(log, 0, "file does not exist: /var/www/favicon.ico");
}

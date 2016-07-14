#[macro_use] extern crate blacklog;
extern crate log;

use log::LogLevel::*;

use blacklog::handle::Dev;
use blacklog::Logger;
use blacklog::logger::SyncLogger;

fn main() {
    // To demonstrate the basic functionality of Blackhole we introduce a Develop handle, which
    // prints all logs to the terminal in an eye-candy colored manner.
    let logger = SyncLogger::new(vec![Box::new(Dev)]);

    // Message formatting.
    loop{
    log!(logger, Debug, "{} {} HTTP/1.1 {} {}", "GET", "/static/image.png", 404, 347);

    // Attaching additional meta information.
    log!(logger, Info, "nginx/1.6 configured", {
        elapsed: 42.15,
        config: "/etc/nginx/nginx.conf",
    });

    log!(logger, Warn, "client stopped connection before send body completed", {
        host: "::1",
        port: 10053,
    });

    // And both. You can even use functions as meta.
    log!(logger, Error, "file does not exist: {}", ["/var/www/favicon.ico"], {
        expires: -1,
        content_type: "text/html",
    });
}
}

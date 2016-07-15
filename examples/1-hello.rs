//! Welcome! These examples will teach you about the easy logging with Rust using Blackhole
//! library.
//! The Blackhole is focused on following goals:
//! - High-performance structured logging (the main goal).
//! - Safe logging from multiple threads without using singletons.
//! - Events separation and termination with runtime configured filters.
//! - Customizable formatting and broadcasting into multiple destinations.
//! - Configuration from generic sources, like JSON, YAML etc. However, initialization directly in
//!   source code is allowed.
//! - At last, integration with the Standard Log crate.
//!
//! Each example will show you one or multiple features and are focused on some goal which is
//! described at the top of module documentation.
//!
//! Imagine, that you just want to start a new project with some kind of logging support, which will
//! print all messages in a eye-candy colored manner directly into the Standard Outout (aka stdout).
//!
//! Let's describe some base structs and traits involved in logging pipeline.
//!
//! A record is an object that contains all necessary information about logging event and acts like
//! a transport. It may be in either inactive or active state. If some record have passed filtering
//! it must be activated as like as it should be activated if all filters are neutral to such
//! record (or there are no filters at all). Then an active record are passed into several handlers.
//!
//! Handles are responsible for handling incoming records. They can do anything they want, but the
//! common pattern involves formatting a record and broadcast the obtained result into the all
//! associated outputs.
//! There are also layouts, filters and outputs, but we'll cover them later.
//!
//! Right now we'll create a special handler named `Dev`, which is okay for small applications
//! development, but is completely unappropriated for complex systems.
//!
//! You may note that Blackhole strongly discourages singleton abusing, in contrast with `log`
//! crate, where there is a single global logger, which is accessable from everywhere. Instead a
//! plain object must be created and used in conjunction with provided logging macro.

#[macro_use] extern crate blacklog;
extern crate log;

use log::LogLevel::*;

use blacklog::handle::Dev;
use blacklog::{Logger, FnMeta};
use blacklog::logger::SyncLogger;

fn main() {
    // To demonstrate the basic functionality of Blackhole we introduce a Develop handle, which
    // prints all logs to the terminal in an eye-candy colored manner.
    let logger = SyncLogger::new(vec![Box::new(Dev)]);

    // And that's all. Let's print some messages with runtime formatting.
    log!(logger, Debug, "{} {} HTTP/1.1 {} {}", "GET", "/static/image.png", 404, 347);

    // Attaching additional meta information.
    log!(logger, Info, "nginx/1.6 configured", {
        config: "/etc/nginx/nginx.conf",
        elapsed: 42.15,
    });

    // More ...
    log!(logger, Warn, "client stopped connection before send body completed", {
        host: "::1",
        port: 10053,
    });

    // And both. You can even use functions as meta for lazy evaluations.
    log!(logger, Error, "file does not exist: {}", ["/var/www/favicon.ico"], {
        path: "/",
        cache: true,
        method: "GET",
        version: 1.1,
        protocol: "HTTP",
        fibonacci: FnMeta::new(|| {
            (0..40).fold((0, 1), |acc, _| (acc.1, acc.0 + acc.1)).0
        }),
    });
}

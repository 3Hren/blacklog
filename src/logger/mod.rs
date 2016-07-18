use std::fmt::Arguments;
use std::ops::Deref;

use record::Record;

pub use self::actor::ActorLogger;
pub use self::filtered::{FilteredLoggerAdapter, SeverityFilteredLoggerAdapter};
pub use self::sync::SyncLogger;

mod actor;
mod filtered;
mod sync;

/// Loggers are, well, responsible for logging. Nuff said.
pub trait Logger: Send {
    /// Logs the given event using provided record and formatting arguments.
    ///
    /// # Note
    ///
    /// Loggers can be combined into chains with various fitlering stages.
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>);
}

impl<T: Logger + ?Sized, U: Deref<Target=T> + Send> Logger for U {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        self.deref().log(rec, args)
    }
}

// TODO: Docs.
#[macro_export]
macro_rules! log (
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {{
        $log.log(&mut $crate::Record::new($sev, line!(), module_path!(),
            &$crate::MetaLink::new(&[
                $($crate::Meta::new(stringify!($name), &$val)),*
            ])
        ), format_args!($fmt, $($args)*));
    }};
    ($log:ident, $sev:expr, $fmt:expr, {$($name:ident: $val:expr,)*}) => {{
        log!($log, $sev, $fmt, [], {$($name: $val,)*})
    }};
    ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*]) => {{
        log!($log, $sev, $fmt, [$($args)*], {})
    }};
    ($log:ident, $sev:expr, $fmt:expr, $($args:tt)*) => {{
        log!($log, $sev, $fmt, [$($args)*], {})
    }};
    ($log:ident, $sev:expr, $fmt:expr) => {{
        log!($log, $sev, $fmt, [], {})
    }};
);

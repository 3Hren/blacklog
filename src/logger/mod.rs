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
    // TODO: Return a result, which can be ignored (without #[must_use]).
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>);
}

impl<T: Logger + ?Sized> Logger for Box<T> {
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

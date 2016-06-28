// mod encode;
//
// pub use self::encode::Encode;
//
pub mod format;
// use std::fmt::{self, Debug};
// use std::sync::{mpsc, Arc, Mutex};
// use std::sync::atomic::{AtomicI32, Ordering};
// use std::thread::{self, JoinHandle};
//
// use {Record, InactiveRecord, Severity};
//
// use super::record::RecordBuf;
//
// use self::encode::{ToEncodeBox};
//
// pub use self::encode::Formatter;
//
// #[derive(Clone)]
// struct Lazy<F: Fn() -> E + Send + Sync + 'static, E: Encode>(Arc<Box<F>>);
//
// impl<F, E> Debug for Lazy<F, E>
//     where F: Fn() -> E + Send + Sync + 'static,
//           E: Encode
// {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), ::std::fmt::Error> {
//         write!(f, "[Lazy]")
//     }
// }
//
// impl<F, E> Lazy<F, E>
//     where F: Fn() -> E + Send + Sync + 'static,
//           E: Encode
// {
//     pub fn new(f: F) -> Lazy<F, E> {
//         Lazy(Arc::new(box f))
//     }
// }
//
// impl<F, E> Encode for Lazy<F, E>
//     where F: Fn() -> E + Send + Sync + 'static,
//           E: Encode
// {
//     fn encode(&self, encoder: &mut Formatter) -> Result<(), Error> {
//         self.0().encode(encoder)
//     }
// }
//
// impl<E, F> ToEncodeBox for Lazy<F, E>
//     where F: Fn() -> E + Send + Sync + 'static,
//           E: Encode + 'static
// {
//     fn to_encode_buf(&self) -> Box<Encode> {
//         box Lazy(self.0.clone())
//     }
// }
//
// enum FilterAction {
//     Deny,
//     Accept,
//     Neutral,
// }
//
// /// Filters are responsible for filtering logging events.
// ///
// /// # Note
// ///
// /// All filters must be satisfy Sync trait to be safely usable from multiple threads.
// trait Filter : Send + Sync {
//     fn filter<'a>(&self, rec: &Record<'a>) -> FilterAction;
// }
//
// struct NullFilter;
//
// impl Filter for NullFilter {
//     fn filter<'a>(&self, _rec: &Record<'a>) -> FilterAction {
//         FilterAction::Neutral
//     }
// }
//
// pub type Error = ::std::io::Error;
//
// pub trait Encode2 : Encode + ToEncodeBox {}
//
// impl<T: Encode + ToEncodeBox> Encode2 for T {}
//
// #[derive(Debug, Copy, Clone)]
// pub struct Meta<'a> {
//     pub name: &'static str, // TODO: Not sure this should be public.
//     pub value: &'a Encode2,
// }
//
// impl<'a> Meta<'a> {
//     #[inline]
//     pub fn new(name: &'static str, value: &'a Encode2) -> Meta<'a> {
//         Meta {
//             name: name,
//             value: value,
//         }
//     }
// }
//
// #[derive(Debug)]
// pub struct MetaList<'a> {
//     prev: Option<&'a MetaList<'a>>,
//     meta: &'a [Meta<'a>],
// }
//
// impl<'a> MetaList<'a> {
//     #[inline]
//     pub fn new(meta: &'a [Meta<'a>]) -> MetaList<'a> {
//         MetaList::next(meta, None)
//     }
//
//     pub fn next(meta: &'a [Meta<'a>], prev: Option<&'a MetaList<'a>>) -> MetaList<'a> {
//         MetaList {
//             prev: prev,
//             meta: meta,
//         }
//     }
//
//     pub fn prev(&self) -> Option<&'a MetaList<'a>> {
//         self.prev
//     }
//
//     pub fn meta(&self) -> &[Meta<'a>] {
//         self.meta
//     }
// }
//
// #[derive(Debug)]
// pub struct MetaBuf {
//     name: &'static str,
//     value: Box<Encode>,
// }
//
// impl MetaBuf {
//     fn new(name: &'static str, value: Box<Encode>) -> MetaBuf {
//         MetaBuf {
//             name: name,
//             value: value,
//         }
//     }
// }
//
// impl<'a> From<&'a MetaList<'a>> for Vec<MetaBuf> {
//     fn from(val: &'a MetaList<'a>) -> Vec<MetaBuf> {
//         let mut result = Vec::with_capacity(32);
//
//         let mut node = val;
//         loop {
//             for meta in node.meta.iter().rev() {
//                 result.push(MetaBuf::new(meta.name, meta.value.to_encode_buf()));
//             }
//
//             if let Some(prev) = node.prev {
//                 node = prev;
//             } else {
//                 break;
//             }
//         }
//
//         result
//     }
// }
//
// #[macro_export]
// macro_rules! log (
//     ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*], {$($name:ident: $val:expr,)*}) => {{
//         $log.log(&$crate::Record::new($sev, line!(), module_path!(), format_args!($fmt, $($args)*),
//             &$crate::MetaList::new(&[
//                 $($crate::Meta::new(stringify!($name), &$val)),*
//             ])
//         ));
//     }};
//     ($log:ident, $sev:expr, $fmt:expr, {$($name:ident: $val:expr,)*}) => {{
//         log!($log, $sev, $fmt, [], {$($name: $val,)*})
//     }};
//     ($log:ident, $sev:expr, $fmt:expr, [$($args:tt)*]) => {{
//         log!($log, $sev, $fmt, [$($args)*], {})
//     }};
//     ($log:ident, $sev:expr, $fmt:expr, $($args:tt)*) => {{
//         log!($log, $sev, $fmt, [$($args)*], {})
//     }};
//     ($log:ident, $sev:expr, $fmt:expr) => {{
//         log!($log, $sev, $fmt, [], {})
//     }};
// );
//
// pub trait Logger : Send {
//     fn log<'a>(&self, record: &InactiveRecord<'a>);
// }
//
// trait Handler : Send + Sync {
//     fn handle(&self, rec: &mut Record);
// }
//
// #[derive(Clone)]
// struct SyncLogger {
//     handlers: Arc<Vec<Box<Handler>>>,
//     severity: Arc<AtomicI32>,
//     filter: Arc<Mutex<Arc<Box<Filter>>>>,
// }
//
// impl SyncLogger {
//     fn new(handlers: Vec<Box<Handler>>) -> SyncLogger {
//         SyncLogger {
//             handlers: Arc::new(handlers),
//             severity: Arc::new(AtomicI32::new(0)),
//             filter: Arc::new(Mutex::new(Arc::new(box NullFilter))),
//         }
//     }
//
//     fn severity(&self, val: Severity) {
//         self.severity.store(val, Ordering::Release);
//     }
//
//     fn filter<F>(&self, filter: F)
//         where F: Filter + 'static
//     {
//         (*self.filter.lock().unwrap()) = Arc::new(box filter);
//     }
// }
//
// trait Mutant : Send + Sync {
//     fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record));
// }
//
// struct FalloutMutant;
//
// impl FalloutMutant {
//     fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record)) {
//         let v = 42;
//         let m = &[Meta::new("a1", &v)];
//         // let meta = MetaList::next(m, Some(rec.meta));
//         // let mut rec2 = *rec;
//         // rec2.meta = &meta;
//
//         // f(&mut rec2)
//         f(rec)
//     }
// }
//
// trait Layout {}
//
// struct SomeHandler {
//     // layout: Box<Layout>,
//     mutants: Arc<Vec<Box<Mutant>>>,
//     // appenders: Vec<Box<Appender>>,
// }
//
// impl SomeHandler {
//     fn handle_<'a>(&self, rec: &mut Record<'a>, mutants: &[Box<Mutant>]) {
//         println!("{:?}: {:?}", mutants.len(), rec);
//
//         match mutants.iter().next() {
//             Some(mutant) => {
//                 mutant.mutate(rec, &|rec| {
//                     self.handle_(rec, &mutants[1..])
//                 })
//             }
//             None => {
//                 let mut wr: Vec<u8> = Vec::new();
//                 // self.layout.format(rec, &mut wr);
//             }
//         }
//     }
// }
//
// impl Handler for SomeHandler {
//     fn handle<'a>(&self, rec: &mut Record<'a>) {
//         self.handle_(rec, &self.mutants[..])
//     }
// }
//
// impl Logger for SyncLogger {
//     fn log<'a>(&self, record: &InactiveRecord<'a>) {
//         if record.severity() >= self.severity.load(Ordering::Relaxed) {
//             let record = record.activate();
//             let filter = (*self.filter.lock().unwrap()).clone();
//
//             match filter.filter(&record) {
//                 FilterAction::Deny => {}
//                 FilterAction::Accept | FilterAction::Neutral => {
//                     for handler in self.handlers.iter() {
//                         // copy record, make mut.
//                         // add new meta.
//                         // mutate(possible reset meta?)
//                     }
//                     // record.activate().
//                     // .
//                 }
//             }
//         }
//     }
// }
//
// #[derive(Debug)]
// enum Event {
//     Record(RecordBuf),
//     // Reset(Vec<Handler>),
//     // Filter(Filter),
//     Shutdown,
// }
//
// struct Inner {
//     severity: AtomicI32,
//     tx: Mutex<mpsc::Sender<Event>>,
//     thread: Option<JoinHandle<()>>,
// }
//
// impl Inner {
//     fn new(tx: mpsc::Sender<Event>, rx: mpsc::Receiver<Event>) -> Inner {
//         let thread = thread::spawn(move || {
//             for event in rx {
//                 match event {
//                     Event::Record(rec) => {
//                         // println!("{:?}", rec);
//                     }
//                     Event::Shutdown => break,
//                 }
//             }
//         });
//
//         Inner {
//             severity: AtomicI32::new(0),
//             tx: Mutex::new(tx),
//             thread: Some(thread),
//         }
//     }
// }
//
// impl Drop for Inner {
//     fn drop(&mut self) {
//         self.tx.lock().unwrap().send(Event::Shutdown).unwrap();
//         self.thread.take().unwrap().join().unwrap();
//     }
// }
//
// #[derive(Clone)]
// pub struct AsyncLogger {
//     tx: mpsc::Sender<Event>,
//     inner: Arc<Inner>,
// }
//
// impl AsyncLogger {
//     pub fn new() -> AsyncLogger {
//         let (tx, rx) = mpsc::channel();
//
//         AsyncLogger {
//             tx: tx.clone(),
//             inner: Arc::new(Inner::new(tx, rx)),
//         }
//     }
//
//     fn scoped<F: FnOnce() -> &'static str>(&self, f: F) -> Scope<F> {
//         Scope {
//             logger: self as &Logger,
//             f: f,
//         }
//     }
// }
//
// impl Logger for AsyncLogger {
//     fn log<'a>(&self, record: &InactiveRecord<'a>) {
//         if record.severity() >= self.inner.severity.load(Ordering::Relaxed) {
//             if let Err(..) = self.tx.send(Event::Record(RecordBuf::from(record.activate()))) {
//                 // TODO: Return error.
//             }
//         }
//     }
// }
//
// struct Scope<'a, F: FnOnce() -> &'static str> {
//     logger: &'a Logger,
//     f: F,
// }
//
// impl<'a, F: FnOnce() -> &'static str> Drop for Scope<'a, F> {
//     fn drop(&mut self) {
//         let l = &self.logger;
//         log!(l, 42, "fuck you");
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::{SyncLogger, AsyncLogger, Lazy, Logger};
//
//     #[cfg(feature="benchmark")]
//     use test::Bencher;
//
//     #[test]
//     fn sync_logger_send() {
//         fn checker<T: Send>(_v: T) {}
//
//         let log = SyncLogger::new(vec![]);
//         checker(log.clone());
//     }
//
//     #[test]
//     fn async_logger_send() {
//         fn checker<T: Send>(_v: T) {}
//
//         let log = AsyncLogger::new();
//         checker(log.clone());
//     }
//
//     #[test]
//     fn log_with_custom_enum() {
//         enum Severity {
//             Debug,
//             Info,
//             Warn,
//             Error,
//         }
//
//         impl From<Severity> for i32 {
//             fn from(val: Severity) -> i32 {
//                 match val {
//                     Severity::Debug => 0,
//                     Severity::Info  => 1,
//                     Severity::Warn  => 2,
//                     Severity::Error => 3,
//                 }
//             }
//         }
//         let log = SyncLogger::new(vec![]);
//
//         log!(log, Severity::Debug, "file does not exist: /var/www/favicon.ico");
//     }
//
//     #[test]
//     fn log() {
//         let log = AsyncLogger::new();
//
//         // Only severity with message.
//         log!(log, 0, "file does not exist: /var/www/favicon.ico");
//
//         // Add some meta information.
//         log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//             path: "/home",
//         });
//
//         // Delayed formatting.
//         log!(log, 0, "file does not exist: {}", "/var/www/favicon.ico");
//
//         // Alternative syntax for delayed formatting without additional meta information.
//         log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"]);
//
//         // Full syntax both with delayed formatting and meta information.
//         log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
//             flag: true,
//             path: "/home",
//             path: "/home/esafronov", // Duplicates are allowed as a stacking feature.
//             target: "core",
//             owned: "le message".to_string(),
//         });
//
//         log.scoped(move || "wow");
//     }
//
//     #[test]
//     fn log_fn() {
//         let log = AsyncLogger::new();
//         let val = true;
//
//         fn fact(n: u64) -> u64 {
//             match n {
//                 0 | 1 => 1,
//                 n => n * fact(n - 1),
//             }
//         };
//
//         // Only severity, message and meta information.
//         log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//             lazy: Lazy::new(move || { format!("lazy message of {}", val) }),
//             lazy: Lazy::new(move || val ),
//             lazy: Lazy::new(move || fact(10)),
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message(b: &mut Bencher) {
//         let log = AsyncLogger::new();
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: /var/www/favicon.ico");
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_sync(b: &mut Bencher) {
//         let log = SyncLogger::new(vec![]);
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: /var/www/favicon.ico");
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_with_meta1(b: &mut Bencher) {
//         let log = AsyncLogger::new();
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//                 path: "/home",
//             });
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_with_meta6(b: &mut Bencher) {
//         let log = AsyncLogger::new();
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: /var/www/favicon.ico", {
//                 path1: "/home1",
//                 path2: "/home2",
//                 path3: "/home3",
//                 path4: "/home4",
//                 path5: "/home5",
//                 path6: "/home6",
//             });
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_with_format_and_meta6(b: &mut Bencher) {
//         let log = AsyncLogger::new();
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
//                 flag: true,
//                 path1: "/home1",
//                 path2: "/home2",
//                 path3: "/home3",
//                 path4: "/home4",
//                 path5: "/home5",
//             });
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_with_format_and_meta6_sync(b: &mut Bencher) {
//         let log = SyncLogger::new(vec![]);
//
//         b.iter(|| {
//             log!(log, 0, "file does not exist: {}", ["/var/www/favicon.ico"], {
//                 flag: true,
//                 path1: "/home1",
//                 path2: "/home2",
//                 path3: "/home3",
//                 path4: "/home4",
//                 path5: "/home5",
//             });
//         });
//     }
//
//     #[cfg(feature="benchmark")]
//     #[bench]
//     fn bench_log_message_with_format_and_meta6_reject_sync(b: &mut Bencher) {
//         let log = SyncLogger::new(vec![]);
//
//         b.iter(|| {
//             log!(log, -1, "file does not exist: {}", ["/var/www/favicon.ico"], {
//                 flag: true,
//                 path1: "/home1",
//                 path2: "/home2",
//                 path3: "/home3",
//                 path4: "/home4",
//                 path5: "/home5",
//             });
//         });
//     }
// }

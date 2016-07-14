use std::fmt::Arguments;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use handle::Handle;
use logger::Logger;
use record::{Record, RecordBuf};

enum Event {
    Record(RecordBuf),
    Shutdown,
}

struct Inner {
    // TODO: Maybe use tx/rx connectivity to auto break the loop?
    tx: Mutex<mpsc::Sender<Event>>,
    thread: Option<JoinHandle<()>>,
}

impl Inner {
    fn new(tx: Sender<Event>, rx: Receiver<Event>, handlers: Vec<Box<Handle>>) -> Inner {
        let thread = thread::spawn(move || {
            for event in rx {
                match event {
                    Event::Record(rec) => {
                        rec.borrow_and(|rec| {
                            for handle in handlers.iter() {
                                handle.handle(rec).unwrap();
                            }
                        });
                    }
                    Event::Shutdown => break,
                }
            }
        });

        Inner {
            tx: Mutex::new(tx),
            thread: Some(thread),
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Err(..) = self.tx.lock().unwrap().send(Event::Shutdown) {
            // Ignore, but the thread should join anyway.
        }
        self.thread.take().unwrap().join().unwrap();
    }
}

// TODO: Maybe better AsyncLoggerAdaptor?
#[derive(Clone)]
pub struct ActorLogger {
    tx: Sender<Event>,
    inner: Arc<Inner>,
}

impl ActorLogger {
    pub fn new(handlers: Vec<Box<Handle>>) -> ActorLogger {
        let (tx, rx) = mpsc::channel();

        ActorLogger {
            tx: tx.clone(),
            inner: Arc::new(Inner::new(tx, rx, handlers)),
        }
    }
}

impl Logger for ActorLogger {
    fn log<'a, 'b>(&self, rec: &mut Record<'a>, args: Arguments<'b>) {
        rec.activate(args);

        if let Err(..) = self.tx.send(Event::Record(RecordBuf::from(&*rec))) {
            // TODO: Return error.
        }
    }
}

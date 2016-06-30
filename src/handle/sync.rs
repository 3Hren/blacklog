use std::rc::Rc;

use {Config, Handle, Record, Registry};

use factory::Factory;

struct SyncHandle;

impl Handle for SyncHandle {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
        unimplemented!();
    }
}

pub struct SyncHandleFactory;

impl Factory for SyncHandleFactory {
    type Item = Handle;

    fn ty() -> &'static str {
        "synchronous"
    }

    fn from(&self, _cfg: &Config, _registry: &Registry) -> Result<Box<Handle>, Box<::std::error::Error>> {
        Ok(box SyncHandle)
    }
}

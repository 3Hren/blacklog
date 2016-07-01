use {Config, Handle, Record, Registry};

use layout::Layout;
use output::Output;

use factory::Factory;

struct SyncHandle {
    layout: Box<Layout>,
    outputs: Vec<Box<Output>>,
}

impl Handle for SyncHandle {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
        let mut wr: Vec<u8> = Vec::new();
        self.layout.format(rec, &mut wr).unwrap();

        for output in &self.outputs {
            output.write(rec, &wr)?;
        }

        Ok(())
    }
}

pub struct SyncHandleFactory;

impl Factory for SyncHandleFactory {
    type Item = Handle;

    fn ty() -> &'static str {
        "synchronous"
    }

    fn from(&self, cfg: &Config, registry: &Registry) -> Result<Box<Handle>, Box<::std::error::Error>> {
        let layout = registry.layout(cfg.find("layout").unwrap())?;

        let outputs = cfg.find("outputs")
            .expect("section \"outputs\" is required")
            .as_array()
            .expect("section \"outputs\" must be an array")
            .iter()
            .map(|o| registry.output(o))
            .collect()?;

        let res = SyncHandle {
            layout: layout,
            outputs: outputs,
        };

        Ok(box res)
    }
}

use {Config, Handle, Record, Registry};

use layout::Layout;
use output::Output;

use factory::Factory;

pub struct SyncHandle {
    layout: Box<Layout>,
    outputs: Vec<Box<Output>>,
}

impl Handle for SyncHandle {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
        let mut wr = Vec::new();
        self.layout.format(rec, &mut wr).unwrap();

        for output in &self.outputs {
            output.write(rec, &wr)?;
        }

        Ok(())
    }
}

impl Factory for SyncHandle {
    type Item = Handle;

    fn ty() -> &'static str {
        "sync"
    }

    fn from(cfg: &Config, registry: &Registry) -> Result<Box<Handle>, Box<::std::error::Error>> {
        let layout = registry.layout(cfg.find("layout").unwrap())?;

        let outputs = cfg.find("outputs")
            .ok_or("section \"outputs\" is required")?
            .as_array()
            .ok_or("section \"outputs\" must be an array")?
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

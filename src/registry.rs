use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use {Handle, Layout, Output};

use factory::Factory;
use layout::PatternLayoutFactory;
use output::TerminalOutputFactory;
use handle::SyncHandleFactory;

pub type Config = Value;

#[derive(Default)]
pub struct Registry {
    layouts: HashMap<&'static str, Box<Factory<Item=Layout>>>,
    outputs: HashMap<&'static str, Box<Factory<Item=Output>>>,
    handles: HashMap<&'static str, Box<Factory<Item=Handle>>>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut result = Registry::default();
        result.layouts.insert(PatternLayoutFactory::ty(), box PatternLayoutFactory);

        result.outputs.insert(TerminalOutputFactory::ty(), box TerminalOutputFactory);

        result.handles.insert(SyncHandleFactory::ty(), box SyncHandleFactory);

        result
    }

    pub fn init(&self) {

    }

    pub fn layout(&self, cfg: &Config) -> Result<Box<Layout>, Box<Error>> {
        let ty = Registry::ty(cfg)?;

        self.layouts.get(ty)
            .ok_or_else(|| format!("layout \"{}\" not found", ty))?
            .from(cfg, self)
    }

    pub fn output(&self, cfg: &Config) -> Result<Box<Output>, Box<Error>> {
        let ty = Registry::ty(cfg)?;

        self.outputs.get(ty)
            .ok_or_else(|| format!("handle \"{}\" not found", ty))?
            .from(cfg, self)
    }

    pub fn handle(&self, cfg: &Config) -> Result<Box<Handle>, Box<Error>> {
        let ty = Registry::ty(cfg)?;

        self.handles.get(ty)
            .ok_or_else(|| format!("handle \"{}\" not found", ty))?
            .from(cfg, self)
    }

    // TODO: fn filter(&self, cfg: &Config) -> Result<Box<Filter>, Box<Error>>;
    // TODO: fn mutant(&self, cfg: &Config) -> Result<Box<Mutant>, Box<Error>>;
    // TODO: fn logger(&self, cfg: &Config) -> Result<Box<Logger>, Box<Error>>;

    // TODO: Give a way to register user-defined components.
    fn ty(cfg: &Config) -> Result<&str, &str> {
        cfg.find("type")
            .ok_or("field \"type\" is required")?
            .as_string()
            .ok_or("field \"type\" must be a string")
    }
}

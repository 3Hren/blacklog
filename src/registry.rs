use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use {Layout, Output};

use factory::Factory;
use layout::PatternLayoutFactory;
use output::TerminalOutputFactory;

pub type Config = Value;

#[derive(Default)]
pub struct Registry {
    layouts: HashMap<&'static str, Box<Factory<Item=Layout>>>,
    outputs: HashMap<&'static str, Box<Factory<Item=Output>>>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut result = Registry::default();
        result.layouts.insert(PatternLayoutFactory::ty(), box PatternLayoutFactory);

        result.outputs.insert(TerminalOutputFactory::ty(), box TerminalOutputFactory);

        result
    }

    pub fn layout(&self, cfg: &Config) -> Result<Box<Layout>, Box<Error>> {
        let ty = Registry::ty(cfg)?;

        self.layouts.get(ty)
            .ok_or(format!("layout with \"{}\" not found", ty))?
            .from(cfg)
    }

    pub fn output(&self, cfg: &Config) -> Result<Box<Output>, Box<Error>> {
        let ty = Registry::ty(cfg)?;

        self.outputs.get(ty)
            .ok_or("...")?
            .from(cfg)
    }

    // TODO: fn logger(&self, cfg: &Config) -> Result<Box<Logger>, Error>;
    // TODO: fn handle(&self, cfg: &Config) -> Result<Box<Handle>, Error>;
    // TODO: fn filter(&self, cfg: &Config) -> Result<Box<Filter>, Error>;
    // TODO: fn mutant(&self, cfg: &Config) -> Result<Box<Mutant>, Error>;

    // TODO: Give a way to register user-defined components.
    fn ty(cfg: &Config) -> Result<&str, &str> {
        cfg.find("type")
            .ok_or("field \"type\" is required")?
            .as_string()
            .ok_or("field \"type\" must be a string")
    }
}

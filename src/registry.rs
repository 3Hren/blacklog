use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use {Layout, Output};

use factory::Factory;
use layout::PatternLayoutFactory;

pub type Config = Value;

#[derive(Default)]
pub struct Registry {
    layouts: HashMap<&'static str, Box<Factory<Item=Layout>>>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut result = Registry::default();
        result.layouts.insert(PatternLayoutFactory::ty(), box PatternLayoutFactory);

        result
    }
    // fn logger(cfg: Config) -> Result<Box<Logger>, Error>;
    // fn handle(cfg: Config) -> Result<Box<Handle>, Error>;
    pub fn layout(&self, cfg: &Config) -> Result<Box<Layout>, Box<Error>> {
        let ty = cfg.find("type")
            .ok_or("field \"type\" is required")?
            .as_string()
            .ok_or("field \"type\" must be a string")?;

        self.layouts.get(ty)
            .ok_or(format!("layout with \"{}\" not found", ty))?
            .from(cfg)
    }
    // fn filter(cfg: Config) -> Result<Box<Filter>, Error>;
    // fn mutant(cfg: Config) -> Result<Box<Mutant>, Error>;
    pub fn output(&self, cfg: &Config) -> Result<Box<Output>, Box<Error>> {
        unimplemented!();
    }
}

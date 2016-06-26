use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use {Layout, LayoutFactory};

use layout::PatternLayoutFactory;

pub type Config = Value;

#[derive(Default)]
pub struct Registry {
    layouts: HashMap<&'static str, Box<LayoutFactory>>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut result = Registry::default();
        result.layouts.insert(PatternLayoutFactory::ty(), box PatternLayoutFactory);

        result
    }
    // fn logger(cfg: Config) -> Result<Box<Logger>, Error>;
    // fn handle(cfg: Config) -> Result<Box<Handle>, Error>;
    pub fn layout(cfg: Config) -> Result<Box<Layout>, Box<Error>> {
        unimplemented!();
    }
    // fn filter(cfg: Config) -> Result<Box<Filter>, Error>;
    // fn mutant(cfg: Config) -> Result<Box<Mutant>, Error>;
    // fn appender(cfg: Config) -> Result<Box<Appender>, Error>;
}

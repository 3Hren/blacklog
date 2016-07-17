use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use {Handle, Layout, Logger, Output};

use factory::Factory;
use layout::{PatternLayout};
use logger::{SyncLogger};
use output::{FileOutput, NullOutput, Term};
use handle::{SyncHandle};

pub type Config = Value;

type FnFactory<T> = Fn(&Config, &Registry) -> Result<Box<T>, Box<Error>>;

#[derive(Default)]
pub struct Registry {
    layouts: HashMap<&'static str, Box<FnFactory<Layout>>>,
    outputs: HashMap<&'static str, Box<FnFactory<Output>>>,
    handles: HashMap<&'static str, Box<FnFactory<Handle>>>,
    loggers: HashMap<&'static str, Box<FnFactory<Logger>>>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut result = Registry::default();

        result.add_layout::<PatternLayout>();

        result.add_output::<FileOutput>();
        result.add_output::<NullOutput>();
        result.add_output::<Term>();

        result.add_handle::<SyncHandle>();

        result.add_logger::<SyncLogger>();

        result
    }

    fn add_layout<T: Factory<Item=Layout> + 'static>(&mut self) {
        Registry::add_component::<T, Layout>(&mut self.layouts);
    }

    fn add_output<T: Factory<Item=Output> + 'static>(&mut self) {
        Registry::add_component::<T, Output>(&mut self.outputs);
    }

    fn add_handle<T: Factory<Item=Handle> + 'static>(&mut self) {
        Registry::add_component::<T, Handle>(&mut self.handles);
    }

    fn add_logger<T: Factory<Item=Logger> + 'static>(&mut self) {
        Registry::add_component::<T, Logger>(&mut self.loggers);
    }

    fn add_component<T, C: ?Sized>(map: &mut HashMap<&'static str, Box<FnFactory<C>>>)
        where T: Factory<Item=C> + 'static
    {
        map.insert(T::ty(), box |cfg, registry| {
            T::from(cfg, registry)
        });
    }

    pub fn layout(&self, cfg: &Config) -> Result<Box<Layout>, Box<Error>> {
        let ty = Registry::ty(cfg)?;
        let func = self.layouts.get(ty)
            .ok_or_else(|| format!("layout \"{}\" not found", ty))?;
        func(cfg, self)
    }

    pub fn output(&self, cfg: &Config) -> Result<Box<Output>, Box<Error>> {
        let ty = Registry::ty(cfg)?;
        let func = self.outputs.get(ty)
            .ok_or_else(|| format!("output \"{}\" not found", ty))?;
        func(cfg, self)
    }

    pub fn handle(&self, cfg: &Config) -> Result<Box<Handle>, Box<Error>> {
        let ty = Registry::ty(cfg)?;
        let func = self.handles.get(ty)
            .ok_or_else(|| format!("handle \"{}\" not found", ty))?;
        func(cfg, self)
    }

    pub fn logger(&self, cfg: &Config) -> Result<Box<Logger>, Box<Error>> {
        let ty = Registry::ty(cfg)?;
        let func = self.loggers.get(ty)
            .ok_or_else(|| format!("logger \"{}\" not found", ty))?;
        func(cfg, self)
    }

    // TODO: fn filter(&self, cfg: &Config) -> Result<Box<Filter>, Box<Error>>;
    // TODO: fn mutant(&self, cfg: &Config) -> Result<Box<Mutant>, Box<Error>>;

    // TODO: Give a way to register user-defined components.
    fn ty(cfg: &Config) -> Result<&str, &str> {
        cfg.find("type")
            .ok_or("field \"type\" is required")?
            .as_string()
            .ok_or("field \"type\" must be a string")
    }
}

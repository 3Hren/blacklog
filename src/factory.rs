use {Config, Registry};

pub use ::std::error::Error;

pub trait Factory {
    type Item: ?Sized;

    /// Returns type as a string that is used mainly for concrete component identification.
    fn ty() -> &'static str where Self: Sized;

    /// Constructs a new component by configuring it with the given config.
    // TODO: Maybe replace with a trait?
    fn from(&self, cfg: &Config, registry: &Registry) -> Result<Box<Self::Item>, Box<Error>>;
}

pub use ::std::error::Error;

use {Config, Registry};

pub trait Factory {
    type Item: ?Sized;

    /// Returns type as a string that is used mainly for concrete component identification.
    fn ty() -> &'static str where Self: Sized;

    /// Constructs a new component by configuring it with the given config.
    fn from(cfg: &Config, registry: &Registry) -> Result<Box<Self::Item>, Box<Error>>
        where Self: Sized;
}

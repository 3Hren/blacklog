use std::sync::Arc;

use {Format, Formatter, IntoBoxedFormat};

use meta::format::FormatInto;

pub type Error = ::std::io::Error;

/// Represents a clonable wrapper over user-defined function, making it a valid meta information.
///
/// The actual meta value will be evaluated each time on demand.
#[derive(Clone)]
pub struct FnMeta<F>(Arc<Box<F>>);

impl<F, R> FnMeta<F>
    where F: Fn() -> R + Send + Sync,
          R: Format
{
    /// Creates a new FnMeta by wrapping the given function.
    pub fn new(f: F) -> FnMeta<F> {
        FnMeta(Arc::new(box f))
    }
}

impl<F, R> Format for FnMeta<F>
    where F: Fn() -> R + Send + Sync,
          R: Format
{
    fn format(&self, format: &mut Formatter) -> Result<(), Error> {
        self.0().format(format)
    }
}

impl<F, R> IntoBoxedFormat for FnMeta<F>
    where F: Fn() -> R + Send + Sync + 'static,
          R: Format
{
    fn to_boxed_format(&self) -> Box<FormatInto> {
        box FnMeta(self.0.clone())
    }
}

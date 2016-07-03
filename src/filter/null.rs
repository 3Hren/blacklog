use record::Record;

use super::{Filter, FilterAction};

/// A filter which is neutral to all records.
///
/// This is the default filter for all components that support filtering.
pub struct NullFilter;

impl Filter for NullFilter {
    fn filter(&self, _rec: &Record) -> FilterAction {
        FilterAction::Neutral
    }
}

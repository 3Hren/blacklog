use record::Record;

use super::Filter;

/// A filter that accepts all records.
///
/// This is the default filter for all components that support filtering.
struct NullFilter;

impl Filter for NullFilter {
    fn filter(&self, _rec: &Record) -> FilterAction {
        FilterAction::Neutral
    }
}

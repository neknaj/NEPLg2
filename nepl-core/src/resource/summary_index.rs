extern crate alloc;

use alloc::collections::BTreeMap;

pub(super) trait FunctionSummary {
    fn function_name(&self) -> &str;
}

pub(super) struct SummaryIndex<'a, T> {
    entries: &'a [T],
    by_function: BTreeMap<&'a str, usize>,
}

impl<'a, T> SummaryIndex<'a, T>
where
    T: FunctionSummary,
{
    pub(super) fn new(entries: &'a [T]) -> Self {
        let mut by_function = BTreeMap::new();
        for (index, entry) in entries.iter().enumerate() {
            by_function.insert(entry.function_name(), index);
        }
        Self {
            entries,
            by_function,
        }
    }

    pub(super) fn get(&self, function: &str) -> Option<&'a T> {
        self.by_function
            .get(function)
            .and_then(|index| self.entries.get(*index))
    }
}

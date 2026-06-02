extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

pub(super) trait FunctionSummary {
    fn function_name(&self) -> &str;
}

pub(super) struct SummaryIndex<'a, T> {
    entries: &'a [T],
    by_function: SummaryIndexMap<'a>,
}

enum SummaryIndexMap<'a> {
    BorrowedNames(BTreeMap<&'a str, usize>),
    StableNames(&'a BTreeMap<String, usize>),
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
            by_function: SummaryIndexMap::BorrowedNames(by_function),
        }
    }

    pub(super) fn get(&self, function: &str) -> Option<&'a T> {
        let index = match &self.by_function {
            SummaryIndexMap::BorrowedNames(by_function) => by_function.get(function),
            SummaryIndexMap::StableNames(by_function) => by_function.get(function),
        };
        index.and_then(|index| self.entries.get(*index))
    }
}

/// 固定点計算中に、summary の関数名から `Vec` 上の位置を引くための索引。
///
/// `SummaryIndex::new` は不変 slice から軽量な検索 view を作るための型だが、
/// Resource summary の固定点計算では同じ summary 集合に対して多数回 lookup を行う。
/// この索引は summary 本体を所有せず、関数名と位置だけを保持して更新時に差し替える。
/// これにより、検査結果の意味を変えずに、反復ごとの `BTreeMap` 再構築を避けられる。
pub(super) struct SummaryNameIndex {
    by_function: BTreeMap<String, usize>,
}

impl SummaryNameIndex {
    pub(super) fn new() -> Self {
        Self {
            by_function: BTreeMap::new(),
        }
    }

    pub(super) fn from_entries<T>(entries: &[T]) -> Self
    where
        T: FunctionSummary,
    {
        let mut index = Self::new();
        for (position, entry) in entries.iter().enumerate() {
            index
                .by_function
                .insert(entry.function_name().to_string(), position);
        }
        index
    }

    pub(super) fn as_summary_index<'a, T>(&'a self, entries: &'a [T]) -> SummaryIndex<'a, T>
    where
        T: FunctionSummary,
    {
        SummaryIndex {
            entries,
            by_function: SummaryIndexMap::StableNames(&self.by_function),
        }
    }

    pub(super) fn position(&self, function: &str) -> Option<usize> {
        self.by_function.get(function).copied()
    }

    pub(super) fn insert_at_end(&mut self, function: &str, index: usize) {
        self.by_function.insert(function.to_string(), index);
    }

    pub(super) fn remove_and_shift(&mut self, function: &str, removed_index: usize) {
        self.by_function.remove(function);
        for index in self.by_function.values_mut() {
            if *index > removed_index {
                *index -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec;

    struct TestSummary {
        function: String,
        value: usize,
    }

    impl FunctionSummary for TestSummary {
        fn function_name(&self) -> &str {
            &self.function
        }
    }

    #[test]
    fn summary_name_index_tracks_append_and_remove() {
        let mut summaries = vec![
            TestSummary {
                function: "a".into(),
                value: 1,
            },
            TestSummary {
                function: "b".into(),
                value: 2,
            },
        ];
        let mut names = SummaryNameIndex::from_entries(&summaries);
        {
            let index = names.as_summary_index(&summaries);
            assert_eq!(index.get("a").map(|summary| summary.value), Some(1));
            assert_eq!(index.get("b").map(|summary| summary.value), Some(2));
            assert_eq!(index.get("c").map(|summary| summary.value), None);
        }

        names.insert_at_end("c", summaries.len());
        summaries.push(TestSummary {
            function: "c".into(),
            value: 3,
        });
        {
            let index = names.as_summary_index(&summaries);
            assert_eq!(index.get("c").map(|summary| summary.value), Some(3));
        }

        summaries.remove(0);
        names.remove_and_shift("a", 0);
        {
            let index = names.as_summary_index(&summaries);
            assert_eq!(index.get("a").map(|summary| summary.value), None);
            assert_eq!(index.get("b").map(|summary| summary.value), Some(2));
            assert_eq!(index.get("c").map(|summary| summary.value), Some(3));
        }
    }
}

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::model::{AggregateKind, Place};
use super::place_utils::construct_aggregate_field_place;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FunctionAliasTable {
    entries: Vec<FunctionAliasEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionAliasEntry {
    place: Place,
    functions: Vec<String>,
}

impl FunctionAliasTable {
    pub(super) fn functions(&self, place: &Place) -> &[String] {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.functions.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn set_alias(&mut self, place: &Place, function: String) {
        self.set_functions(place, vec![function]);
    }

    pub(super) fn copy_alias(&mut self, source: &Place, target: &Place) {
        let functions = self.functions(source).to_vec();
        if !functions.is_empty() {
            self.set_functions(target, functions);
        } else {
            self.clear_alias(target);
        }
    }

    pub(super) fn merge_paths(paths: &[FunctionAliasTable]) -> Self {
        let mut out = FunctionAliasTable::default();
        for path in paths {
            for entry in &path.entries {
                out.union_functions(&entry.place, entry.functions.iter().cloned());
            }
        }
        out
    }

    fn set_functions(&mut self, place: &Place, functions: Vec<String>) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.place == *place) {
            entry.functions = dedupe_functions(functions);
            return;
        }
        self.entries.push(FunctionAliasEntry {
            place: place.clone(),
            functions: dedupe_functions(functions),
        });
    }

    pub(super) fn clear_alias(&mut self, place: &Place) {
        self.entries.retain(|entry| entry.place != *place);
    }

    fn union_functions<I>(&mut self, place: &Place, functions: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut merged = self.functions(place).to_vec();
        for function in functions {
            if !merged.contains(&function) {
                merged.push(function);
            }
        }
        if !merged.is_empty() {
            self.set_functions(place, merged);
        }
    }
}

pub(super) fn construct_function_alias_fields(
    function_aliases: &mut FunctionAliasTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        function_aliases.copy_alias(input, &field);
    }
}

fn dedupe_functions(functions: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for function in functions {
        if !out.contains(&function) {
            out.push(function);
        }
    }
    out
}

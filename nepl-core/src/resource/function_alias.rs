use alloc::vec;
use alloc::vec::Vec;

use crate::function_identity::FunctionValueIdentity;

use super::model::{AggregateKind, Place, ResourceFunctionValueKind};
use super::place_utils::construct_aggregate_field_place;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FunctionAliasTable {
    entries: Vec<FunctionAliasEntry>,
}

/// Resource IR 上で値として運ばれている関数の alias。
///
/// `memo_call` で作られる memoized function value は、同じ `FunctionValueIdentity` を
/// 指していても plain function pointer とは private cache boundary が異なる。alias
/// 解析は indirect call の候補を伝播するため、identity だけでなく value kind も保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FunctionValueAlias {
    pub(super) identity: FunctionValueIdentity,
    pub(super) value_kind: ResourceFunctionValueKind,
}

impl FunctionValueAlias {
    pub(super) fn symbol(&self) -> &str {
        self.identity.symbol()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionAliasEntry {
    place: Place,
    functions: Vec<FunctionValueAlias>,
}

impl FunctionAliasTable {
    pub(super) fn functions(&self, place: &Place) -> &[FunctionValueAlias] {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.functions.as_slice())
            .unwrap_or(&[])
    }

    /// 現行 summary index が使う underlying function symbol を重複なしで返す。
    ///
    /// plain function value と memoized function value は alias としては別物だが、sealed
    /// backend が入るまでは同じ function summary を参照する。既存 summary consumer は
    /// kind をまだ解釈できないため、同じ symbol を二重適用しない境界をここへ集約する。
    pub(super) fn function_symbols(&self, place: &Place) -> Vec<&str> {
        let mut out = Vec::new();
        for function in self.functions(place) {
            let symbol = function.symbol();
            if !out.contains(&symbol) {
                out.push(symbol);
            }
        }
        out
    }

    pub(super) fn set_alias(
        &mut self,
        place: &Place,
        function: FunctionValueIdentity,
        value_kind: ResourceFunctionValueKind,
    ) {
        self.set_functions(
            place,
            vec![FunctionValueAlias {
                identity: function,
                value_kind,
            }],
        );
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

    fn set_functions(&mut self, place: &Place, functions: Vec<FunctionValueAlias>) {
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
        I: IntoIterator<Item = FunctionValueAlias>,
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

fn dedupe_functions(functions: Vec<FunctionValueAlias>) -> Vec<FunctionValueAlias> {
    let mut out = Vec::new();
    for function in functions {
        if !out.contains(&function) {
            out.push(function);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::ast::Effect;
    use crate::function_identity::FunctionValueIdentity;
    use crate::types::TypeId;

    use super::super::model::{Place, ResourceFunctionValueKind};
    use super::FunctionAliasTable;

    #[test]
    fn function_alias_merge_keeps_plain_and_memoized_kinds() {
        let place = place("callback");
        let identity = identity("same_target");
        let mut plain_path = FunctionAliasTable::default();
        plain_path.set_alias(&place, identity.clone(), ResourceFunctionValueKind::Plain);
        let mut memoized_path = FunctionAliasTable::default();
        memoized_path.set_alias(
            &place,
            identity.clone(),
            ResourceFunctionValueKind::Memoized,
        );

        let merged = FunctionAliasTable::merge_paths(&[plain_path, memoized_path]);
        let aliases = merged.functions(&place);

        assert_eq!(aliases.len(), 2);
        assert!(aliases.iter().any(|alias| {
            alias.identity == identity && alias.value_kind == ResourceFunctionValueKind::Plain
        }));
        assert!(aliases.iter().any(|alias| {
            alias.identity == identity && alias.value_kind == ResourceFunctionValueKind::Memoized
        }));
    }

    #[test]
    fn function_alias_copy_preserves_memoized_kind() {
        let source = place("source");
        let target = place("target");
        let identity = identity("memoized_target");
        let mut aliases = FunctionAliasTable::default();
        aliases.set_alias(
            &source,
            identity.clone(),
            ResourceFunctionValueKind::Memoized,
        );

        aliases.copy_alias(&source, &target);

        assert_eq!(
            aliases.functions(&target),
            &[super::FunctionValueAlias {
                identity,
                value_kind: ResourceFunctionValueKind::Memoized
            }]
        );
    }

    #[test]
    fn function_alias_symbols_deduplicate_plain_and_memoized_identity() {
        let place = place("callback");
        let identity = identity("same_target");
        let mut plain_path = FunctionAliasTable::default();
        plain_path.set_alias(&place, identity.clone(), ResourceFunctionValueKind::Plain);
        let mut memoized_path = FunctionAliasTable::default();
        memoized_path.set_alias(
            &place,
            identity.clone(),
            ResourceFunctionValueKind::Memoized,
        );
        let merged = FunctionAliasTable::merge_paths(&[plain_path, memoized_path]);

        assert_eq!(merged.function_symbols(&place), vec!["same_target"]);
    }

    fn place(name: &str) -> Place {
        Place::local(String::from(name), TypeId(0))
    }

    fn identity(name: &str) -> FunctionValueIdentity {
        FunctionValueIdentity::new(String::from(name), None, TypeId(0), Effect::Pure, vec![])
    }
}

extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, ResourceFunction, ResourceModule};
use super::owner_summary_leaf::owner_leaf_places;
use super::summary_dependency::ResourceSummaryDependencyGraph;
use crate::types::TypeCtx;

/// Owner return summary の固定点計算に入れる関数を保守的に絞る。
///
/// summary は caller が callee の owner return / consume / host-memory contract を
/// 適用するための情報であり、単なる scalar-only helper には facts が発生しない。
/// ここでは owner leaf を運ぶ public signature と、raw memory / raw view / external
/// memory contract を直接作る operation だけを relevant に残す。判定に含めた operation
/// は通常の summary engine で再検査されるため、この関数は「不要と断定できるもの」を
/// worklist から外すだけで、summary facts 自体を合成しない。
pub(super) fn owner_summary_relevant_functions(
    module: &ResourceModule,
    types: &TypeCtx,
    dependency_graph: &ResourceSummaryDependencyGraph,
) -> Vec<bool> {
    module
        .functions
        .iter()
        .enumerate()
        .map(|(function_index, function)| {
            owner_summary_relevant_function(types, function, dependency_graph, function_index)
        })
        .collect()
}

fn owner_summary_relevant_function(
    types: &TypeCtx,
    function: &ResourceFunction,
    dependency_graph: &ResourceSummaryDependencyGraph,
    function_index: usize,
) -> bool {
    function_signature_carries_owner_summary_facts(types, function)
        || dependency_graph.has_direct_owner_summary_op(function_index)
}

fn function_signature_carries_owner_summary_facts(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    function
        .params
        .iter()
        .any(|param| place_type_has_owner_leaf(types, &param.place))
        || place_type_has_owner_leaf(types, &Place::local("__return".into(), function.result))
}

fn place_type_has_owner_leaf(types: &TypeCtx, place: &Place) -> bool {
    !owner_leaf_places(types, place).is_empty()
}

#[cfg(test)]
#[path = "owner_summary_relevance_tests.rs"]
mod tests;

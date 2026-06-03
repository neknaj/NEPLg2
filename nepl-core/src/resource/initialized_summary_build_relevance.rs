extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_summary_seed::summary_input_type_may_seed_raw_address_alias;
use super::model::{ResourceFunction, ResourceModule};
use super::summary_dependency::ResourceSummaryDependencyGraph;

#[cfg(test)]
pub(super) fn raw_cell_initialization_summary_relevance_with_graph(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    dependency_graph: &ResourceSummaryDependencyGraph,
) -> Vec<bool> {
    raw_cell_initialization_summary_relevance_with_dependencies(
        module,
        types,
        raw_alias_summaries,
        dependency_graph,
        None,
    )
}

pub(super) fn raw_cell_initialization_call_boundary_summary_relevance_with_graph(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    dependency_graph: &ResourceSummaryDependencyGraph,
) -> Vec<bool> {
    raw_cell_initialization_summary_relevance_with_dependencies(
        module,
        types,
        raw_alias_summaries,
        dependency_graph,
        Some(dependency_graph.raw_init_dependents()),
    )
}

fn raw_cell_initialization_summary_relevance_with_dependencies(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    dependency_graph: &ResourceSummaryDependencyGraph,
    consumer_edges: Option<&[Vec<usize>]>,
) -> Vec<bool> {
    let dependencies = dependency_graph.raw_init_dependencies();
    let signature_relevant = module
        .functions
        .iter()
        .map(|function| function_raw_cell_initialization_signature_relevant(types, function))
        .collect::<Vec<_>>();
    let mut relevant = module
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            signature_relevant[index]
                && function_has_raw_cell_initialization_summary_consumer(consumer_edges, index)
                && (raw_alias_summaries.get(&function.name).is_some()
                    || dependency_graph.has_direct_raw_initialization_summary_op(index))
        })
        .collect::<Vec<_>>();
    let mut changed = true;
    while changed {
        changed = false;
        for (index, function_dependencies) in dependencies.iter().enumerate() {
            if relevant[index]
                || !signature_relevant[index]
                || !function_has_raw_cell_initialization_summary_consumer(consumer_edges, index)
            {
                continue;
            }
            if function_dependencies
                .iter()
                .any(|dependency| relevant[*dependency])
            {
                relevant[index] = true;
                changed = true;
            }
        }
    }
    relevant
}

fn function_has_raw_cell_initialization_summary_consumer(
    consumer_edges: Option<&[Vec<usize>]>,
    function_index: usize,
) -> bool {
    // raw-init summary は call 境界で parameter / return の raw-cell facts を再生するための
    // 要約である。consumer_edges が渡された実コンパイル経路では、内部 caller がいない
    // 関数を固定点から外し、その本体は final initialized check に任せる。テスト用の直接
    // builder 経路では None を渡して、summary builder 単体の性質を従来通り検査できる。
    consumer_edges
        .map(|edges| {
            edges
                .get(function_index)
                .is_some_and(|dependents| !dependents.is_empty())
        })
        .unwrap_or(true)
}

fn function_raw_cell_initialization_signature_relevant(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    // raw initialization summary は raw address / byte-range / release requirement を
    // call 境界で受け渡すための要約である。`i32` は raw address の表現にも使われるが、
    // すべての整数関数を summary 対象にすると、普通の算術処理まで raw memory 解析へ
    // 巻き込んでしまう。まず公開可能な型を持つ関数だけを候補にし、実際の raw alias
    // summary・raw memory 系 op・関連 callee から到達できるものだけを計算対象にする。
    summary_input_type_may_seed_raw_address_alias(types, function.result)
        || function
            .params
            .iter()
            .any(|param| place_may_seed_raw_initialization_summary(types, &param.place))
}

fn place_may_seed_raw_initialization_summary(types: &TypeCtx, place: &super::model::Place) -> bool {
    summary_input_type_may_seed_raw_address_alias(types, place.ty)
        || reference_target_type(types, place.ty).is_some_and(|target_ty| {
            summary_input_type_may_seed_raw_address_alias(types, target_ty)
        })
}

pub(super) fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

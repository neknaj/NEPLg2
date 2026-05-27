extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationFunctionSummaryIndex,
};
use super::initialized_summary_cells::collect_return_initialized_raw_cells;
use super::initialized_summary_param_byte_ranges::collect_param_initialized_raw_byte_ranges;
use super::initialized_summary_param_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_release_build::collect_param_release_requirements_from_ops;
use super::initialized_summary_return_byte_ranges::collect_return_initialized_raw_byte_ranges;
use super::initialized_summary_seed::{
    seed_summary_input_place, summary_input_type_may_seed_raw_address_alias,
};
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator};
use super::place_utils::reference_target_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::summary_dependency::build_function_summary_dependencies;
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;

pub(super) fn compute_raw_cell_initialization_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
) -> Vec<RawCellInitializationFunctionSummary> {
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let relevant =
        raw_cell_initialization_summary_relevance(module, types, &raw_alias_summary_index);
    let mut worklist = SummaryWorklist::new_filtered(module, relevant);
    let mut summaries = Vec::new();
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(&summaries);
        let summary = function_raw_cell_initialization_summary(
            function,
            types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
        );
        function_start.log("raw_init_summary", function);
        if update_raw_cell_initialization_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_init_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn raw_cell_initialization_summary_relevance(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
) -> Vec<bool> {
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
                && (raw_alias_summaries.get(&function.name).is_some()
                    || function_has_direct_raw_initialization_summary_op(function))
        })
        .collect::<Vec<_>>();
    let dependencies = build_function_summary_dependencies(module);
    let mut changed = true;
    while changed {
        changed = false;
        for (index, function_dependencies) in dependencies.iter().enumerate() {
            if relevant[index] || !signature_relevant[index] {
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

fn function_has_direct_raw_initialization_summary_op(function: &ResourceFunction) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_have_direct_raw_initialization_summary_op(&block.ops))
}

fn ops_have_direct_raw_initialization_summary_op(ops: &[super::model::ResourceOp]) -> bool {
    ops.iter().any(op_has_direct_raw_initialization_summary_op)
}

fn op_has_direct_raw_initialization_summary_op(op: &super::model::ResourceOp) -> bool {
    match op {
        // Collection slot ops are not raw-memory instructions themselves, but the
        // initialized-cell facts they create are consumed through the same raw init
        // summary boundary at call sites. Treat them as direct triggers so helper
        // functions without explicit RawMemory ops are not pruned out.
        super::model::ResourceOp::RawMemory { .. }
        | super::model::ResourceOp::RawAddressAlias { .. }
        | super::model::ResourceOp::RawAddressView { .. }
        | super::model::ResourceOp::StorageOrigin { .. }
        | super::model::ResourceOp::IndirectCall { .. }
        | super::model::ResourceOp::CollectionSlotLifecycle { .. }
        | super::model::ResourceOp::CollectionStorageRelocate { .. }
        | super::model::ResourceOp::CollectionSlotDropTraversal { .. }
        | super::model::ResourceOp::CollectionSlotTransformRange { .. } => true,
        super::model::ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_have_direct_raw_initialization_summary_op(then_ops)
                || ops_have_direct_raw_initialization_summary_op(else_ops)
        }
        super::model::ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_have_direct_raw_initialization_summary_op(condition_ops)
                || ops_have_direct_raw_initialization_summary_op(body_ops)
        }
        super::model::ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_have_direct_raw_initialization_summary_op(&arm.ops)),
        super::model::ResourceOp::Call { .. }
        | super::model::ResourceOp::Expr { .. }
        | super::model::ResourceOp::DeclareLocal { .. }
        | super::model::ResourceOp::Read { .. }
        | super::model::ResourceOp::Assign { .. }
        | super::model::ResourceOp::Borrow { .. }
        | super::model::ResourceOp::Move { .. }
        | super::model::ResourceOp::Drop { .. }
        | super::model::ResourceOp::EndScope { .. }
        | super::model::ResourceOp::CallEffect { .. }
        | super::model::ResourceOp::FunctionValue { .. }
        | super::model::ResourceOp::Construct { .. } => false,
    }
}

fn update_raw_cell_initialization_summary(
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
    summary: RawCellInitializationFunctionSummary,
) -> bool {
    let has_facts = !summary.return_cells.is_empty()
        || !summary.return_byte_ranges.is_empty()
        || !summary.param_cells.is_empty()
        || !summary.param_byte_ranges.is_empty()
        || !summary.param_release_requirements.is_empty()
        || !summary.variant_param_cells.is_empty()
        || !summary.variant_param_byte_ranges.is_empty()
        || !summary.variant_required_param_cells.is_empty()
        || !summary.variant_conditions.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_raw_cell_initialization_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
) -> RawCellInitializationFunctionSummary {
    let empty_collection_slot_summaries = CollectionSlotLifecycleFunctionSummaryIndex::new(&[]);
    let engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries: &empty_collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    for param in &function.params {
        seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
        }
    }

    let mut out = RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        return_cells: Vec::new(),
        return_byte_ranges: Vec::new(),
        param_cells: Vec::new(),
        param_byte_ranges: Vec::new(),
        param_release_requirements: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_param_byte_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_return_byte_ranges = None;
    let mut guaranteed_param_cells = None;
    let mut guaranteed_param_byte_ranges = None;
    let mut variant_initializations = PendingVariantRawCellInitializations::default();
    for block in &function.blocks {
        collect_param_release_requirements_from_ops(
            &mut out.param_release_requirements,
            &engine,
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &mut pending_reallocs,
            &mut variant_initializations,
            &function.params,
            raw_init_summaries,
            &block.ops,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let mut path_return_cells = Vec::new();
            if let Some(value) = value {
                collect_return_initialized_raw_cells(
                    &mut path_return_cells,
                    &cells,
                    &raw_aliases,
                    value,
                );
            }
            merge_guaranteed_facts(&mut guaranteed_return_cells, path_return_cells);

            let mut path_return_byte_ranges = Vec::new();
            if let Some(value) = value {
                collect_return_initialized_raw_byte_ranges(
                    &mut path_return_byte_ranges,
                    &cells,
                    &raw_aliases,
                    value,
                );
            }
            merge_guaranteed_facts(&mut guaranteed_return_byte_ranges, path_return_byte_ranges);

            let mut path_param_cells = Vec::new();
            collect_param_initialized_raw_cells(
                &mut path_param_cells,
                &cells,
                &raw_aliases,
                &function.params,
            );
            merge_guaranteed_facts(&mut guaranteed_param_cells, path_param_cells);

            let mut path_param_byte_ranges = Vec::new();
            collect_param_initialized_raw_byte_ranges(
                &mut path_param_byte_ranges,
                &cells,
                &raw_aliases,
                &function.params,
            );
            merge_guaranteed_facts(&mut guaranteed_param_byte_ranges, path_param_byte_ranges);
        }
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if ops_have_top_level_branch_output_for_return(&block.ops, value) {
                collect_variant_param_initialized_raw_cells_from_return(
                    &mut out.variant_param_cells,
                    &mut out.variant_param_byte_ranges,
                    &mut out.variant_required_param_cells,
                    &mut out.variant_conditions,
                    function,
                    types,
                    raw_alias_summaries,
                    i32_scalar_summaries,
                    raw_init_summaries,
                    &block.ops,
                    value,
                );
            }
        }
    }
    out.return_cells = guaranteed_return_cells.unwrap_or_default();
    out.return_byte_ranges = guaranteed_return_byte_ranges.unwrap_or_default();
    out.param_cells = guaranteed_param_cells.unwrap_or_default();
    out.param_byte_ranges = guaranteed_param_byte_ranges.unwrap_or_default();
    out
}

fn ops_have_top_level_branch_output_for_return(ops: &[ResourceOp], return_value: &Place) -> bool {
    // variant-param summary の collector は、現時点では return value そのものを
    // output とする top-level Branch だけを facts の抽出対象にしている。
    // その Branch がない block では collector を起動しても block prefix の再生だけが走るため、
    // 観測境界を保ったままここで探索対象から外す。
    ops.iter()
        .any(|op| matches!(op, ResourceOp::Branch { output, .. } if output == return_value))
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeCtx;

    use super::super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
    use super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceOp, ResourceTerminator,
    };
    use super::*;

    /// collection slot helper は raw memory op を直接持たない場合でも、
    /// call 境界で raw initialization summary と同じ initialized-cell facts を運ぶ。
    /// relevance pruning がこれを落とすと、helper 経由の slot 初期化証明が消える。
    #[test]
    fn collection_slot_ops_are_raw_initialization_summary_triggers() {
        let types = TypeCtx::new();
        let unit = types.unit();
        let slot = Place::local(String::from("slot"), unit);
        let function = ResourceFunction {
            name: String::from("slot_helper"),
            origin_name: String::from("slot_helper"),
            type_params: vec![],
            params: vec![],
            result: unit,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::CollectionSlotLifecycle {
                    target: slot,
                    event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: unit },
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };

        assert!(
            function_has_direct_raw_initialization_summary_op(&function),
            "collection slot marker だけを持つ helper も raw initialization summary worklist の seed である必要がある"
        );
    }

    /// variant-param summary の前提判定は、collector が実際に読む
    /// top-level Branch の output と return value の一致だけを見る。
    /// 一致しない Branch を候補に含めると、variant-param facts を作れない block でも
    /// ResourceCheckEngine の prefix replay が起動し、性能劣化だけが残る。
    #[test]
    fn variant_param_summary_scan_skips_without_return_branch_output() {
        let types = TypeCtx::new();
        let unit = types.unit();
        let return_value = Place::local(String::from("ret"), unit);
        let other_value = Place::local(String::from("other"), unit);
        let condition = Place::local(String::from("cond"), unit);

        assert!(!ops_have_top_level_branch_output_for_return(
            &[],
            &return_value
        ));

        let ops = vec![ResourceOp::Branch {
            output: other_value.clone(),
            condition,
            condition_fact: None,
            then_ops: vec![],
            then_value: other_value.clone(),
            else_ops: vec![],
            else_value: other_value,
            span: Span::dummy(),
        }];

        assert!(!ops_have_top_level_branch_output_for_return(
            &ops,
            &return_value
        ));
    }

    /// return value を直接作る top-level Branch は、variant-param summary が
    /// 分岐ごとの param-cell facts を回収するための入口である。
    /// この入口を保つことで、性能最適化が既存の variant 証明能力を削らないことを確認する。
    #[test]
    fn variant_param_summary_scan_detects_return_branch_output() {
        let types = TypeCtx::new();
        let unit = types.unit();
        let return_value = Place::local(String::from("ret"), unit);
        let condition = Place::local(String::from("cond"), unit);
        let then_value = Place::local(String::from("then"), unit);
        let else_value = Place::local(String::from("else"), unit);
        let ops = vec![ResourceOp::Branch {
            output: return_value.clone(),
            condition,
            condition_fact: None,
            then_ops: vec![],
            then_value,
            else_ops: vec![],
            else_value,
            span: Span::dummy(),
        }];

        assert!(ops_have_top_level_branch_output_for_return(
            &ops,
            &return_value
        ));
    }
}

fn merge_guaranteed_facts<T: Clone + Eq>(guaranteed: &mut Option<Vec<T>>, path: Vec<T>) {
    match guaranteed {
        Some(existing) => {
            existing.retain(|fact| path.contains(fact));
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}

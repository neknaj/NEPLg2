extern crate alloc;

use crate::types::{TypeCtx, TypeId, TypeKind};
use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateEntry, CollectionSlotStateTable};
use super::collection_slot_summary_build::compute_collection_slot_lifecycle_function_summaries_with_recomputations;
use super::collection_slot_summary_build_range_lifetime::transform_range_certificate_survives_op;
use super::collection_slot_summary_build_state::{
    CollectionSlotSummaryBuildState, CollectionSlotTransformRangeCertificateCandidate,
};
use super::collection_slot_summary_build_transform_range::loop_transform_range_certificates;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::drop_model::ResourceDropPoint;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    compute_raw_cell_address_return_summaries_with_recomputations, expr_kind_preserves_raw_alias,
    expr_kind_preserves_read_scalar_facts, RawCellAddressReturnSummaryIndex,
};
use super::initialized_function_check_value_cache::{
    initialized_function_check_cache_input,
    record_initialized_function_check_value_cache_candidate,
    replay_initialized_function_check_from_value_cache,
};
use super::initialized_path_state::{merge_path_alternatives_into, ResourcePathAlternatives};
use super::initialized_scalar_flow::{
    compute_i32_scalar_return_summaries, I32ScalarReturnSummaryIndex,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_summary_build::compute_call_boundary_raw_cell_initialization_function_summaries_with_recomputations;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellStateEntry, Place, PlaceRoot, ResourceBlock, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{reference_target_place, type_preserves_raw_address_alias};
use super::raw_realloc::PendingRawReallocs;
use super::report::{
    ResourceCheckDeferred, ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport,
    ResourceFunctionCheck,
};
use super::resource_summary_value_cache::{
    ResourceSummaryComputationStage, ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary_dependency::ResourceSummaryDependencyGraph;
#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
use super::timing::{resource_op_timing_enabled, resource_timing_function_matches};
use super::timing::{ResourceFunctionTimer, ResourceStageTimer};

pub fn check_resource_initialized_moves(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceCheckReport {
    check_resource_initialized_moves_inner(module, types, None, None)
}

/// Resource summary value cache の観測を伴う initialized-state checker。
///
/// この関数は通常の安全性判定を変えず、Resource summary のうち stable mirror へ
/// 変換できる小さな leaf entry だけを session cache へ保存・再投影する。長寿命の
/// value には `TypeId` や `Span` を保持せず、現在の compile の `TypeCtx` と source
/// policy 境界へ戻せる場合だけ worklist 前の preseed に使う。
pub fn check_resource_initialized_moves_with_summary_cache(
    module: &ResourceModule,
    types: &TypeCtx,
    summary_value_cache: &mut ResourceSummaryValueCache,
    summary_value_cache_context: &ResourceSummaryValueCacheContext,
) -> ResourceCheckReport {
    check_resource_initialized_moves_inner(
        module,
        types,
        Some(summary_value_cache),
        Some(summary_value_cache_context),
    )
}

fn check_resource_initialized_moves_inner(
    module: &ResourceModule,
    types: &TypeCtx,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> ResourceCheckReport {
    let stage_start = ResourceStageTimer::start();
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceCheckDeferred::default();
    let dependency_graph = ResourceSummaryDependencyGraph::build(module);
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        let op_count = module
            .functions
            .iter()
            .map(resource_function_op_count)
            .sum();
        cache.record_resource_static_input_shape(module.functions.len(), op_count);
    }
    let (raw_alias_summaries, raw_alias_recomputations) =
        compute_raw_cell_address_return_summaries_with_recomputations(
            module,
            types,
            &dependency_graph,
            summary_value_cache.as_deref_mut(),
            summary_value_cache_context,
        );
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        cache.record_initialized_summary_stage(
            ResourceSummaryComputationStage::RawAlias,
            raw_alias_recomputations,
            raw_alias_summaries.len(),
        );
    }
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    stage_start.log("resource_initialized_raw_alias_summaries");
    let stage_start = ResourceStageTimer::start();
    let (i32_scalar_summaries, i32_scalar_recomputations) = compute_i32_scalar_return_summaries(
        module,
        types,
        &raw_alias_summary_index,
        &dependency_graph,
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    );
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        cache.record_initialized_summary_stage(
            ResourceSummaryComputationStage::I32Scalar,
            i32_scalar_recomputations,
            i32_scalar_summaries.len(),
        );
    }
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    stage_start.log("resource_initialized_i32_scalar_summaries");
    let stage_start = ResourceStageTimer::start();
    let (raw_init_summaries, raw_init_recomputations) =
        compute_call_boundary_raw_cell_initialization_function_summaries_with_recomputations(
            module,
            types,
            &raw_alias_summaries,
            &i32_scalar_summaries,
            &dependency_graph,
            summary_value_cache.as_deref_mut(),
            summary_value_cache_context,
        );
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        cache.record_initialized_summary_stage(
            ResourceSummaryComputationStage::RawInit,
            raw_init_recomputations,
            raw_init_summaries.len(),
        );
    }
    let raw_init_summary_index =
        RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    stage_start.log("resource_initialized_raw_init_summaries");
    let stage_start = ResourceStageTimer::start();
    let (collection_slot_summaries, collection_slot_recomputations) =
        compute_collection_slot_lifecycle_function_summaries_with_recomputations(
            module,
            types,
            &raw_alias_summaries,
            &i32_scalar_summaries,
            &raw_init_summaries,
            &dependency_graph,
            summary_value_cache.as_deref_mut(),
            summary_value_cache_context,
        );
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        cache.record_initialized_summary_stage(
            ResourceSummaryComputationStage::CollectionSlot,
            collection_slot_recomputations,
            collection_slot_summaries.len(),
        );
    }
    let collection_slot_summary_index =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    stage_start.log("resource_initialized_collection_slot_summaries");
    let stage_start = ResourceStageTimer::start();
    let mut final_check_pass_plan = match (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        (Some(cache), Some(context))
            if cache.stable_entry_collection_enabled()
                || cache.has_initialized_function_check_pass_snapshot() =>
        {
            Some(cache.begin_initialized_function_check_pass_plan(
                context,
                types,
                module,
                &dependency_graph,
            ))
        }
        _ => None,
    };
    let can_replay_function_check_entries =
        match (summary_value_cache.as_ref(), summary_value_cache_context) {
            (Some(cache), Some(context)) => {
                cache.has_initialized_function_check_replay_entries(context)
            }
            _ => false,
        };
    let can_record_function_check_entries = summary_value_cache
        .as_ref()
        .is_some_and(|cache| cache.stable_entry_collection_enabled());
    for (function_index, function) in module.functions.iter().enumerate() {
        let function_start = ResourceFunctionTimer::start();
        let function_op_count = resource_function_op_count(function);
        if let (Some(cache), Some(plan)) = (
            summary_value_cache.as_deref_mut(),
            final_check_pass_plan.as_mut(),
        ) {
            if let Some(replayed_check) = cache.replay_unchanged_initialized_function_check_pass(
                plan,
                function_index,
                function,
                function_op_count,
            ) {
                merge_deferred(&mut deferred, replayed_check.deferred);
                functions.push(replayed_check);
                function_start.log("resource_initialized_function_check", function);
                continue;
            }
        }
        let function_check_cache_input =
            if can_replay_function_check_entries || can_record_function_check_entries {
                initialized_function_check_cache_input(
                    summary_value_cache.as_deref_mut(),
                    summary_value_cache_context,
                    types,
                    module,
                    Some(dependency_graph.dependencies()),
                    function_index,
                    function,
                    function_op_count,
                )
            } else {
                None
            };
        if can_replay_function_check_entries {
            if let Some(cache) = summary_value_cache.as_deref_mut() {
                cache.record_initialized_function_check_replay_probe_function();
            }
            if let Some(replayed_check) = replay_initialized_function_check_from_value_cache(
                summary_value_cache.as_deref_mut(),
                summary_value_cache_context,
                types,
                function,
                function_check_cache_input.as_ref(),
                function_op_count,
            ) {
                merge_deferred(&mut deferred, replayed_check.deferred);
                if let Some(plan) = final_check_pass_plan.as_mut() {
                    plan.record_pass(function_index, replayed_check.deferred);
                }
                functions.push(replayed_check);
                function_start.log("resource_initialized_function_check", function);
                continue;
            }
            if let Some(cache) = summary_value_cache.as_deref_mut() {
                cache.record_initialized_function_check_replay_miss_function();
            }
        }
        if let Some(cache) = summary_value_cache.as_deref_mut() {
            cache.record_initialized_function_check(function_op_count);
        }
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summary_index,
            i32_scalar_summaries: &i32_scalar_summary_index,
            raw_init_summaries: &raw_init_summary_index,
            collection_slot_summaries: &collection_slot_summary_index,
            transform_range_certificates: function_needs_local_transform_range_certificates(
                function,
            )
            .then(Vec::new),
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
            path_alternatives: ResourcePathAlternatives::default(),
        };
        let (final_cells, final_collection_slots) = engine.check_function(function);
        merge_deferred(&mut deferred, engine.deferred);
        dedup_resource_check_diagnostics(&mut engine.diagnostics);
        let function_has_diagnostics = !engine.diagnostics.is_empty();
        diagnostics.extend(engine.diagnostics);
        let function_check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells,
            final_collection_slots,
            auto_drop_points: engine.auto_drop_points,
            deferred: engine.deferred,
        };
        let recorded_cache_pass = record_initialized_function_check_value_cache_candidate(
            summary_value_cache.as_deref_mut(),
            summary_value_cache_context,
            types,
            function,
            function_check_cache_input.as_ref(),
            &function_check,
            function_has_diagnostics,
            function_op_count,
        );
        if recorded_cache_pass {
            if let Some(plan) = final_check_pass_plan.as_mut() {
                plan.record_pass(function_index, function_check.deferred);
            }
        }
        functions.push(function_check);
        function_start.log("resource_initialized_function_check", function);
    }
    if let (Some(cache), Some(plan)) = (summary_value_cache.as_deref_mut(), final_check_pass_plan) {
        cache.finish_initialized_function_check_pass_plan(plan);
    }
    stage_start.log("resource_initialized_function_checks");

    ResourceCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

fn dedup_resource_check_diagnostics(diagnostics: &mut Vec<ResourceCheckDiagnostic>) {
    let mut unique = Vec::new();
    for diagnostic in diagnostics.drain(..) {
        if !unique.contains(&diagnostic) {
            unique.push(diagnostic);
        }
    }
    *diagnostics = unique;
}

fn resource_function_op_count(function: &ResourceFunction) -> usize {
    function.blocks.iter().map(|block| block.ops.len()).sum()
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn resource_op_kind(op: &ResourceOp) -> &'static str {
    match op {
        ResourceOp::Expr { .. } => "expr",
        ResourceOp::DeclareLocal { .. } => "declare_local",
        ResourceOp::Read { .. } => "read",
        ResourceOp::Assign { .. } => "assign",
        ResourceOp::Borrow { .. } => "borrow",
        ResourceOp::Move { .. } => "move",
        ResourceOp::Drop { .. } => "drop",
        ResourceOp::EndScope { .. } => "end_scope",
        ResourceOp::CallEffect { .. } => "call_effect",
        ResourceOp::FunctionValue { .. } => "function_value",
        ResourceOp::Call { .. } => "call",
        ResourceOp::IndirectCall { .. } => "indirect_call",
        ResourceOp::RawMemory { .. } => "raw_memory",
        ResourceOp::RawAddressAlias { .. } => "raw_address_alias",
        ResourceOp::RawAddressView { .. } => "raw_address_view",
        ResourceOp::StorageOrigin { .. } => "storage_origin",
        ResourceOp::CollectionSlotLifecycle { .. } => "collection_slot_lifecycle",
        ResourceOp::CollectionStorageRelocate { .. } => "collection_storage_relocate",
        ResourceOp::CollectionSlotDropTraversal { .. } => "collection_slot_drop_traversal",
        ResourceOp::CollectionSlotTransformRange { .. } => "collection_slot_transform_range",
        ResourceOp::Construct { .. } => "construct",
        ResourceOp::Branch { .. } => "branch",
        ResourceOp::Loop { .. } => "loop",
        ResourceOp::Match { .. } => "match",
    }
}

pub(super) fn op_can_run_on_merged_path_state(_types: &TypeCtx, op: &ResourceOp) -> bool {
    matches!(op, ResourceOp::Branch { .. } | ResourceOp::Match { .. })
}

pub(super) fn apply_path_preserving_fast_op(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    op: &ResourceOp,
) -> bool {
    match op {
        ResourceOp::Expr { kind, output, .. } if expr_can_use_path_preserving_fast_op(*kind, output) => {
            match kind {
                ResourceExprKind::LiteralI32(value) => {
                    cells.mark_initialized(output);
                    raw_aliases.set_i32_value(output, *value);
                }
                ResourceExprKind::LayoutSizeOf(ty) => {
                    cells.mark_initialized(output);
                    raw_aliases.set_i32_type_size(output, *ty);
                }
                _ => return false,
            }
            true
        }
        ResourceOp::EndScope { locals, .. } if locals.iter().all(|local| types.is_copy(local.ty)) => {
            // Copy local だけを含む scope 終了は auto drop を生成せず、path ごとの差分も
            // 変更しない。path alternatives を保持したまま処理済みとして扱うことで、
            // Result return path や collection slot proof を誤って畳まない。
            true
        }
        _ => false,
    }
}

fn expr_can_use_path_preserving_fast_op(kind: ResourceExprKind, output: &Place) -> bool {
    // path-sensitive alternatives を保持している最中に merged state だけを進めると、
    // その operation のあとで alternatives を破棄してしまう。ここで許可する式は、
    // 既存 place を読まず、診断も生成せず、fresh temporary だけに確定値を置くものに
    // 限定し、各 alternative に同じ小さな状態更新だけを反映する。local や projection
    // 付き output を許すと、別 path の alias / scalar fact を上書きし、後続診断の精度を
    // 落とす可能性がある。
    matches!(
        kind,
        ResourceExprKind::LiteralI32(_) | ResourceExprKind::LayoutSizeOf(_)
    ) && matches!(output.root, PlaceRoot::Temporary(_))
        && output.projections.is_empty()
}

fn function_needs_local_transform_range_certificates(function: &ResourceFunction) -> bool {
    // transform range certificate は同一関数内の
    // `CollectionSlotTransformRange` が消費する局所証明である。関数内に消費先が
    // 存在しない場合、loop ごとに候補を構築しても静的検査の結果には影響せず、
    // 文字列処理など collection transform を含まない hot path で不要な探索になる。
    function
        .blocks
        .iter()
        .any(|block| ops_need_local_transform_range_certificates(&block.ops))
}

fn ops_need_local_transform_range_certificates(ops: &[ResourceOp]) -> bool {
    ops.iter().any(op_needs_local_transform_range_certificates)
}

fn op_needs_local_transform_range_certificates(op: &ResourceOp) -> bool {
    match op {
        ResourceOp::CollectionSlotTransformRange { .. } => true,
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_need_local_transform_range_certificates(then_ops)
                || ops_need_local_transform_range_certificates(else_ops)
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_need_local_transform_range_certificates(condition_ops)
                || ops_need_local_transform_range_certificates(body_ops)
        }
        ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_need_local_transform_range_certificates(&arm.ops)),
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::Construct { .. } => false,
    }
}

fn collection_slot_event_precondition_state(
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotState> {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { .. }
        | CollectionSlotLifecycleEvent::StorageDealloc { .. } => None,
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty }
        | CollectionSlotLifecycleEvent::MoveOut { expected_ty }
        | CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => {
            Some(CollectionSlotState::Initialized(expected_ty))
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized { old_ty, .. } => {
            Some(CollectionSlotState::Initialized(old_ty))
        }
    }
}

pub(super) struct ResourceCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) types: &'a TypeCtx,
    pub(super) raw_alias_summaries: &'a RawCellAddressReturnSummaryIndex<'a>,
    pub(super) i32_scalar_summaries: &'a I32ScalarReturnSummaryIndex<'a>,
    pub(super) raw_init_summaries: &'a RawCellInitializationFunctionSummaryIndex<'a>,
    pub(super) collection_slot_summaries: &'a CollectionSlotLifecycleFunctionSummaryIndex<'a>,
    pub(super) transform_range_certificates:
        Option<Vec<CollectionSlotTransformRangeCertificateCandidate>>,
    pub(super) diagnostics: Vec<ResourceCheckDiagnostic>,
    pub(super) auto_drop_points: Vec<ResourceDropPoint>,
    pub(super) deferred: ResourceCheckDeferred,
    pub(super) path_alternatives: ResourcePathAlternatives,
}

impl ResourceCheckEngine<'_> {
    fn check_function(
        &mut self,
        function: &ResourceFunction,
    ) -> (Vec<CellStateEntry>, Vec<CollectionSlotStateEntry>) {
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        let mut variant_initializations = PendingVariantRawCellInitializations::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            self.seed_external_raw_storage_parameter(&mut cells, &mut raw_aliases, &param.place);
            seed_str_storage_layout(self.types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = self.reference_target_type(param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                cells.mark_initialized(&target);
                self.seed_external_raw_storage_parameter(&mut cells, &mut raw_aliases, &target);
                seed_str_storage_layout(self.types, &mut cells, &mut raw_aliases, &target);
            }
        }
        self.seed_collection_slot_summary_preconditions(
            &mut collection_slots,
            &raw_aliases,
            function,
        );
        for block in &function.blocks {
            self.check_block(
                &mut cells,
                &mut collection_slots,
                &mut raw_aliases,
                &mut function_aliases,
                &mut pending_reallocs,
                &mut variant_initializations,
                block,
            );
        }
        (cells.into_entries(), collection_slots.entries().to_vec())
    }

    fn seed_collection_slot_summary_preconditions(
        &self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        function: &ResourceFunction,
    ) {
        let Some(summary) = self.collection_slot_summaries.get(&function.name) else {
            return;
        };
        let args = function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect::<Vec<_>>();
        if summary.return_paths.is_empty() {
            let mut simulated_slots = collection_slots.clone();
            self.seed_collection_slot_summary_op_preconditions(
                collection_slots,
                &mut simulated_slots,
                raw_aliases,
                &args,
                &summary.ops,
            );
        } else {
            for return_path in &summary.return_paths {
                let mut simulated_slots = collection_slots.clone();
                self.seed_collection_slot_summary_op_preconditions(
                    collection_slots,
                    &mut simulated_slots,
                    raw_aliases,
                    &args,
                    &return_path.ops,
                );
            }
        }
    }

    fn seed_collection_slot_summary_op_preconditions(
        &self,
        collection_slots: &mut CollectionSlotStateTable,
        simulated_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        ops: &[CollectionSlotLifecycleSummaryOp],
    ) {
        for op in ops {
            match op {
                CollectionSlotLifecycleSummaryOp::Event { target, event, .. } => {
                    let Some(target) =
                        instantiate_summary_target_with_aliases(self, args, raw_aliases, target)
                    else {
                        continue;
                    };
                    if let Some(initial_state) = collection_slot_event_precondition_state(*event) {
                        let state = simulated_slots.state_with_aliases(&target, raw_aliases);
                        if matches!(state, CollectionSlotState::Uninitialized)
                            && apply_collection_slot_lifecycle_event(self.types, state, *event)
                                .is_err()
                        {
                            collection_slots.set_slot_state(&target, initial_state);
                            simulated_slots.set_slot_state(&target, initial_state);
                        }
                    }
                    let _ = simulated_slots.apply_slot_event_with_aliases(
                        self.types,
                        &target,
                        raw_aliases,
                        *event,
                    );
                }
                CollectionSlotLifecycleSummaryOp::Merge { paths } => {
                    let mut path_slots = Vec::new();
                    for path in paths {
                        let mut branch_slots = simulated_slots.clone();
                        self.seed_collection_slot_summary_op_preconditions(
                            collection_slots,
                            &mut branch_slots,
                            raw_aliases,
                            args,
                            path,
                        );
                        path_slots.push(branch_slots);
                    }
                    if !path_slots.is_empty() {
                        *simulated_slots = CollectionSlotStateTable::merge_paths(&path_slots);
                    }
                }
                CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops,
                    body_ops,
                } => {
                    let mut condition_slots = simulated_slots.clone();
                    self.seed_collection_slot_summary_op_preconditions(
                        collection_slots,
                        &mut condition_slots,
                        raw_aliases,
                        args,
                        condition_ops,
                    );
                    let exit_slots = condition_slots.clone();
                    let mut body_slots = condition_slots;
                    self.seed_collection_slot_summary_op_preconditions(
                        collection_slots,
                        &mut body_slots,
                        raw_aliases,
                        args,
                        body_ops,
                    );
                    *simulated_slots =
                        CollectionSlotStateTable::merge_paths(&[exit_slots, body_slots]);
                }
                CollectionSlotLifecycleSummaryOp::Relocate { .. }
                | CollectionSlotLifecycleSummaryOp::DropTraversal { .. }
                | CollectionSlotLifecycleSummaryOp::TransformRange { .. }
                | CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain { .. } => {}
            }
        }
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            &block.ops,
            ResourceDropPointPath {
                block: block.id,
                steps: Vec::new(),
            },
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.consume_return_value(
                        cells,
                        collection_slots,
                        raw_aliases,
                        function_aliases,
                        pending_reallocs,
                        variant_initializations,
                        value,
                        *span,
                    );
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn consume_return_value(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        value: &Place,
        span: crate::span::Span,
    ) {
        let Some(mut alternatives) =
            core::mem::take(&mut self.path_alternatives).into_feasible_states()
        else {
            self.consume_by_value(cells, value, ResourceCheckOperation::ReturnValue, span);
            return;
        };
        if alternatives.is_empty() {
            self.path_alternatives = ResourcePathAlternatives::from_states(alternatives);
            return;
        }
        for state in &mut alternatives {
            self.consume_by_value(
                &mut state.cells,
                value,
                ResourceCheckOperation::ReturnValue,
                span,
            );
        }
        merge_path_alternatives_into(
            &alternatives,
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        );
    }

    pub(super) fn check_ops(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        ops: &[ResourceOp],
        path: ResourceDropPointPath,
    ) {
        for (index, op) in ops.iter().enumerate() {
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            let op_timing = {
                if !resource_op_timing_enabled() || !resource_timing_function_matches(self.function)
                {
                    None
                } else {
                    std::eprintln!(
                        "[resource-op-timing] start function={} op={} kind={} incoming_paths={}",
                        self.function,
                        index,
                        resource_op_kind(op),
                        self.path_alternatives.len()
                    );
                    Some(std::time::Instant::now())
                }
            };
            let mut pending_transform_range_certificates = self
                .pending_local_transform_range_certificates(
                    cells,
                    collection_slots,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    op,
                );
            let incoming_path_alternatives = core::mem::take(&mut self.path_alternatives);
            let op_path = path.clone().with_step(ResourceDropPointStep::Op { index });
            match incoming_path_alternatives {
                ResourcePathAlternatives::None => self.check_op(
                    cells,
                    collection_slots,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    op,
                    op_path,
                ),
                ResourcePathAlternatives::Feasible(alternatives) => {
                    let mut alternatives = alternatives;
                    if alternatives.is_empty() {
                        self.path_alternatives =
                            ResourcePathAlternatives::from_states(alternatives);
                    } else if apply_path_preserving_fast_op(self.types, cells, raw_aliases, op) {
                        for state in &mut alternatives {
                            apply_path_preserving_fast_op(
                                self.types,
                                &mut state.cells,
                                &mut state.raw_aliases,
                                op,
                            );
                        }
                        merge_path_alternatives_into(
                            &alternatives,
                            cells,
                            collection_slots,
                            raw_aliases,
                            function_aliases,
                            pending_reallocs,
                            variant_initializations,
                        );
                        self.path_alternatives = ResourcePathAlternatives::from_states(alternatives);
                    } else if op_can_run_on_merged_path_state(self.types, op) {
                        self.check_op(
                            cells,
                            collection_slots,
                            raw_aliases,
                            function_aliases,
                            pending_reallocs,
                            variant_initializations,
                            op,
                            op_path,
                        );
                    } else {
                        let advanced =
                            self.advance_path_alternatives_after_op(alternatives, op, op_path);
                        merge_path_alternatives_into(
                            &advanced,
                            cells,
                            collection_slots,
                            raw_aliases,
                            function_aliases,
                            pending_reallocs,
                            variant_initializations,
                        );
                        self.path_alternatives = ResourcePathAlternatives::from_states(advanced);
                    }
                }
            }
            self.retain_local_transform_range_certificates_after_op(raw_aliases, op);
            if let Some(candidates) = &mut self.transform_range_certificates {
                candidates.append(&mut pending_transform_range_certificates);
            }
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            if let Some(start) = op_timing {
                std::eprintln!(
                    "[resource-op-timing] end function={} op={} kind={} outgoing_paths={} elapsed_ms={}",
                    self.function,
                    index,
                    resource_op_kind(op),
                    self.path_alternatives.len(),
                    start.elapsed().as_millis()
                );
            }
        }
    }

    fn pending_local_transform_range_certificates(
        &self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        pending_reallocs: &PendingRawReallocs,
        variant_initializations: &PendingVariantRawCellInitializations,
        op: &ResourceOp,
    ) -> Vec<CollectionSlotTransformRangeCertificateCandidate> {
        if self.transform_range_certificates.is_none() {
            return Vec::new();
        }
        let ResourceOp::Loop {
            condition_ops,
            condition_fact,
            body_ops,
            ..
        } = op
        else {
            return Vec::new();
        };
        let state = CollectionSlotSummaryBuildState {
            cells: cells.clone(),
            collection_slots: collection_slots.clone(),
            raw_aliases: raw_aliases.clone(),
            function_aliases: function_aliases.clone(),
            pending_reallocs: pending_reallocs.clone(),
            variant_initializations: variant_initializations.clone(),
            drop_traversal_range_certificates: Vec::new(),
            transform_range_certificates: self
                .transform_range_certificates
                .clone()
                .unwrap_or_default(),
        };
        let candidates = loop_transform_range_certificates(
            self,
            &state,
            condition_ops,
            condition_fact.as_ref(),
            body_ops,
        );
        candidates
    }

    fn retain_local_transform_range_certificates_after_op(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        op: &ResourceOp,
    ) {
        let Some(candidates) = &mut self.transform_range_certificates else {
            return;
        };
        let raw_aliases = raw_aliases.clone();
        candidates.retain(|candidate| {
            transform_range_certificate_survives_op(self.types, &raw_aliases, candidate, op)
        });
    }

    pub(super) fn apply_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) -> bool {
        apply_direct_call_raw_alias_summary(
            raw_aliases,
            output,
            target,
            args,
            self.raw_alias_summaries,
            self.types,
        )
    }

    pub(super) fn apply_indirect_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) -> bool {
        apply_indirect_call_raw_alias_summary(
            raw_aliases,
            function_aliases,
            output,
            callee,
            args,
            self.raw_alias_summaries,
            self.types,
        )
    }

    pub(super) fn check_expr(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::LiteralI32(value) => {
                cells.mark_initialized(output);
                raw_aliases.set_i32_value(output, value);
            }
            ResourceExprKind::LayoutSizeOf(ty) => {
                cells.mark_initialized(output);
                raw_aliases.set_i32_type_size(output, ty);
            }
            ResourceExprKind::Literal
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop
            | ResourceExprKind::Loop => cells.mark_initialized(output),
            ResourceExprKind::LocalRead
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
            | ResourceExprKind::Borrow => {}
        }
        if !matches!(
            kind,
            ResourceExprKind::LiteralI32(_) | ResourceExprKind::LayoutSizeOf(_)
        ) && !expr_kind_preserves_raw_alias(kind)
            && !(matches!(kind, ResourceExprKind::Deref)
                && type_preserves_raw_address_alias(self.types, output.ty))
        {
            if expr_kind_preserves_read_scalar_facts(kind) {
                raw_aliases.clear_raw_address_facts(output);
            } else {
                raw_aliases.clear(output);
            }
        }
        seed_str_storage_layout(self.types, cells, raw_aliases, output);
    }

    pub(super) fn reference_target_type(&self, ty: TypeId) -> Option<TypeId> {
        let resolved = self.types.resolve_named_type_id(self.types.resolve_id(ty));
        match self.types.get_ref(resolved) {
            TypeKind::Reference(target, _) => Some(*target),
            _ => None,
        }
    }
}

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::resource::model::{
        EffectOp, PlaceProjection, PlaceRoot, ResourceId, ResourceMatchArm, ResourceMatchPattern,
    };
    use crate::span::Span;

    use super::*;

    fn local(name: &str) -> Place {
        Place {
            root: PlaceRoot::Local(String::from(name)),
            projections: Vec::new(),
            ty: TypeId(0),
        }
    }

    fn temporary(id: usize) -> Place {
        Place {
            root: PlaceRoot::Temporary(ResourceId(id)),
            projections: Vec::new(),
            ty: TypeId(1),
        }
    }

    fn function_with_ops(ops: Vec<ResourceOp>) -> ResourceFunction {
        ResourceFunction {
            name: String::from("test"),
            origin_name: String::from("test"),
            type_params: Vec::new(),
            params: Vec::new(),
            result: TypeId(0),
            effect: Effect::Pure,
            entry_block: super::super::model::ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: super::super::model::ResourceBlockId(0),
                ops,
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    /// fresh temporary へ書く i32 scalar 生成は、path ごとの入力を読まず診断も
    /// 生成しないため、path alternatives を保持したまま軽量に反映できることを確認する。
    #[test]
    fn path_preserving_fast_op_accepts_fresh_temporary_i32_scalar_exprs() {
        let types = TypeCtx::new();
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let literal = ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(42),
            output: temporary(0),
            ty: TypeId(1),
            span: Span::dummy(),
        };
        let layout_size = ResourceOp::Expr {
            kind: ResourceExprKind::LayoutSizeOf(TypeId(1)),
            output: temporary(1),
            ty: TypeId(1),
            span: Span::dummy(),
        };

        assert!(apply_path_preserving_fast_op(
            &types,
            &mut cells,
            &mut raw_aliases,
            &literal
        ));
        assert!(apply_path_preserving_fast_op(
            &types,
            &mut cells,
            &mut raw_aliases,
            &layout_size
        ));
        assert!(!op_can_run_on_merged_path_state(&types, &literal));
        assert!(!op_can_run_on_merged_path_state(&types, &layout_size));
    }

    /// local や projection 付き output は、path ごとに異なる alias / scalar fact を
    /// 持ち得るため、path-preserving fast path でも処理しないことを確認する。
    #[test]
    fn path_preserving_fast_op_rejects_non_fresh_scalar_expr_outputs() {
        let types = TypeCtx::new();
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let local_output = ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(7),
            output: local("x"),
            ty: TypeId(1),
            span: Span::dummy(),
        };
        let projected_temporary = ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(7),
            output: temporary(0).with_projection(PlaceProjection::Deref, TypeId(1)),
            ty: TypeId(1),
            span: Span::dummy(),
        };
        let non_scalar_literal = ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: temporary(1),
            ty: TypeId(0),
            span: Span::dummy(),
        };

        assert!(!apply_path_preserving_fast_op(
            &types,
            &mut cells,
            &mut raw_aliases,
            &local_output
        ));
        assert!(!apply_path_preserving_fast_op(
            &types,
            &mut cells,
            &mut raw_aliases,
            &projected_temporary
        ));
        assert!(!apply_path_preserving_fast_op(
            &types,
            &mut cells,
            &mut raw_aliases,
            &non_scalar_literal
        ));
    }

    /// `CallEffect` は initialized-state を直接変更しないが、call の直前に置かれる。
    /// Result return path などの path-sensitive fact を次の `Call` へ渡す必要があるため、
    /// merged state だけで処理して path alternatives を畳んではならない。
    #[test]
    fn merged_path_state_rejects_call_effect_path_collapse() {
        let types = TypeCtx::new();
        let call_effect = ResourceOp::CallEffect {
            effect: EffectOp::Pure,
            span: Span::dummy(),
        };

        assert!(!op_can_run_on_merged_path_state(&types, &call_effect));
    }

    fn literal_op(output: &str) -> ResourceOp {
        ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: local(output),
            ty: TypeId(0),
            span: Span::dummy(),
        }
    }

    fn transform_range_op() -> ResourceOp {
        ResourceOp::CollectionSlotTransformRange {
            source_storage: local("source_storage"),
            source_initialized_count: local("source_count"),
            output_storage: local("output_storage"),
            output_initialized_count: local("output_count"),
            expected_ty: TypeId(0),
            span: Span::dummy(),
        }
    }

    /// loop や分岐を含んでいても、関数内に transform-range 証明の消費先が
    /// ないなら候補構築を起動しないことを確認する。
    #[test]
    fn local_transform_range_certificate_scan_skips_functions_without_consumer() {
        let function = function_with_ops(vec![ResourceOp::Loop {
            condition_ops: vec![literal_op("condition_tmp")],
            condition: local("condition"),
            condition_fact: None,
            body_ops: vec![ResourceOp::Branch {
                output: local("branch_output"),
                condition: local("branch_condition"),
                condition_fact: None,
                then_ops: vec![literal_op("then_value")],
                then_value: local("then_value"),
                else_ops: vec![literal_op("else_value")],
                else_value: local("else_value"),
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        assert!(!function_needs_local_transform_range_certificates(
            &function
        ));
    }

    /// transform-range 証明の消費先が nested control flow の内側にある場合でも、
    /// 関数全体として候補構築を有効にすることを確認する。
    #[test]
    fn local_transform_range_certificate_scan_finds_nested_consumer() {
        let function = function_with_ops(vec![ResourceOp::Match {
            output: local("match_output"),
            scrutinee: local("scrutinee"),
            scrutinee_is_borrow_target: false,
            arms: vec![ResourceMatchArm {
                pattern: ResourceMatchPattern::Wildcard,
                bind_local: None,
                bind_source_name: None,
                bind_mode: None,
                ops: vec![ResourceOp::Loop {
                    condition_ops: Vec::new(),
                    condition: local("condition"),
                    condition_fact: None,
                    body_ops: vec![transform_range_op()],
                    span: Span::dummy(),
                }],
                value: local("arm_value"),
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        assert!(function_needs_local_transform_range_certificates(&function));
    }
}

#[cfg(test)]
#[path = "initialized_function_check_value_cache_tests.rs"]
mod initialized_function_check_value_cache_tests;

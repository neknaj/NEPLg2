use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::TypeKind;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_build_range_bound::initialized_range_loop_bound;
use super::collection_slot_summary_build_range_preserve::body_preserves_place;
use super::collection_slot_summary_build_range_step_expr::{
    effect_is_proof_pure, loop_step_expr_effect, LoopStepExprEffect,
};
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::condition_fact::record_condition_fact_value_constraints;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_control_slot_transfer::transfer_control_value_slots as transfer_slots;
use super::initialized_path_state::{
    log_path_state_replay_reason, merge_path_alternatives_into, path_states_need_replay,
    ResourceCheckState, ResourcePathAlternatives,
};
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, Place, PlaceProjection, RawMemoryOp, ResourceConditionFact, ResourceMatchArm,
    ResourceOffset, ResourceOp,
};
use super::place_utils::{match_bind_payload_place, place_suffix_after_prefix};
use super::raw_cell_lifecycle::RawCellLifecycleEvent;
use super::raw_realloc::{
    raw_realloc_condition_outcome, PendingRawReallocs, RawReallocConditionOutcome,
};
use super::report::ResourceCheckOperation;
use super::summary_projection::{SummaryOffset, SummaryProjection};

impl ResourceCheckEngine<'_> {
    pub(super) fn check_branch(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        condition: &Place,
        condition_fact: Option<&ResourceConditionFact>,
        then_ops: &[ResourceOp],
        then_value: &Place,
        else_ops: &[ResourceOp],
        else_value: &Place,
        span: Span,
        then_path: ResourceDropPointPath,
        else_path: ResourceDropPointPath,
    ) {
        let condition_available = self.consume_by_value(
            cells,
            condition,
            ResourceCheckOperation::BranchCondition,
            span,
        );
        cells.discard_raw_cell_loaded_value_origin(condition);
        let mut then_cells = cells.clone();
        let mut else_cells = cells.clone();
        let mut then_collection_slots = collection_slots.clone();
        let mut else_collection_slots = collection_slots.clone();
        let mut then_aliases = raw_aliases.clone();
        let mut else_aliases = raw_aliases.clone();
        let mut then_function_aliases = function_aliases.clone();
        let mut else_function_aliases = function_aliases.clone();
        let mut then_pending_reallocs = pending_reallocs.clone();
        let mut else_pending_reallocs = pending_reallocs.clone();
        let mut then_variant_initializations = variant_initializations.clone();
        let mut else_variant_initializations = variant_initializations.clone();

        self.apply_branch_condition_fact(
            &mut then_cells,
            &mut then_aliases,
            &mut then_pending_reallocs,
            condition_fact,
            true,
        );
        self.apply_branch_condition_fact(
            &mut else_cells,
            &mut else_aliases,
            &mut else_pending_reallocs,
            condition_fact,
            false,
        );
        self.check_ops(
            &mut then_cells,
            &mut then_collection_slots,
            &mut then_aliases,
            &mut then_function_aliases,
            &mut then_pending_reallocs,
            &mut then_variant_initializations,
            then_ops,
            then_path,
        );
        let then_path_alternatives = core::mem::take(&mut self.path_alternatives);
        self.check_ops(
            &mut else_cells,
            &mut else_collection_slots,
            &mut else_aliases,
            &mut else_function_aliases,
            &mut else_pending_reallocs,
            &mut else_variant_initializations,
            else_ops,
            else_path,
        );
        let else_path_alternatives = core::mem::take(&mut self.path_alternatives);

        let mut branch_paths = Vec::new();
        let mut paths_available = condition_available;
        if !self.place_is_never(then_value) {
            let then_states = path_alternatives_or_single(
                then_path_alternatives,
                then_cells,
                then_collection_slots,
                then_aliases,
                then_function_aliases,
                then_pending_reallocs,
                then_variant_initializations,
            );
            branch_paths.extend(self.transfer_control_value_path_states(
                then_states,
                then_value,
                output,
                ResourceCheckOperation::BranchValue,
                span,
                &mut paths_available,
            ));
        }
        if !self.place_is_never(else_value) {
            let else_states = path_alternatives_or_single(
                else_path_alternatives,
                else_cells,
                else_collection_slots,
                else_aliases,
                else_function_aliases,
                else_pending_reallocs,
                else_variant_initializations,
            );
            branch_paths.extend(self.transfer_control_value_path_states(
                else_states,
                else_value,
                output,
                ResourceCheckOperation::BranchValue,
                span,
                &mut paths_available,
            ));
        }

        let has_branch_paths = !branch_paths.is_empty();
        if has_branch_paths {
            merge_path_alternatives_into(
                &branch_paths,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            );
            if path_states_need_replay(&branch_paths) {
                log_path_state_replay_reason(self.function, "branch", &branch_paths);
                self.path_alternatives = ResourcePathAlternatives::from_states(branch_paths);
            }
        }
        if paths_available && has_branch_paths {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            invalidate_control_output_state(
                cells,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                output,
            );
            invalidate_control_output_path_states(&mut self.path_alternatives, output);
        }
    }

    pub(super) fn check_loop(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        condition_ops: &[ResourceOp],
        condition: &Place,
        condition_fact: Option<&ResourceConditionFact>,
        body_ops: &[ResourceOp],
        span: Span,
        condition_path: ResourceDropPointPath,
        body_path: ResourceDropPointPath,
    ) {
        let mut condition_cells = cells.clone();
        let mut condition_collection_slots = collection_slots.clone();
        let mut condition_aliases = raw_aliases.clone();
        let mut condition_function_aliases = function_aliases.clone();
        let mut condition_pending_reallocs = pending_reallocs.clone();
        let mut condition_variant_initializations = variant_initializations.clone();
        self.check_ops(
            &mut condition_cells,
            &mut condition_collection_slots,
            &mut condition_aliases,
            &mut condition_function_aliases,
            &mut condition_pending_reallocs,
            &mut condition_variant_initializations,
            condition_ops,
            condition_path.clone(),
        );
        core::mem::take(&mut self.path_alternatives);
        self.consume_by_value(
            &mut condition_cells,
            condition,
            ResourceCheckOperation::LoopCondition,
            span,
        );
        condition_cells.discard_raw_cell_loaded_value_origin(condition);

        let mut exit_cells = condition_cells.clone();
        let exit_collection_slots = condition_collection_slots.clone();
        let mut exit_aliases = condition_aliases.clone();
        let mut exit_pending_reallocs = condition_pending_reallocs.clone();
        self.apply_branch_condition_fact(
            &mut exit_cells,
            &mut exit_aliases,
            &mut exit_pending_reallocs,
            condition_fact,
            false,
        );

        let mut body_cells = condition_cells;
        let mut body_collection_slots = condition_collection_slots;
        let mut body_aliases = condition_aliases;
        let mut body_function_aliases = condition_function_aliases.clone();
        let mut body_pending_reallocs = condition_pending_reallocs;
        let mut body_variant_initializations = condition_variant_initializations.clone();
        self.apply_branch_condition_fact(
            &mut body_cells,
            &mut body_aliases,
            &mut body_pending_reallocs,
            condition_fact,
            true,
        );
        let initialized_range_candidates =
            loop_initialized_range_candidates(self, &body_aliases, condition_fact, body_ops);
        let body_entry_cells = body_cells.clone();
        self.check_ops(
            &mut body_cells,
            &mut body_collection_slots,
            &mut body_aliases,
            &mut body_function_aliases,
            &mut body_pending_reallocs,
            &mut body_variant_initializations,
            body_ops,
            body_path.clone(),
        );
        let body_path_alternatives = core::mem::take(&mut self.path_alternatives);
        let body_states = path_alternatives_or_single(
            body_path_alternatives,
            body_cells,
            body_collection_slots,
            body_aliases,
            body_function_aliases,
            body_pending_reallocs,
            body_variant_initializations,
        );

        let backedge_requirements =
            loop_backedge_entry_requirements(condition_ops, condition, body_ops, span);
        self.report_loop_backedge_entry_conflicts(
            &body_entry_cells,
            &body_states,
            &backedge_requirements,
        );

        let mut loop_paths = Vec::with_capacity(body_states.len() + 1);
        loop_paths.push(ResourceCheckState::new(
            exit_cells,
            exit_collection_slots,
            exit_aliases,
            condition_function_aliases,
            exit_pending_reallocs,
            condition_variant_initializations,
        ));
        loop_paths.extend(body_states);

        merge_path_alternatives_into(
            &loop_paths,
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        );
        for candidate in initialized_range_candidates {
            collection_slots.mark_initialized_range_with_aliases(
                &candidate.storage,
                &candidate.initialized_count,
                candidate.value_ty,
                candidate.element_stride,
                raw_aliases,
            );
        }
        if path_states_need_replay(&loop_paths) {
            log_path_state_replay_reason(self.function, "loop", &loop_paths);
            self.path_alternatives = ResourcePathAlternatives::from_states(loop_paths);
        }
    }

    fn report_loop_backedge_entry_conflicts(
        &mut self,
        entry_cells: &CellTable,
        body_states: &[ResourceCheckState],
        requirements: &[LoopEntryRequirement],
    ) {
        let mut reported = Vec::new();
        for state in body_states {
            for requirement in requirements {
                if self.types.is_copy(requirement.place.ty)
                    || !matches!(
                        entry_cells.availability_state_with_types(self.types, &requirement.place),
                        CellState::Initialized(_)
                    )
                    || matches!(
                        state
                            .cells
                            .availability_state_with_types(self.types, &requirement.place),
                        CellState::Initialized(_)
                    )
                    || reported.iter().any(|place| place == &requirement.place)
                {
                    continue;
                }
                reported.push(requirement.place.clone());
                self.push_unavailable(
                    ResourceCheckOperation::LoopCondition,
                    &requirement.place,
                    CellState::MaybeMoved,
                    requirement.span,
                );
            }
        }
    }

    pub(super) fn check_match(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        scrutinee: &Place,
        scrutinee_is_borrow_target: bool,
        arms: &[ResourceMatchArm],
        span: Span,
        path: ResourceDropPointPath,
    ) {
        let scrutinee_available = if scrutinee_is_borrow_target {
            self.ensure_available(
                cells,
                scrutinee,
                ResourceCheckOperation::MatchScrutinee,
                span,
            )
        } else {
            self.consume_by_value(
                cells,
                scrutinee,
                ResourceCheckOperation::MatchScrutinee,
                span,
            )
        };
        let mut arms_available = true;
        let mut match_paths = Vec::new();

        for (arm_index, arm) in arms.iter().enumerate() {
            let mut arm_cells = cells.clone();
            let mut arm_collection_slots = collection_slots.clone();
            let mut arm_aliases = raw_aliases.clone();
            let mut arm_function_aliases = function_aliases.clone();
            let mut arm_pending_reallocs = pending_reallocs.clone();
            let mut arm_variant_initializations = variant_initializations.clone();
            if !arm_variant_initializations.match_arm_reachable(scrutinee, &arm.pattern) {
                continue;
            }
            if let Some(bind_local) = &arm.bind_local {
                arm_cells.mark_initialized(bind_local);
                if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                    self.copy_raw_alias_and_rekey_cells(
                        &mut arm_cells,
                        &mut arm_aliases,
                        &source,
                        bind_local,
                    );
                    arm_cells.transfer_raw_cell_loaded_value_origin(&source, bind_local);
                    transfer_slots(self, &mut arm_collection_slots, &source, bind_local, span);
                    arm_function_aliases.copy_alias(&source, bind_local);
                    arm_pending_reallocs.copy_result(&source, bind_local);
                    arm_variant_initializations.copy_result(&source, bind_local);
                } else {
                    arm_aliases.clear(bind_local);
                    arm_pending_reallocs.clear_result(bind_local);
                    arm_variant_initializations.clear_result(bind_local);
                }
            }
            arm_variant_initializations.apply_match_arm(
                self,
                &mut arm_cells,
                &mut arm_aliases,
                scrutinee,
                &arm.pattern,
                arm.span,
            );
            self.check_ops(
                &mut arm_cells,
                &mut arm_collection_slots,
                &mut arm_aliases,
                &mut arm_function_aliases,
                &mut arm_pending_reallocs,
                &mut arm_variant_initializations,
                &arm.ops,
                path.clone()
                    .with_step(ResourceDropPointStep::MatchArm { index: arm_index }),
            );
            let arm_path_alternatives = core::mem::take(&mut self.path_alternatives);
            if !self.place_is_never(&arm.value) {
                let arm_value = &arm.value;
                let arm_states = path_alternatives_or_single(
                    arm_path_alternatives,
                    arm_cells,
                    arm_collection_slots,
                    arm_aliases,
                    arm_function_aliases,
                    arm_pending_reallocs,
                    arm_variant_initializations,
                );
                match_paths.extend(self.transfer_control_value_path_states(
                    arm_states,
                    arm_value,
                    output,
                    ResourceCheckOperation::MatchValue,
                    arm.span,
                    &mut arms_available,
                ));
            }
        }

        if match_paths.is_empty() {
            arms_available = false;
        }
        let has_match_paths = !match_paths.is_empty();
        if has_match_paths {
            merge_path_alternatives_into(
                &match_paths,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            );
            if path_states_need_replay(&match_paths) {
                log_path_state_replay_reason(self.function, "match", &match_paths);
                self.path_alternatives = ResourcePathAlternatives::from_states(match_paths);
            }
        }
        if scrutinee_available && arms_available {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            invalidate_control_output_state(
                cells,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                output,
            );
            invalidate_control_output_path_states(&mut self.path_alternatives, output);
        }
    }

    pub(super) fn transfer_control_value_path_states(
        &mut self,
        mut states: Vec<ResourceCheckState>,
        value: &Place,
        output: &Place,
        operation: ResourceCheckOperation,
        span: Span,
        paths_available: &mut bool,
    ) -> Vec<ResourceCheckState> {
        for state in &mut states {
            let available = self.consume_by_value(&mut state.cells, value, operation, span);
            *paths_available &= available;
            if available {
                state
                    .cells
                    .set_state(output, CellState::Initialized(output.ty));
                self.copy_raw_alias_and_rekey_cells_preferring_target(
                    &mut state.cells,
                    &mut state.raw_aliases,
                    value,
                    output,
                );
                state
                    .cells
                    .copy_initialized_raw_byte_ranges_through_value_aliases(
                        value,
                        output,
                        &state.raw_aliases,
                    );
                state
                    .cells
                    .transfer_raw_cell_loaded_value_origin(value, output);
                transfer_slots(self, &mut state.collection_slots, value, output, span);
                state.function_aliases.copy_alias(value, output);
                state.pending_reallocs.copy_result(value, output);
                state.variant_initializations.copy_result(value, output);
                seed_str_storage_layout(
                    self.types,
                    &mut state.cells,
                    &mut state.raw_aliases,
                    output,
                );
            } else {
                invalidate_control_output_state(
                    &mut state.cells,
                    &mut state.raw_aliases,
                    &mut state.function_aliases,
                    &mut state.pending_reallocs,
                    &mut state.variant_initializations,
                    output,
                );
            }
        }
        states
    }

    fn apply_realloc_condition_fact(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        fact: Option<&ResourceConditionFact>,
        truthy_path: bool,
    ) {
        let Some(fact) = fact else {
            return;
        };
        let Some((place, outcome)) = raw_realloc_condition_outcome(fact, truthy_path) else {
            return;
        };
        let Some(pending) = pending_reallocs.take_for_result(place) else {
            return;
        };
        match outcome {
            RawReallocConditionOutcome::Success => {
                cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::ReallocSuccessTransfer {
                        source: &pending.source,
                        result: &pending.result,
                        collection_managed_non_copy_cells: &pending
                            .collection_managed_non_copy_cells,
                    },
                    raw_aliases,
                    self.types,
                );
                pending_reallocs.certify_success(&pending.storage_source, &pending.result);
                raw_aliases.clear(&pending.source);
                raw_aliases.mark(&pending.result);
            }
            RawReallocConditionOutcome::Failure => {
                raw_aliases.clear(&pending.result);
            }
        }
    }

    pub(super) fn apply_branch_condition_fact(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        fact: Option<&ResourceConditionFact>,
        truthy_path: bool,
    ) {
        record_initialized_condition_fact(raw_aliases, fact, truthy_path);
        self.apply_realloc_condition_fact(cells, raw_aliases, pending_reallocs, fact, truthy_path);
    }

    pub(super) fn place_is_never(&self, place: &Place) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(place.ty)),
            TypeKind::Never
        )
    }
}

pub(super) fn path_alternatives_or_single(
    path_alternatives: ResourcePathAlternatives,
    cells: CellTable,
    collection_slots: CollectionSlotStateTable,
    raw_aliases: RawCellAddressAliases,
    function_aliases: FunctionAliasTable,
    pending_reallocs: PendingRawReallocs,
    variant_initializations: PendingVariantRawCellInitializations,
) -> Vec<ResourceCheckState> {
    match path_alternatives {
        ResourcePathAlternatives::None => {
            let mut out = Vec::new();
            out.push(ResourceCheckState::new(
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            ));
            out
        }
        ResourcePathAlternatives::Feasible(states) => states,
    }
}

pub(super) fn invalidate_control_output_path_states(
    alternatives: &mut ResourcePathAlternatives,
    output: &Place,
) {
    match alternatives {
        ResourcePathAlternatives::None => {}
        ResourcePathAlternatives::Feasible(states) => {
            for state in states {
                invalidate_control_output_state(
                    &mut state.cells,
                    &mut state.raw_aliases,
                    &mut state.function_aliases,
                    &mut state.pending_reallocs,
                    &mut state.variant_initializations,
                    output,
                );
            }
        }
    }
}

pub(super) fn invalidate_control_output_state(
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    output: &Place,
) {
    cells.set_state(output, CellState::Uninit);
    cells.clear_initialized_raw_byte_ranges_through_value(output);
    raw_aliases.clear(output);
    function_aliases.clear_alias(output);
    pending_reallocs.clear_result(output);
    variant_initializations.clear_result(output);
}

fn record_initialized_condition_fact(
    raw_aliases: &mut RawCellAddressAliases,
    fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
) {
    let Some(fact) = fact else {
        return;
    };
    record_condition_fact_value_constraints(raw_aliases, fact, truthy_path);
}

struct LoopInitializedRangeCandidate {
    storage: Place,
    initialized_count: Place,
    value_ty: crate::types::TypeId,
    element_stride: usize,
}

struct LoopInitializationTarget {
    target: Place,
    value_ty: crate::types::TypeId,
    offset_source: Option<Place>,
}

/// `while i < n` の形で、0 から 1 ずつ進む index に対応する slot を
/// 各反復で初期化している loop を検出する。
///
/// この証明により、loop の path merge が個別 slot を MaybeInitialized にしても、
/// loop 完了後の `0..n` 範囲は initialized range として扱える。
fn loop_initialized_range_candidates(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    condition_fact: Option<&ResourceConditionFact>,
    body_ops: &[ResourceOp],
) -> Vec<LoopInitializedRangeCandidate> {
    let Some((index, initialized_count)) =
        initialized_range_loop_bound(raw_aliases, condition_fact)
    else {
        return Vec::new();
    };
    let index = raw_aliases.canonicalize_scalar(&index);
    if raw_aliases.i32_value(&index) != Some(0) {
        return Vec::new();
    }
    let Some(step_index) =
        loop_body_increment_step_for_initialization(engine, raw_aliases, body_ops, &index)
    else {
        return Vec::new();
    };
    let initialized_count = raw_aliases.canonicalize_scalar(&initialized_count);
    let mut out = Vec::new();
    let mut scan_aliases = raw_aliases.clone();
    let mut scan_function_aliases = FunctionAliasTable::default();
    for (op_index, op) in body_ops[..step_index].iter().enumerate() {
        for event_target in loop_initialization_targets_from_op(engine, &scan_aliases, op) {
            let Some((storage, element_stride)) = initialized_loop_slot_storage(
                engine.types,
                &event_target.target,
                event_target.offset_source.as_ref(),
                &index,
                &scan_aliases,
            ) else {
                continue;
            };
            if element_stride == 0
                || element_stride != storage_size_bytes(engine.types, event_target.value_ty)
            {
                continue;
            }
            let tail = &body_ops[op_index + 1..];
            if !body_preserves_place(engine, &scan_aliases, tail, &storage)
                || !body_preserves_place(engine, &scan_aliases, tail, &initialized_count)
            {
                continue;
            }
            push_loop_initialized_range_candidate(
                &mut out,
                LoopInitializedRangeCandidate {
                    storage,
                    initialized_count: initialized_count.clone(),
                    value_ty: event_target.value_ty,
                    element_stride,
                },
            );
        }
        propagate_i32_scalar_ops(
            &mut scan_aliases,
            &mut scan_function_aliases,
            core::slice::from_ref(op),
            engine.i32_scalar_summaries,
            engine.raw_alias_summaries,
            engine.types,
        );
    }
    out
}

fn loop_initialization_targets_from_op(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
) -> Vec<LoopInitializationTarget> {
    let mut out = Vec::new();
    match op {
        ResourceOp::CollectionSlotLifecycle {
            target,
            event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
            ..
        } => {
            out.push(LoopInitializationTarget {
                target: target.clone(),
                value_ty: *value_ty,
                offset_source: None,
            });
        }
        ResourceOp::Call { target, args, .. } => {
            let super::model::ResourceCallTarget::User { name, .. } = target else {
                return out;
            };
            let Some(summary) = engine.collection_slot_summaries.get(name) else {
                return out;
            };
            for summary_op in &summary.ops {
                let CollectionSlotLifecycleSummaryOp::Event {
                    target,
                    event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
                    ..
                } = summary_op
                else {
                    continue;
                };
                let Some(target) =
                    instantiate_summary_target_with_aliases(engine, args, raw_aliases, target)
                else {
                    continue;
                };
                out.push(LoopInitializationTarget {
                    target,
                    value_ty: *value_ty,
                    offset_source: summary_storage_offset_source(summary_op, args),
                });
            }
        }
        ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. }
        | ResourceOp::Branch { .. }
        | ResourceOp::Loop { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::Expr { .. } => {}
    }
    out
}

fn summary_storage_offset_source(
    summary_op: &CollectionSlotLifecycleSummaryOp,
    args: &[Place],
) -> Option<Place> {
    let CollectionSlotLifecycleSummaryOp::Event { target, .. } = summary_op else {
        return None;
    };
    let offset = target
        .suffix
        .iter()
        .rev()
        .find_map(|projection| match projection {
            SummaryProjection::StorageOffset(offset) => Some(offset),
            SummaryProjection::Field { .. }
            | SummaryProjection::TupleField { .. }
            | SummaryProjection::EnumPayload { .. }
            | SummaryProjection::Deref => None,
        })?;
    let source = summary_offset_source(offset)?;
    if !source.suffix.is_empty() {
        return None;
    }
    args.get(source.parameter_index).cloned()
}

fn summary_offset_source(
    offset: &SummaryOffset,
) -> Option<&super::summary_projection::SummaryPlace> {
    match offset {
        SummaryOffset::Symbolic { place }
        | SummaryOffset::ScaledSymbolic { place, .. }
        | SummaryOffset::Offset { place, .. }
        | SummaryOffset::ScaledOffset { place, .. } => Some(place),
        SummaryOffset::Known(_) | SummaryOffset::Unknown => None,
    }
}

fn initialized_loop_slot_storage(
    types: &crate::types::TypeCtx,
    target: &Place,
    offset_source: Option<&Place>,
    index: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<(Place, usize)> {
    let mut address = target.clone();
    if matches!(address.projections.last(), Some(PlaceProjection::Deref)) {
        address.projections.pop();
    }
    let projection = address.projections.pop()?;
    match projection {
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale }) => {
            let offset_place = raw_aliases.canonicalize_scalar(&place);
            let index = raw_aliases.canonicalize_scalar(index);
            (offset_place == index).then_some((address, scale))
        }
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }) => {
            let (source, scale) = raw_aliases.i32_scaled_source(&place)?;
            let index = raw_aliases.canonicalize_scalar(index);
            (source == index).then_some((address, scale))
        }
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)) => {
            let offset_source = offset_source?;
            let index = raw_aliases.canonicalize_scalar(index);
            if let Some((source, scale)) = raw_aliases.i32_scaled_source(offset_source) {
                if source == index {
                    return Some((address, scale));
                }
            }
            let (source, element_ty) = raw_aliases.i32_type_size_scaled_source(offset_source)?;
            (source == index).then(|| (address, storage_size_bytes(types, element_ty)))
        }
        PlaceProjection::StorageOffset(ResourceOffset::Known(_))
        | PlaceProjection::StorageOffset(ResourceOffset::Offset { .. })
        | PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset { .. })
        | PlaceProjection::StorageOffset(ResourceOffset::Unknown)
        | PlaceProjection::Field { .. }
        | PlaceProjection::TupleField { .. }
        | PlaceProjection::EnumPayload { .. }
        | PlaceProjection::Deref => None,
    }
}

fn push_loop_initialized_range_candidate(
    out: &mut Vec<LoopInitializedRangeCandidate>,
    candidate: LoopInitializedRangeCandidate,
) {
    if !out.iter().any(|existing| {
        existing.storage == candidate.storage
            && existing.initialized_count == candidate.initialized_count
            && existing.value_ty == candidate.value_ty
            && existing.element_stride == candidate.element_stride
    }) {
        out.push(candidate);
    }
}

fn loop_body_increment_step_for_initialization(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    index: &Place,
) -> Option<usize> {
    let mut index_aliases = Vec::new();
    index_aliases.push(index.clone());
    let mut one_values = Vec::new();
    let mut increment_values = Vec::new();
    let mut step = None;
    for (op_index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Expr { kind, output, .. } => {
                match loop_step_expr_effect(kind, output, index) {
                    LoopStepExprEffect::Marker => {}
                    LoopStepExprEffect::Reject => return None,
                    LoopStepExprEffect::LiteralOne(output) => {
                        clear_loop_step_place(
                            output,
                            &mut index_aliases,
                            &mut one_values,
                            &mut increment_values,
                        );
                        push_loop_step_place(&mut one_values, output);
                    }
                    LoopStepExprEffect::Clear(output) => clear_loop_step_place(
                        output,
                        &mut index_aliases,
                        &mut one_values,
                        &mut increment_values,
                    ),
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                if loop_step_place_in(&index_aliases, source) {
                    push_loop_step_place(&mut index_aliases, output);
                } else {
                    clear_loop_step_place(
                        output,
                        &mut index_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                ..
            } => {
                if !effect_is_proof_pure(effect) {
                    if loop_initialization_targets_from_op(engine, raw_aliases, op).is_empty()
                        || loop_step_place_touches(output, index)
                    {
                        return None;
                    }
                    clear_loop_step_place(
                        output,
                        &mut index_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    continue;
                }
                if loop_step_call_adds_one_to_index(target, args, &index_aliases, &one_values) {
                    push_loop_step_place(&mut increment_values, output);
                } else {
                    clear_loop_step_place(
                        output,
                        &mut index_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Assign { target, value, .. } if *target == *index => {
                if step.is_some() || !loop_step_place_in(&increment_values, value) {
                    return None;
                }
                step = Some(op_index);
                index_aliases.clear();
                push_loop_step_place(&mut index_aliases, target);
            }
            ResourceOp::Assign { target, .. } if loop_step_place_touches(target, index) => {
                return None;
            }
            ResourceOp::Assign { .. } => {}
            ResourceOp::RawMemory {
                operation, output, ..
            } => {
                if matches!(operation, RawMemoryOp::Load) {
                    clear_loop_step_place(
                        output,
                        &mut index_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::RawAddressAlias { target, .. }
            | ResourceOp::RawAddressView { target, .. }
            | ResourceOp::StorageOrigin { target, .. } => {
                if loop_step_place_touches(target, index) {
                    return None;
                }
            }
            ResourceOp::CollectionSlotLifecycle {
                event: CollectionSlotLifecycleEvent::InitializeEmpty { .. },
                ..
            }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::EndScope { .. } => {}
            ResourceOp::DeclareLocal { place, .. }
            | ResourceOp::Drop { place, .. }
            | ResourceOp::Borrow { output: place, .. }
            | ResourceOp::FunctionValue { output: place, .. }
            | ResourceOp::Construct { output: place, .. } => clear_loop_step_place(
                place,
                &mut index_aliases,
                &mut one_values,
                &mut increment_values,
            ),
            ResourceOp::Branch { .. }
            | ResourceOp::Loop { .. }
            | ResourceOp::Match { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. } => return None,
        }
    }
    step
}

fn loop_step_call_adds_one_to_index(
    target: &super::model::ResourceCallTarget,
    args: &[Place],
    index_aliases: &[Place],
    one_values: &[Place],
) -> bool {
    if super::scalar_primitive::I32ArithmeticPrimitive::from_resource_call_target(target)
        != Some(super::scalar_primitive::I32ArithmeticPrimitive::Add)
    {
        return false;
    }
    let [left, right] = args else {
        return false;
    };
    (loop_step_place_in(index_aliases, left) && loop_step_place_in(one_values, right))
        || (loop_step_place_in(index_aliases, right) && loop_step_place_in(one_values, left))
}

fn clear_loop_step_place(
    place: &Place,
    index_aliases: &mut Vec<Place>,
    one_values: &mut Vec<Place>,
    increment_values: &mut Vec<Place>,
) {
    index_aliases.retain(|existing| existing != place);
    one_values.retain(|existing| existing != place);
    increment_values.retain(|existing| existing != place);
}

fn loop_step_place_touches(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}

fn loop_step_place_in(places: &[Place], place: &Place) -> bool {
    places.iter().any(|existing| existing == place)
}

fn push_loop_step_place(places: &mut Vec<Place>, place: &Place) {
    if !loop_step_place_in(places, place) {
        places.push(place.clone());
    }
}

struct LoopEntryRequirement {
    place: Place,
    span: Span,
}

fn loop_backedge_entry_requirements(
    condition_ops: &[ResourceOp],
    condition: &Place,
    body_ops: &[ResourceOp],
    loop_span: Span,
) -> Vec<LoopEntryRequirement> {
    let mut requirements = Vec::new();
    let mut defined = Vec::new();
    collect_loop_entry_requirements(condition_ops, &mut defined, &mut requirements);
    push_loop_entry_requirement(&mut requirements, &defined, condition, loop_span);
    collect_loop_entry_requirements(body_ops, &mut defined, &mut requirements);
    requirements
}

fn collect_loop_entry_requirements(
    ops: &[ResourceOp],
    defined: &mut Vec<Place>,
    requirements: &mut Vec<LoopEntryRequirement>,
) {
    for op in ops {
        match op {
            ResourceOp::Expr { output, .. } | ResourceOp::FunctionValue { output, .. } => {
                push_loop_defined_place(defined, output);
            }
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    push_loop_entry_requirement(requirements, defined, initializer, *span);
                }
                push_loop_defined_place(defined, place);
            }
            ResourceOp::Read {
                source,
                output,
                span,
            }
            | ResourceOp::Move {
                source,
                output,
                span,
            }
            | ResourceOp::RawAddressAlias {
                source,
                target: output,
                span,
                ..
            }
            | ResourceOp::RawAddressView {
                source,
                target: output,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, source, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                push_loop_entry_requirement(requirements, defined, value, *span);
                push_loop_defined_place(defined, target);
            }
            ResourceOp::Borrow {
                source,
                output,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, source, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::Drop { place, span } => {
                push_loop_entry_requirement(requirements, defined, place, *span);
                push_loop_defined_place(defined, place);
            }
            ResourceOp::EndScope { result, span, .. } => {
                if let Some(result) = result {
                    push_loop_entry_requirement(requirements, defined, result, *span);
                }
            }
            ResourceOp::Call {
                output, args, span, ..
            } => {
                push_loop_entry_requirements(requirements, defined, args, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, callee, *span);
                push_loop_entry_requirements(requirements, defined, args, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::RawMemory {
                output, args, span, ..
            } => {
                push_loop_entry_requirements(requirements, defined, args, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::StorageOrigin { .. } | ResourceOp::CallEffect { .. } => {}
            ResourceOp::CollectionSlotLifecycle { target, span, .. } => {
                push_loop_entry_requirement(requirements, defined, target, *span);
            }
            ResourceOp::CollectionStorageRelocate {
                old_storage,
                new_storage,
                span,
            } => {
                push_loop_entry_requirement(requirements, defined, old_storage, *span);
                push_loop_entry_requirement(requirements, defined, new_storage, *span);
            }
            ResourceOp::CollectionSlotDropTraversal {
                storage,
                initialized_count,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, storage, *span);
                push_loop_entry_requirement(requirements, defined, initialized_count, *span);
            }
            ResourceOp::CollectionSlotTransformRange {
                source_storage,
                source_initialized_count,
                output_storage,
                output_initialized_count,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, source_storage, *span);
                push_loop_entry_requirement(requirements, defined, source_initialized_count, *span);
                push_loop_entry_requirement(requirements, defined, output_storage, *span);
                push_loop_entry_requirement(requirements, defined, output_initialized_count, *span);
            }
            ResourceOp::Construct {
                output,
                inputs,
                span,
                ..
            } => {
                push_loop_entry_requirements(requirements, defined, inputs, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::Branch {
                output,
                condition,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, condition, *span);
                let mut then_defined = defined.clone();
                collect_loop_entry_requirements(then_ops, &mut then_defined, requirements);
                push_loop_entry_requirement(requirements, &then_defined, then_value, *span);
                let mut else_defined = defined.clone();
                collect_loop_entry_requirements(else_ops, &mut else_defined, requirements);
                push_loop_entry_requirement(requirements, &else_defined, else_value, *span);
                push_loop_defined_place(defined, output);
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span,
                ..
            } => {
                let nested =
                    loop_backedge_entry_requirements(condition_ops, condition, body_ops, *span);
                for requirement in nested {
                    push_loop_entry_requirement(
                        requirements,
                        defined,
                        &requirement.place,
                        requirement.span,
                    );
                }
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
                ..
            } => {
                push_loop_entry_requirement(requirements, defined, scrutinee, *span);
                for arm in arms {
                    let mut arm_defined = defined.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        push_loop_defined_place(&mut arm_defined, bind_local);
                    }
                    collect_loop_entry_requirements(&arm.ops, &mut arm_defined, requirements);
                    push_loop_entry_requirement(requirements, &arm_defined, &arm.value, arm.span);
                }
                push_loop_defined_place(defined, output);
            }
        }
    }
}

fn push_loop_entry_requirements(
    requirements: &mut Vec<LoopEntryRequirement>,
    defined: &[Place],
    places: &[Place],
    span: Span,
) {
    for place in places {
        push_loop_entry_requirement(requirements, defined, place, span);
    }
}

fn push_loop_entry_requirement(
    requirements: &mut Vec<LoopEntryRequirement>,
    defined: &[Place],
    place: &Place,
    span: Span,
) {
    if loop_place_is_defined_by(defined, place)
        || requirements
            .iter()
            .any(|requirement| requirement.place == *place)
    {
        return;
    }
    requirements.push(LoopEntryRequirement {
        place: place.clone(),
        span,
    });
}

fn push_loop_defined_place(defined: &mut Vec<Place>, place: &Place) {
    if defined.iter().any(|defined| defined == place) {
        return;
    }
    defined.push(place.clone());
}

fn loop_place_is_defined_by(defined: &[Place], place: &Place) -> bool {
    defined
        .iter()
        .any(|defined| defined == place || place_suffix_after_prefix(place, defined).is_some())
}

#[cfg(test)]
mod tests {
    use super::super::model::ResourceI32RelationOp::{Ge, Lt};
    use super::*;
    use crate::types::TypeId;
    use alloc::string::String;

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    #[test]
    fn initialized_branch_condition_fact_records_i32_relation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_initialized_condition_fact(&mut raw_aliases, Some(&fact), true);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Lt, &right),
            Some(true)
        );
    }

    #[test]
    fn initialized_branch_condition_fact_records_false_relation_negation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_initialized_condition_fact(&mut raw_aliases, Some(&fact), false);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Ge, &right),
            Some(true)
        );
    }
}

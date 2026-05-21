use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeKind;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::condition_fact::record_condition_fact_value_constraints;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_control_slot_transfer::transfer_control_value_slots as transfer_slots;
use super::initialized_path_state::{
    merge_path_alternatives_into, ResourceCheckState, ResourcePathAlternatives,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::place_utils::match_bind_payload_place;
use super::raw_cell_lifecycle::RawCellLifecycleEvent;
use super::raw_realloc::{
    raw_realloc_condition_outcome, PendingRawReallocs, RawReallocConditionOutcome,
};
use super::report::ResourceCheckOperation;

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

        if !branch_paths.is_empty() {
            self.path_alternatives = ResourcePathAlternatives::from_states(branch_paths.clone());
            merge_path_alternatives_into(
                &branch_paths,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            );
        }
        if paths_available && !branch_paths.is_empty() {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            invalidate_control_output_path_states(&mut self.path_alternatives, output);
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
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
            condition_path,
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
        self.check_ops(
            &mut body_cells,
            &mut body_collection_slots,
            &mut body_aliases,
            &mut body_function_aliases,
            &mut body_pending_reallocs,
            &mut body_variant_initializations,
            body_ops,
            body_path,
        );
        core::mem::take(&mut self.path_alternatives);
        let cell_paths = [exit_cells, body_cells];
        let collection_slot_paths = [exit_collection_slots, body_collection_slots];
        let alias_paths = [exit_aliases, body_aliases];
        let merged_raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
        *cells =
            CellTable::merge_paths_with_raw_aliases(&cell_paths, &alias_paths, &merged_raw_aliases);
        *raw_aliases = merged_raw_aliases;
        *collection_slots = CollectionSlotStateTable::merge_paths(&collection_slot_paths);
        *function_aliases =
            FunctionAliasTable::merge_paths(&[condition_function_aliases, body_function_aliases]);
        *pending_reallocs =
            PendingRawReallocs::merge_paths(&[exit_pending_reallocs, body_pending_reallocs]);
        *variant_initializations = PendingVariantRawCellInitializations::merge_paths(&[
            condition_variant_initializations,
            body_variant_initializations,
        ]);
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
        if !match_paths.is_empty() {
            self.path_alternatives = ResourcePathAlternatives::from_states(match_paths.clone());
            merge_path_alternatives_into(
                &match_paths,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            );
        }
        if scrutinee_available && arms_available {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            invalidate_control_output_path_states(&mut self.path_alternatives, output);
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
        }
    }

    fn transfer_control_value_path_states(
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
                state.cells.set_state(output, CellState::Uninit);
                state.raw_aliases.clear(output);
                state.function_aliases.clear_alias(output);
                state.pending_reallocs.clear_result(output);
                state.variant_initializations.clear_result(output);
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
                    },
                    raw_aliases,
                    self.types,
                );
                pending_reallocs.certify_success(&pending.source, &pending.result);
                raw_aliases.clear(&pending.source);
                raw_aliases.mark(&pending.result);
            }
            RawReallocConditionOutcome::Failure => {
                raw_aliases.clear(&pending.result);
            }
        }
    }

    fn apply_branch_condition_fact(
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

    fn place_is_never(&self, place: &Place) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(place.ty)),
            TypeKind::Never
        )
    }
}

fn path_alternatives_or_single(
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

fn invalidate_control_output_path_states(
    alternatives: &mut ResourcePathAlternatives,
    output: &Place,
) {
    match alternatives {
        ResourcePathAlternatives::None => {}
        ResourcePathAlternatives::Feasible(states) => {
            for state in states {
                state.cells.set_state(output, CellState::Uninit);
                state.raw_aliases.clear(output);
                state.function_aliases.clear_alias(output);
                state.pending_reallocs.clear_result(output);
                state.variant_initializations.clear_result(output);
            }
        }
    }
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

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeKind;

use super::cell_state::CellTable;
use super::condition_fact::record_condition_fact_value_constraints;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::place_utils::match_bind_payload_place;
use super::raw_realloc::{
    raw_realloc_condition_outcome, PendingRawReallocs, RawReallocConditionOutcome,
};
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_branch(
        &mut self,
        cells: &mut CellTable,
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
        let mut then_cells = cells.clone();
        let mut else_cells = cells.clone();
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
            &mut then_aliases,
            &mut then_function_aliases,
            &mut then_pending_reallocs,
            &mut then_variant_initializations,
            then_ops,
            then_path,
        );
        self.check_ops(
            &mut else_cells,
            &mut else_aliases,
            &mut else_function_aliases,
            &mut else_pending_reallocs,
            &mut else_variant_initializations,
            else_ops,
            else_path,
        );

        let mut cell_paths = Vec::new();
        let mut alias_paths = Vec::new();
        let mut function_alias_paths = Vec::new();
        let mut pending_realloc_paths = Vec::new();
        let mut variant_initialization_paths = Vec::new();
        let mut paths_available = condition_available;
        if !self.place_is_never(then_value) {
            let then_available = self.consume_by_value(
                &mut then_cells,
                then_value,
                ResourceCheckOperation::BranchValue,
                span,
            );
            paths_available &= then_available;
            if then_available {
                self.copy_raw_alias_and_rekey_cells_preferring_target(
                    &mut then_cells,
                    &mut then_aliases,
                    then_value,
                    output,
                );
                then_cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                    then_value,
                    output,
                    &then_aliases,
                );
                then_function_aliases.copy_alias(then_value, output);
                then_pending_reallocs.copy_result(then_value, output);
                then_variant_initializations.copy_result(then_value, output);
            }
            cell_paths.push(then_cells);
            alias_paths.push(then_aliases);
            function_alias_paths.push(then_function_aliases);
            pending_realloc_paths.push(then_pending_reallocs);
            variant_initialization_paths.push(then_variant_initializations);
        }
        if !self.place_is_never(else_value) {
            let else_available = self.consume_by_value(
                &mut else_cells,
                else_value,
                ResourceCheckOperation::BranchValue,
                span,
            );
            paths_available &= else_available;
            if else_available {
                self.copy_raw_alias_and_rekey_cells_preferring_target(
                    &mut else_cells,
                    &mut else_aliases,
                    else_value,
                    output,
                );
                else_cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                    else_value,
                    output,
                    &else_aliases,
                );
                else_function_aliases.copy_alias(else_value, output);
                else_pending_reallocs.copy_result(else_value, output);
                else_variant_initializations.copy_result(else_value, output);
            }
            cell_paths.push(else_cells);
            alias_paths.push(else_aliases);
            function_alias_paths.push(else_function_aliases);
            pending_realloc_paths.push(else_pending_reallocs);
            variant_initialization_paths.push(else_variant_initializations);
        }

        if !cell_paths.is_empty() {
            let merged_raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
            *cells = CellTable::merge_paths_with_raw_aliases(
                &cell_paths,
                &alias_paths,
                &merged_raw_aliases,
            );
            *raw_aliases = merged_raw_aliases;
            *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
            *variant_initializations =
                PendingVariantRawCellInitializations::merge_paths(&variant_initialization_paths);
        }
        if paths_available && !cell_paths.is_empty() {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
        }
    }

    pub(super) fn check_loop(
        &mut self,
        cells: &mut CellTable,
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
        let mut condition_aliases = raw_aliases.clone();
        let mut condition_function_aliases = function_aliases.clone();
        let mut condition_pending_reallocs = pending_reallocs.clone();
        let mut condition_variant_initializations = variant_initializations.clone();
        self.check_ops(
            &mut condition_cells,
            &mut condition_aliases,
            &mut condition_function_aliases,
            &mut condition_pending_reallocs,
            &mut condition_variant_initializations,
            condition_ops,
            condition_path,
        );
        self.consume_by_value(
            &mut condition_cells,
            condition,
            ResourceCheckOperation::LoopCondition,
            span,
        );

        let mut exit_cells = condition_cells.clone();
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
            &mut body_aliases,
            &mut body_function_aliases,
            &mut body_pending_reallocs,
            &mut body_variant_initializations,
            body_ops,
            body_path,
        );
        let cell_paths = [exit_cells, body_cells];
        let alias_paths = [exit_aliases, body_aliases];
        let merged_raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
        *cells =
            CellTable::merge_paths_with_raw_aliases(&cell_paths, &alias_paths, &merged_raw_aliases);
        *raw_aliases = merged_raw_aliases;
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
        let mut arm_paths = Vec::new();
        let mut alias_paths = Vec::new();
        let mut function_alias_paths = Vec::new();
        let mut pending_realloc_paths = Vec::new();
        let mut variant_initialization_paths = Vec::new();

        for (arm_index, arm) in arms.iter().enumerate() {
            let mut arm_cells = cells.clone();
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
                &mut arm_aliases,
                &mut arm_function_aliases,
                &mut arm_pending_reallocs,
                &mut arm_variant_initializations,
                &arm.ops,
                path.clone()
                    .with_step(ResourceDropPointStep::MatchArm { index: arm_index }),
            );
            if !self.place_is_never(&arm.value) {
                let arm_available = self.consume_by_value(
                    &mut arm_cells,
                    &arm.value,
                    ResourceCheckOperation::MatchValue,
                    arm.span,
                );
                arms_available &= arm_available;
                if arm_available {
                    self.copy_raw_alias_and_rekey_cells_preferring_target(
                        &mut arm_cells,
                        &mut arm_aliases,
                        &arm.value,
                        output,
                    );
                    arm_cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                        &arm.value,
                        output,
                        &arm_aliases,
                    );
                    arm_function_aliases.copy_alias(&arm.value, output);
                    arm_pending_reallocs.copy_result(&arm.value, output);
                    arm_variant_initializations.copy_result(&arm.value, output);
                }
                arm_paths.push(arm_cells);
                alias_paths.push(arm_aliases);
                function_alias_paths.push(arm_function_aliases);
                pending_realloc_paths.push(arm_pending_reallocs);
                variant_initialization_paths.push(arm_variant_initializations);
            }
        }

        if arm_paths.is_empty() {
            arms_available = false;
        }
        if !arm_paths.is_empty() {
            let merged_raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
            *cells = CellTable::merge_paths_with_raw_aliases(
                &arm_paths,
                &alias_paths,
                &merged_raw_aliases,
            );
            *raw_aliases = merged_raw_aliases;
            *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
            *variant_initializations =
                PendingVariantRawCellInitializations::merge_paths(&variant_initialization_paths);
        }
        if scrutinee_available && arms_available {
            cells.set_state(output, CellState::Initialized(output.ty));
            seed_str_storage_layout(self.types, cells, raw_aliases, output);
        } else {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
        }
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
                let source_owned = cells.owns_raw_storage_under(&pending.source);
                let relocated = cells.copy_initialized_copy_raw_cells(
                    &pending.source,
                    &pending.result,
                    self.types,
                );
                let relocated_ranges =
                    cells.copy_initialized_raw_byte_ranges_under(&pending.source, &pending.result);
                cells.clear_raw_cells_under(&pending.source);
                cells.release_owned_raw_storage_under(&pending.source);
                cells.mark_initialized(&pending.result);
                if source_owned {
                    cells.mark_owned_raw_storage_root(&pending.result);
                }
                cells.extend_entries(relocated);
                cells.extend_initialized_raw_byte_ranges(relocated_ranges);
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

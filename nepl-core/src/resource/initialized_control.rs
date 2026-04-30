use alloc::vec::Vec;

use crate::span::Span;

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
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
        output: &Place,
        condition: &Place,
        condition_fact: Option<&ResourceConditionFact>,
        then_ops: &[ResourceOp],
        then_value: &Place,
        else_ops: &[ResourceOp],
        else_value: &Place,
        span: Span,
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

        self.apply_realloc_condition_fact(
            &mut then_cells,
            &mut then_aliases,
            &mut then_pending_reallocs,
            condition_fact,
            true,
        );
        self.apply_realloc_condition_fact(
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
            then_ops,
        );
        self.check_ops(
            &mut else_cells,
            &mut else_aliases,
            &mut else_function_aliases,
            &mut else_pending_reallocs,
            else_ops,
        );

        let then_available = self.consume_by_value(
            &mut then_cells,
            then_value,
            ResourceCheckOperation::BranchValue,
            span,
        );
        let else_available = self.consume_by_value(
            &mut else_cells,
            else_value,
            ResourceCheckOperation::BranchValue,
            span,
        );
        if then_available {
            self.copy_raw_alias_and_rekey_cells_preferring_target(
                &mut then_cells,
                &mut then_aliases,
                then_value,
                output,
            );
            then_function_aliases.copy_alias(then_value, output);
            then_pending_reallocs.copy_result(then_value, output);
        }
        if else_available {
            self.copy_raw_alias_and_rekey_cells_preferring_target(
                &mut else_cells,
                &mut else_aliases,
                else_value,
                output,
            );
            else_function_aliases.copy_alias(else_value, output);
            else_pending_reallocs.copy_result(else_value, output);
        }

        *cells = CellTable::merge_paths(&[then_cells, else_cells]);
        *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
        *function_aliases =
            FunctionAliasTable::merge_paths(&[then_function_aliases, else_function_aliases]);
        *pending_reallocs =
            PendingRawReallocs::merge_paths(&[then_pending_reallocs, else_pending_reallocs]);
        if condition_available && then_available && else_available {
            cells.set_state(output, CellState::Initialized(output.ty));
        } else {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
        }
    }

    pub(super) fn check_loop(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        condition_ops: &[ResourceOp],
        condition: &Place,
        body_ops: &[ResourceOp],
        span: Span,
    ) {
        let mut condition_cells = cells.clone();
        let mut condition_aliases = raw_aliases.clone();
        let mut condition_function_aliases = function_aliases.clone();
        let mut condition_pending_reallocs = pending_reallocs.clone();
        self.check_ops(
            &mut condition_cells,
            &mut condition_aliases,
            &mut condition_function_aliases,
            &mut condition_pending_reallocs,
            condition_ops,
        );
        self.consume_by_value(
            &mut condition_cells,
            condition,
            ResourceCheckOperation::LoopCondition,
            span,
        );

        let mut body_cells = condition_cells.clone();
        let mut body_aliases = condition_aliases.clone();
        let mut body_function_aliases = condition_function_aliases.clone();
        let mut body_pending_reallocs = condition_pending_reallocs.clone();
        self.check_ops(
            &mut body_cells,
            &mut body_aliases,
            &mut body_function_aliases,
            &mut body_pending_reallocs,
            body_ops,
        );
        *cells = CellTable::merge_paths(&[condition_cells, body_cells]);
        *raw_aliases = RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
        *function_aliases =
            FunctionAliasTable::merge_paths(&[condition_function_aliases, body_function_aliases]);
        *pending_reallocs =
            PendingRawReallocs::merge_paths(&[condition_pending_reallocs, body_pending_reallocs]);
    }

    pub(super) fn check_match(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        output: &Place,
        scrutinee: &Place,
        arms: &[ResourceMatchArm],
        span: Span,
    ) {
        let scrutinee_available = self.consume_by_value(
            cells,
            scrutinee,
            ResourceCheckOperation::MatchScrutinee,
            span,
        );
        let mut arms_available = true;
        let mut arm_paths = Vec::new();
        let mut alias_paths = Vec::new();
        let mut function_alias_paths = Vec::new();
        let mut pending_realloc_paths = Vec::new();

        for arm in arms {
            let mut arm_cells = cells.clone();
            let mut arm_aliases = raw_aliases.clone();
            let mut arm_function_aliases = function_aliases.clone();
            let mut arm_pending_reallocs = pending_reallocs.clone();
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
                } else {
                    arm_aliases.clear(bind_local);
                    arm_pending_reallocs.clear_result(bind_local);
                }
            }
            self.check_ops(
                &mut arm_cells,
                &mut arm_aliases,
                &mut arm_function_aliases,
                &mut arm_pending_reallocs,
                &arm.ops,
            );
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
                arm_function_aliases.copy_alias(&arm.value, output);
                arm_pending_reallocs.copy_result(&arm.value, output);
            }
            arm_paths.push(arm_cells);
            alias_paths.push(arm_aliases);
            function_alias_paths.push(arm_function_aliases);
            pending_realloc_paths.push(arm_pending_reallocs);
        }

        if !arm_paths.is_empty() {
            *cells = CellTable::merge_paths(&arm_paths);
            *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
            *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
        }
        if scrutinee_available && arms_available {
            cells.set_state(output, CellState::Initialized(output.ty));
        } else {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
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
                cells.clear_raw_cells_under(&pending.source);
                cells.release_owned_raw_storage_under(&pending.source);
                cells.mark_initialized(&pending.result);
                if source_owned {
                    cells.mark_owned_raw_storage_root(&pending.result);
                }
                cells.extend_entries(relocated);
                raw_aliases.clear(&pending.source);
                raw_aliases.mark(&pending.result);
            }
            RawReallocConditionOutcome::Failure => {
                raw_aliases.clear(&pending.result);
            }
        }
    }
}

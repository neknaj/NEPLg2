extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::place_with_suffix;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_raw_cell_initialization_summary(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: Span,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self
            .raw_init_summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
            .cloned()
        else {
            return;
        };
        self.apply_raw_cell_initialization_function_summary(
            cells,
            raw_aliases,
            variant_initializations,
            output,
            args,
            &summary,
            span,
        );
    }

    pub(super) fn apply_indirect_call_raw_cell_initialization_summary(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        function_aliases: &FunctionAliasTable,
        callee: &Place,
        args: &[Place],
        span: Span,
    ) {
        let summaries = function_aliases
            .functions(callee)
            .into_iter()
            .filter_map(|function| {
                self.raw_init_summaries
                    .iter()
                    .find(|summary| summary.function == function.as_str())
                    .cloned()
            })
            .collect::<Vec<_>>();
        for summary in summaries {
            self.apply_raw_cell_initialization_function_summary(
                cells,
                raw_aliases,
                variant_initializations,
                output,
                args,
                &summary,
                span,
            );
        }
    }

    fn apply_raw_cell_initialization_function_summary(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        args: &[Place],
        summary: &RawCellInitializationFunctionSummary,
        span: Span,
    ) {
        for destruction in &summary.param_destructions {
            let Some(arg) = args.get(destruction.param_index) else {
                continue;
            };
            let place = place_with_suffix(arg, &destruction.suffix, destruction.ty);
            let place = raw_aliases.canonicalize(&place);
            self.ensure_no_live_non_copy_raw_cells(cells, &place, destruction.operation, span);
        }

        variant_initializations.record_call(raw_aliases, output, args, summary);

        if !summary.return_cells.is_empty() {
            mark_known_raw_address(raw_aliases, output);
        }
        for cell in &summary.return_cells {
            let place = place_with_suffix(output, &cell.suffix, cell.ty);
            cells.mark_initialized(&place);
            if cell.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }

        for cell in &summary.param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let place = place_with_suffix(&arg, &cell.suffix, cell.ty);
            cells.mark_initialized(&place);
            if cell.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }
    }
}

fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}

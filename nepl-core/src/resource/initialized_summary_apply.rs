use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::projected_place_with_concrete_type;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_raw_cell_initialization_summary(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: crate::span::Span,
    ) -> bool {
        let ResourceCallTarget::User { name, .. } = target else {
            return true;
        };
        let Some(summary) = self
            .raw_init_summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
            return true;
        };
        self.apply_raw_cell_initialization_function_summary(
            cells,
            raw_aliases,
            variant_initializations,
            output,
            args,
            summary,
            span,
        )
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
        span: crate::span::Span,
    ) -> bool {
        let mut ok = true;
        for function in function_aliases.functions(callee) {
            let Some(summary) = self
                .raw_init_summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            else {
                continue;
            };
            ok &= self.apply_raw_cell_initialization_function_summary(
                cells,
                raw_aliases,
                variant_initializations,
                output,
                args,
                summary,
                span,
            );
        }
        ok
    }

    fn apply_raw_cell_initialization_function_summary(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        args: &[Place],
        summary: &RawCellInitializationFunctionSummary,
        span: crate::span::Span,
    ) -> bool {
        let release_requirements_ok =
            self.apply_raw_cell_release_requirements(cells, raw_aliases, args, summary, span);

        variant_initializations.record_call(self.types, raw_aliases, output, args, summary);

        if !summary.return_cells.is_empty() {
            mark_known_raw_address(raw_aliases, output);
        }
        for cell in &summary.return_cells {
            let place =
                projected_place_with_concrete_type(self.types, output, &cell.suffix, cell.ty);
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
            let place = projected_place_with_concrete_type(self.types, &arg, &cell.suffix, cell.ty);
            cells.mark_initialized(&place);
            if cell.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }
        release_requirements_ok
    }
}

fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_apply_param::apply_param_initialization_summary;
use super::initialized_summary_apply_return::apply_return_initialization_summary;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceCallTarget};

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

        apply_return_initialization_summary(self.types, cells, raw_aliases, output, summary);

        apply_param_initialization_summary(self.types, cells, raw_aliases, args, summary);

        release_requirements_ok
    }
}

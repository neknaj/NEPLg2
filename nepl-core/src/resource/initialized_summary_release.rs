use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellReleaseRequirementKind,
};
use super::model::Place;
use super::owner_extent_summary::instantiate_summary_type;
use super::place_utils::projected_place_with_concrete_type;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_raw_cell_release_requirements(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &RawCellInitializationFunctionSummary,
        span: crate::span::Span,
    ) -> bool {
        let mut ok = true;
        for requirement in &summary.param_release_requirements {
            let Some(arg) = args.get(requirement.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let address = projected_place_with_concrete_type(
                self.types,
                &arg,
                &requirement.suffix,
                instantiate_summary_type(&summary.type_params, type_args, requirement.ty),
            );
            let address = raw_aliases.canonicalize(&address);
            ok &= self.ensure_no_live_non_copy_raw_cells(
                cells,
                &address,
                release_requirement_operation(requirement.kind),
                span,
            );
        }
        ok
    }
}

fn release_requirement_operation(kind: RawCellReleaseRequirementKind) -> ResourceCheckOperation {
    match kind {
        RawCellReleaseRequirementKind::Store => ResourceCheckOperation::RawMemoryStoreCell,
        RawCellReleaseRequirementKind::Dealloc => ResourceCheckOperation::RawMemoryDeallocCell,
        RawCellReleaseRequirementKind::Realloc => ResourceCheckOperation::RawMemoryReallocCell,
        RawCellReleaseRequirementKind::Fill => ResourceCheckOperation::RawMemoryFillCell,
        RawCellReleaseRequirementKind::BulkDestination => {
            ResourceCheckOperation::RawMemoryBulkDestinationCell
        }
        RawCellReleaseRequirementKind::BulkSource => {
            ResourceCheckOperation::RawMemoryBulkSourceCell
        }
    }
}

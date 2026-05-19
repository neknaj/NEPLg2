use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_state::OwnerTable;
use super::owner_summary_record::parameter_return_extent_for_source;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerParameterReturnExtent, OwnerProjectionSource,
    OwnerReturnSummary,
};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn summary_return_extent_requirement_holds(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &OwnerReturnSummary,
        source: &OwnerProjectionSource,
        requirements: &[OwnerConsumedExtentRequirement],
        span: Span,
    ) -> bool {
        let Some(requirement) = requirements
            .iter()
            .find(|requirement| &requirement.owner == source)
        else {
            return true;
        };
        let extent = instantiate_owner_extent_summary(
            self.types,
            &summary.type_params,
            type_args,
            args,
            &requirement.extent,
        );
        if self.ensure_owner_extent_matches_summary(
            owners,
            raw_aliases,
            place,
            &extent,
            requirement.operation,
            span,
        ) {
            true
        } else {
            self.push_unavailable(
                requirement.operation,
                place,
                owners.state(place).unwrap_or(OwnerState::NoFreeObligation),
                span,
            );
            false
        }
    }
}

pub(super) fn apply_returned_owner_extent(
    types: &crate::types::TypeCtx,
    owners: &mut OwnerTable,
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    args: &[Place],
    output: &Place,
    source: &OwnerProjectionSource,
    extents: &[OwnerParameterReturnExtent],
) {
    let Some(extent) = parameter_return_extent_for_source(extents, source) else {
        return;
    };
    let extent =
        instantiate_owner_extent_summary(types, summary_type_params, type_args, args, extent);
    if !matches!(extent, OwnerStorageExtent::Unknown) {
        owners.set_live_extent(output, extent);
    }
}

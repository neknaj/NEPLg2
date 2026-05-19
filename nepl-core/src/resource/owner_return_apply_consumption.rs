use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerConsumedExtentRequirement, OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_summary_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        source: &OwnerProjectionSource,
        requirements: &[OwnerConsumedExtentRequirement],
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &OwnerReturnSummary,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        span: Span,
    ) {
        variant_owner_effects.materialize_return_owner_for_target(
            self,
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            arg,
            span,
        );
        if self.place_is_copy_owner_view(owners, raw_aliases, arg) {
            return;
        }
        let requirement = requirements
            .iter()
            .find(|requirement| &requirement.owner == source);
        if let Some(requirement) = requirement {
            let extent = instantiate_owner_extent_summary(
                self.types,
                &summary.type_params,
                type_args,
                args,
                &requirement.extent,
            );
            self.consume_call_argument_owner_with_extent(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                &extent,
                requirement.operation,
                span,
            );
        } else {
            self.consume_call_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                span,
            );
        }
    }
}

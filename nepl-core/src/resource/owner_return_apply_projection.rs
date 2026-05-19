use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_extent::apply_returned_owner_extent;
use super::owner_return_apply_place::owner_projection_source_place;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerProjectionReturnSummary, OwnerProjectionSource,
    OwnerReturnSummary,
};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_owner_projection_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        owner_summary: &OwnerReturnSummary,
        summary: &OwnerProjectionReturnSummary,
        consumed_extent_requirements: &[OwnerConsumedExtentRequirement],
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        span: Span,
    ) {
        let mut transferred = false;
        if owners.state(output).is_none() {
            owners.set_state(output, OwnerState::NoFreeObligation);
        }
        if self.has_transferable_owner(owners, raw_aliases, output) {
            return;
        }
        for (parameter_index, arg) in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index).map(|arg| (*index, arg)))
        {
            let source = OwnerProjectionSource {
                parameter_index,
                suffix: Vec::new(),
                ty: arg.ty,
            };
            variant_owner_effects.materialize_return_owner_for_target(
                self,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                span,
            );
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.try_copy_parameter_view_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    arg,
                    args,
                    type_args,
                    owner_summary,
                    &source,
                    consumed_extent_requirements,
                    span,
                ) {
                    continue;
                }
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                apply_returned_owner_extent(
                    self.types,
                    owners,
                    &owner_summary.type_params,
                    type_args,
                    args,
                    output,
                    &source,
                    &summary.parameter_return_extents,
                );
                transferred = true;
                break;
            }
        }
        for source in &summary.parameter_sources {
            if transferred {
                break;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            variant_owner_effects.materialize_return_owner_for_target(
                self,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                span,
            );
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.try_copy_parameter_view_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, &source_place) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    &source_place,
                    args,
                    type_args,
                    owner_summary,
                    source,
                    consumed_extent_requirements,
                    span,
                ) {
                    continue;
                }
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    &source_place,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                apply_returned_owner_extent(
                    self.types,
                    owners,
                    &owner_summary.type_params,
                    type_args,
                    args,
                    output,
                    source,
                    &summary.parameter_return_extents,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate_with_extent(
                output,
                instantiate_owner_extent_summary(
                    self.types,
                    &owner_summary.type_params,
                    type_args,
                    args,
                    &summary.returns_fresh_owner_extent,
                ),
            );
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
    }
}

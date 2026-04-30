use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceMatchArm};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::match_bind_payload_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_match_arm_entry(
    path_engine: &mut ResourceOwnerCheckEngine<'_>,
    path_owners: &mut OwnerTable,
    path_raw_aliases: &mut RawCellAddressAliases,
    path_raw_views: &mut RawAddressViewTable,
    path_storage_origins: &mut StorageOriginTable,
    path_function_aliases: &mut FunctionAliasTable,
    path_pending_reallocs: &mut PendingRawReallocs,
    path_variant_owner_effects: &mut PendingVariantOwnerEffects,
    match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
) {
    let Some((scrutinee, arm, span)) = match_arm else {
        return;
    };
    if !path_variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
        return;
    }
    path_variant_owner_effects.apply_match_arm_returns(
        path_engine,
        path_owners,
        path_raw_aliases,
        path_raw_views,
        path_storage_origins,
        scrutinee,
        &arm.pattern,
        span,
    );
    if let Some(bind_local) = &arm.bind_local {
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            if path_engine.initializer_is_non_owning_raw_alias_view(
                path_owners,
                path_raw_aliases,
                path_raw_views,
                true,
                &source,
                bind_local,
            ) {
                path_engine.copy_non_owning_owner_markers(path_owners, &source, bind_local);
                path_raw_aliases.copy_alias_or_seed(&source, bind_local);
                path_storage_origins.copy_origin(&source, bind_local);
            } else {
                path_engine.transfer_owner(
                    path_owners,
                    path_raw_aliases,
                    path_storage_origins,
                    &source,
                    bind_local,
                    ResourceOwnerOperation::MatchValue,
                    span,
                );
            }
            path_function_aliases.copy_alias(&source, bind_local);
            path_raw_views.copy(&source, bind_local);
            path_pending_reallocs.copy_result(&source, bind_local);
            path_variant_owner_effects.copy_result(&source, bind_local);
            path_variant_owner_effects.apply_match_arm_payload_conditions(
                path_raw_aliases,
                scrutinee,
                &arm.pattern,
                Some(bind_local),
            );
        } else {
            path_raw_aliases.clear(bind_local);
            path_raw_views.clear(bind_local);
            path_storage_origins.clear(bind_local);
            path_pending_reallocs.clear_result(bind_local);
            path_variant_owner_effects.clear_result(bind_local);
        }
    } else {
        path_variant_owner_effects.apply_match_arm_payload_conditions(
            path_raw_aliases,
            scrutinee,
            &arm.pattern,
            None,
        );
    }
    path_variant_owner_effects.apply_match_arm(
        path_engine,
        path_owners,
        path_raw_aliases,
        path_raw_views,
        path_storage_origins,
        scrutinee,
        &arm.pattern,
        span,
    );
}

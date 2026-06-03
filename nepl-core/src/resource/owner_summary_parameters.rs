use alloc::vec::Vec;

use crate::resource_primitives::type_is_raw_pointer;
use crate::types::{TypeCtx, TypeKind};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, ResourceFunction};
use super::owner_state::OwnerTable;
use super::owner_summary_i32_leaf::{i32_leaf_places, raw_i32_owner_leaf_places};
use super::owner_summary_leaf::{owner_leaf_places, OwnerLeafPlace};
use super::owner_summary_owner_token_type::type_contains_owner_token;
use super::owner_summary_raw_consumption::{
    function_consumes_raw_owner_from, function_returns_raw_owner_from,
};
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_summary_seed_leaf::{owner_seed_leaf_places, raw_i32_leaf_is_copy_metadata};
use super::place_utils::{reference_target_place, type_can_seed_raw_address_alias};
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionSource, OwnerReturnSummaryIndex};

pub(super) fn seed_owner_summary_parameters(
    types: &TypeCtx,
    function: &ResourceFunction,
    summaries: &OwnerReturnSummaryIndex<'_>,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
) -> (
    Vec<OwnerParameterStorageSource>,
    Vec<OwnerParameterConditionSource>,
) {
    let mut parameter_storage_sources = Vec::new();
    let mut parameter_condition_sources = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        seed_raw_pointer_parameter_alias(types, &param.place, raw_aliases);
        for leaf in i32_leaf_places(types, &param.place) {
            parameter_condition_sources.push(OwnerParameterConditionSource {
                source: OwnerProjectionSource {
                    parameter_index: index,
                    suffix: leaf.suffix,
                    ty: leaf.place.ty,
                },
                place: leaf.place,
            });
        }
        for leaf in owner_seed_leaf_places(types, function, summaries, index, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
            if let Some(OwnerState::Live { storage, .. }) = owners.state(&leaf.place) {
                parameter_storage_sources.push(OwnerParameterStorageSource {
                    storage,
                    source: OwnerProjectionSource {
                        parameter_index: index,
                        suffix: leaf.suffix,
                        ty: leaf.place.ty,
                    },
                    place: leaf.place,
                });
            }
        }
    }
    (parameter_storage_sources, parameter_condition_sources)
}

pub(super) fn seed_owner_check_parameters(
    types: &TypeCtx,
    function: &ResourceFunction,
    summaries: &OwnerReturnSummaryIndex<'_>,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
) {
    for param in &function.params {
        seed_raw_pointer_parameter_alias(types, &param.place, raw_aliases);
        for leaf in owner_check_seed_leaf_places(types, function, summaries, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
        }
    }
}

fn owner_check_seed_leaf_places(
    types: &TypeCtx,
    function: &ResourceFunction,
    summaries: &OwnerReturnSummaryIndex<'_>,
    base: &super::model::Place,
) -> Vec<OwnerLeafPlace> {
    let mut leaves = Vec::new();
    let base_is_raw_pointer = type_is_raw_pointer(types, base.ty);
    let can_seed_structural_owner =
        base_is_raw_pointer || type_contains_owner_token(types, base.ty) || !types.is_copy(base.ty);
    if !base_is_raw_pointer && can_seed_structural_owner {
        for leaf in owner_leaf_places(types, base) {
            if !leaf_is_unbound_type_variable(types, leaf.place.ty) {
                leaves.push(leaf);
            }
        }
    }
    for leaf in raw_i32_owner_leaf_places(types, base) {
        if raw_i32_leaf_is_copy_metadata(types, base, &leaf) {
            continue;
        }
        let should_seed = !base_is_raw_pointer
            && !leaf.suffix.is_empty()
            && (function_consumes_raw_owner_from(function, &leaf.place, summaries)
                || function_returns_raw_owner_from(function, &leaf.place, summaries));
        if should_seed
            && !leaves
                .iter()
                .any(|existing| existing.place == leaf.place && existing.suffix == leaf.suffix)
        {
            leaves.push(leaf);
        }
    }
    leaves
}

fn leaf_is_unbound_type_variable(types: &TypeCtx, ty: crate::types::TypeId) -> bool {
    match types.get_ref(types.resolve_id(ty)) {
        TypeKind::Var(var) => var.binding.is_none(),
        _ => false,
    }
}

fn seed_raw_pointer_parameter_alias(
    types: &TypeCtx,
    place: &super::model::Place,
    raw_aliases: &mut RawCellAddressAliases,
) {
    if type_is_raw_pointer(types, place.ty) {
        for leaf in raw_i32_owner_leaf_places(types, place) {
            raw_aliases.mark(&leaf.place);
        }
    }
    if let Some(target) = raw_address_reference_target_place(types, place) {
        for leaf in raw_i32_owner_leaf_places(types, &target) {
            raw_aliases.mark(&leaf.place);
        }
    }
}

fn raw_address_reference_target_place(
    types: &TypeCtx,
    place: &super::model::Place,
) -> Option<super::model::Place> {
    let resolved = types.resolve_id(place.ty);
    let TypeKind::Reference(target, _) = types.get_ref(resolved) else {
        return None;
    };
    if !type_can_seed_raw_address_alias(types, *target) {
        return None;
    }
    Some(reference_target_place(place, *target))
}

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceProjection};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_return_apply_source::owner_projection_source_place_for_arg;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{owner_source_for_storage, OwnerParameterStorageSource};
use super::summary::{
    OwnerProjectionSource, OwnerValueCondition, OwnerVariantCondition, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};
use super::variant_name::normalize_variant_name;

pub(super) fn owner_projection_sources_for_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Vec<OwnerProjectionSource> {
    let resolved = resolve_owner_alias_place(owners, raw_aliases, place);
    let mut out = Vec::new();
    if let Some(source) =
        owner_projection_source_for_owner_state(owners.state(&resolved), parameter_storage_sources)
    {
        push_unique_projection_source(&mut out, source);
    }
    for entry in owners.live_entries_under(&resolved) {
        if let Some(source) =
            owner_projection_source_for_owner_state(Some(entry.state), parameter_storage_sources)
        {
            push_unique_projection_source(&mut out, source);
        }
    }
    out
}

fn owner_projection_source_for_owner_state(
    state: Option<OwnerState>,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerProjectionSource> {
    let storage = match state {
        Some(OwnerState::Live { storage, .. }) => storage,
        Some(OwnerState::MaybeFreed {
            storage: Some(storage),
        }) => storage,
        Some(
            OwnerState::NoFreeObligation
            | OwnerState::Reserved { .. }
            | OwnerState::Moved
            | OwnerState::Freed,
        )
        | Some(OwnerState::MaybeFreed { storage: None })
        | None => return None,
    };
    owner_source_for_storage(storage, parameter_storage_sources).cloned()
}

pub(super) fn push_unique_variant_consumed_source(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    variant: String,
    source: OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        let entry = OwnerVariantParameterIndex {
            variant,
            parameter_index: source.parameter_index,
        };
        if !index_out.iter().any(|existing| existing == &entry) {
            index_out.push(entry);
        }
    } else {
        let entry = OwnerVariantProjectionSource { variant, source };
        if !source_out.iter().any(|existing| existing == &entry) {
            source_out.push(entry);
        }
    }
}

pub(super) fn push_unique_variant_projection_return(
    out: &mut Vec<OwnerVariantProjectionReturn>,
    entry: OwnerVariantProjectionReturn,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

pub(super) fn push_unique_owner_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    entry: OwnerVariantCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_projection_source(
    out: &mut Vec<OwnerProjectionSource>,
    source: OwnerProjectionSource,
) {
    if !out.iter().any(|existing| existing == &source) {
        out.push(source);
    }
}

pub(super) fn push_unique_source(
    out: &mut Vec<(Place, Vec<PlaceProjection>, TypeId)>,
    arg: Place,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !source_list_contains(out, &arg, &suffix, ty) {
        out.push((arg, suffix, ty));
    }
}

pub(super) fn source_list_contains(
    sources: &[(Place, Vec<PlaceProjection>, TypeId)],
    arg: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> bool {
    sources
        .iter()
        .any(|(existing_arg, existing_suffix, existing_ty)| {
            existing_arg == arg && existing_suffix == suffix && *existing_ty == ty
        })
}

pub(super) fn owner_value_condition_truth(
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    condition: &OwnerValueCondition,
) -> Option<bool> {
    match condition {
        OwnerValueCondition::Always => Some(true),
        OwnerValueCondition::Param { source, condition } => {
            let arg = args.get(source.parameter_index)?;
            let place = owner_projection_source_place_for_arg(arg, source);
            let place = raw_aliases.canonicalize(&place);
            raw_aliases.i32_condition_truth(&place, *condition)
        }
        OwnerValueCondition::Any(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(false)
            }
        }
        OwnerValueCondition::All(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => {}
                    Some(false) => return Some(false),
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(true)
            }
        }
    }
}

pub(super) fn payload_bind_suffix<'a>(
    suffix: &'a [PlaceProjection],
    variant: &str,
) -> &'a [PlaceProjection] {
    let Some(PlaceProjection::EnumPayload {
        variant: suffix_variant,
    }) = suffix.first()
    else {
        return suffix;
    };
    if normalize_variant_name(suffix_variant) == variant {
        &suffix[1..]
    } else {
        suffix
    }
}

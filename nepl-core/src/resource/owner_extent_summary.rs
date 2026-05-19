use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::compiler_memory_place::region_token_size_field_for_raw_owner;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place};
use super::owner_return_apply_place::owner_projection_source_place;
use super::owner_summary_record::OwnerParameterConditionSource;
use super::owner_summary_variant_conditions::extend_owner_projection_source;
use super::place_utils::place_suffix_after_prefix;
use super::summary::OwnerExtentSummary;
use super::type_var::type_contains_unbound_var;

pub(super) fn summarize_owner_storage_extent_for_owner(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    owner: &Place,
    extent: &OwnerStorageExtent,
) -> OwnerExtentSummary {
    match extent {
        OwnerStorageExtent::Unknown => OwnerExtentSummary::Unknown,
        OwnerStorageExtent::RegionTokenSize => region_token_size_field_for_raw_owner(owner)
            .map(|size| summarize_payload_bytes(raw_aliases, parameter_condition_sources, &size))
            .filter(|summary| !matches!(summary, OwnerExtentSummary::Unknown))
            .unwrap_or(OwnerExtentSummary::RegionTokenSize),
        OwnerStorageExtent::PayloadBytes { bytes } => {
            summarize_payload_bytes(raw_aliases, parameter_condition_sources, bytes)
        }
        OwnerStorageExtent::PayloadBytesScaled { source, scale } => {
            summarize_scaled_payload_bytes(raw_aliases, parameter_condition_sources, source, *scale)
        }
    }
}

fn summarize_payload_bytes(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    bytes: &Place,
) -> OwnerExtentSummary {
    summarize_payload_bytes_inner(raw_aliases, parameter_condition_sources, bytes, 0)
}

fn summarize_payload_bytes_inner(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    bytes: &Place,
    depth: usize,
) -> OwnerExtentSummary {
    if let Some(value) = raw_aliases.i32_value(bytes) {
        return OwnerExtentSummary::PayloadBytesI32Constant {
            value,
            ty: bytes.ty,
        };
    }
    if depth < 4 {
        if let Some((source, element_ty)) = raw_aliases.i32_type_size_scaled_source(bytes) {
            let summary = summarize_payload_bytes_inner(
                raw_aliases,
                parameter_condition_sources,
                &source,
                depth + 1,
            );
            if let OwnerExtentSummary::PayloadBytesParameter(source) = summary {
                return OwnerExtentSummary::PayloadBytesParameterTypeSize { source, element_ty };
            }
        }
        if let Some((source, 1)) = raw_aliases.i32_scaled_source(bytes) {
            let summary = summarize_payload_bytes_inner(
                raw_aliases,
                parameter_condition_sources,
                &source,
                depth + 1,
            );
            if !matches!(summary, OwnerExtentSummary::Unknown) {
                return summary;
            }
        }
    }
    for place_alias in raw_aliases.scalar_aliases_for_value(bytes) {
        for source in parameter_condition_sources {
            for param_alias in raw_aliases.scalar_aliases_for_value(&source.place) {
                let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias) else {
                    continue;
                };
                return OwnerExtentSummary::PayloadBytesParameter(extend_owner_projection_source(
                    &source.source,
                    suffix,
                    place_alias.ty,
                ));
            }
        }
    }
    OwnerExtentSummary::Unknown
}

fn summarize_scaled_payload_bytes(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    source: &Place,
    scale: usize,
) -> OwnerExtentSummary {
    let summary = summarize_payload_bytes(raw_aliases, parameter_condition_sources, source);
    if scale == 1 {
        return summary;
    }
    match summary {
        OwnerExtentSummary::PayloadBytesI32Constant { value, ty } => {
            let Some(scale) = i32::try_from(scale).ok() else {
                return OwnerExtentSummary::Unknown;
            };
            value
                .checked_mul(scale)
                .map(|value| OwnerExtentSummary::PayloadBytesI32Constant { value, ty })
                .unwrap_or(OwnerExtentSummary::Unknown)
        }
        OwnerExtentSummary::PayloadBytesParameter(source) => {
            OwnerExtentSummary::PayloadBytesParameterScaled { source, scale }
        }
        OwnerExtentSummary::PayloadBytesParameterScaled { .. }
        | OwnerExtentSummary::PayloadBytesParameterTypeSize { .. }
        | OwnerExtentSummary::RegionTokenSize
        | OwnerExtentSummary::Unknown => OwnerExtentSummary::Unknown,
    }
}

pub(super) fn instantiate_owner_extent_summary(
    types: &TypeCtx,
    summary_type_params: &[TypeId],
    type_args: &[TypeId],
    args: &[Place],
    summary: &OwnerExtentSummary,
) -> OwnerStorageExtent {
    match summary {
        OwnerExtentSummary::Unknown => OwnerStorageExtent::Unknown,
        OwnerExtentSummary::RegionTokenSize => OwnerStorageExtent::RegionTokenSize,
        OwnerExtentSummary::PayloadBytesParameter(source) => {
            owner_projection_source_place(args, source)
                .map(|place| OwnerStorageExtent::payload_bytes(&place))
                .unwrap_or(OwnerStorageExtent::Unknown)
        }
        OwnerExtentSummary::PayloadBytesParameterScaled { source, scale } => {
            owner_projection_source_place(args, source)
                .map(|place| OwnerStorageExtent::payload_bytes_scaled(&place, *scale))
                .unwrap_or(OwnerStorageExtent::Unknown)
        }
        OwnerExtentSummary::PayloadBytesParameterTypeSize { source, element_ty } => {
            let Some(place) = owner_projection_source_place(args, source) else {
                return OwnerStorageExtent::Unknown;
            };
            let Some(scale) =
                instantiated_type_storage_size(types, summary_type_params, type_args, *element_ty)
            else {
                return OwnerStorageExtent::Unknown;
            };
            OwnerStorageExtent::payload_bytes_scaled(&place, scale)
        }
        OwnerExtentSummary::PayloadBytesI32Constant { value, ty } => {
            OwnerStorageExtent::payload_bytes(&Place::i32_constant(*value, *ty))
        }
    }
}

fn instantiated_type_storage_size(
    types: &TypeCtx,
    summary_type_params: &[TypeId],
    type_args: &[TypeId],
    ty: TypeId,
) -> Option<usize> {
    let ty = instantiate_summary_type(summary_type_params, type_args, ty);
    if type_contains_unbound_var(types, ty) {
        return None;
    }
    Some(storage_size_bytes(types, ty)).filter(|scale| *scale > 0)
}

pub(super) fn instantiate_summary_type(
    summary_type_params: &[TypeId],
    type_args: &[TypeId],
    ty: TypeId,
) -> TypeId {
    summary_type_params
        .iter()
        .position(|type_param| *type_param == ty)
        .and_then(|index| type_args.get(index).copied())
        .unwrap_or(ty)
}

pub(super) fn merge_owner_extent_summaries(
    left: OwnerExtentSummary,
    right: OwnerExtentSummary,
) -> OwnerExtentSummary {
    if left == right {
        left
    } else {
        OwnerExtentSummary::Unknown
    }
}

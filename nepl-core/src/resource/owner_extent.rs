use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place, ResourceI32RelationOp};
pub(super) use super::owner_extent_summary::{
    instantiate_owner_extent_summary, instantiate_summary_type, merge_owner_extent_summaries,
    summarize_owner_storage_extent_for_owner,
};
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_summary_variant_conditions::extend_owner_projection_source;
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::report::ResourceOwnerOperation;
use super::summary::{OwnerConsumedExtentRequirement, OwnerExtentSummary, OwnerProjectionSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingOwnerExtentRequirement {
    pub(super) owner: Place,
    pub(super) expected: OwnerStorageExtent,
    pub(super) operation: ResourceOwnerOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OwnerExtentProof {
    Proven,
    Unknown,
    Mismatch,
}

pub(super) fn prove_owner_extent_matches_argument(
    raw_aliases: &RawCellAddressAliases,
    extent: &OwnerStorageExtent,
    actual: &Place,
) -> OwnerExtentProof {
    match extent {
        OwnerStorageExtent::Unknown => OwnerExtentProof::Unknown,
        OwnerStorageExtent::PayloadBytes { bytes } => {
            prove_scalar_places_equal(raw_aliases, bytes, actual)
        }
        OwnerStorageExtent::PayloadBytesScaled { source, scale } => {
            prove_scaled_place_equals(raw_aliases, source, *scale, actual)
        }
        OwnerStorageExtent::RegionTokenSize => OwnerExtentProof::Unknown,
    }
}

pub(super) fn prove_owner_extent_matches_storage(
    raw_aliases: &RawCellAddressAliases,
    extent: &OwnerStorageExtent,
    expected: &OwnerStorageExtent,
) -> OwnerExtentProof {
    match expected {
        OwnerStorageExtent::Unknown => OwnerExtentProof::Proven,
        OwnerStorageExtent::RegionTokenSize => OwnerExtentProof::Unknown,
        OwnerStorageExtent::PayloadBytes { bytes } => {
            prove_owner_extent_matches_argument(raw_aliases, extent, bytes)
        }
        OwnerStorageExtent::PayloadBytesScaled { source, scale } => match extent {
            OwnerStorageExtent::PayloadBytes { bytes } => {
                prove_scaled_place_equals(raw_aliases, source, *scale, bytes)
            }
            OwnerStorageExtent::PayloadBytesScaled {
                source: extent_source,
                scale: extent_scale,
            } => {
                if extent_scale == scale
                    && prove_scalar_places_equal(raw_aliases, extent_source, source)
                        == OwnerExtentProof::Proven
                {
                    OwnerExtentProof::Proven
                } else {
                    OwnerExtentProof::Mismatch
                }
            }
            OwnerStorageExtent::Unknown | OwnerStorageExtent::RegionTokenSize => {
                OwnerExtentProof::Unknown
            }
        },
    }
}

pub(super) fn summarize_consumed_extent_requirements(
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
    requirements: &[PendingOwnerExtentRequirement],
    _consumed_indices: &[usize],
    _consumed_sources: &[OwnerProjectionSource],
) -> Vec<OwnerConsumedExtentRequirement> {
    let mut out = Vec::new();
    for requirement in requirements {
        let extent = summarize_owner_storage_extent_for_owner(
            raw_aliases,
            parameter_condition_sources,
            &requirement.owner,
            &requirement.expected,
        );
        if matches!(extent, OwnerExtentSummary::Unknown) {
            continue;
        }
        for owner in
            owner_requirement_sources(raw_aliases, parameter_storage_sources, &requirement.owner)
        {
            push_or_merge_consumed_extent_requirement(
                &mut out,
                OwnerConsumedExtentRequirement {
                    owner,
                    extent: extent.clone(),
                    operation: requirement.operation,
                },
            );
        }
    }
    out
}

fn prove_scalar_places_equal(
    raw_aliases: &RawCellAddressAliases,
    expected: &Place,
    actual: &Place,
) -> OwnerExtentProof {
    let expected = raw_aliases.canonicalize_scalar(expected);
    let actual = raw_aliases.canonicalize_scalar(actual);
    if expected == actual {
        return OwnerExtentProof::Proven;
    }
    match (
        raw_aliases.i32_value(&expected),
        raw_aliases.i32_value(&actual),
    ) {
        (Some(left), Some(right)) if left == right => return OwnerExtentProof::Proven,
        (Some(_), Some(_)) => return OwnerExtentProof::Mismatch,
        _ => {}
    }
    if raw_aliases.i32_relation_truth(&expected, ResourceI32RelationOp::Eq, &actual) == Some(true) {
        return OwnerExtentProof::Proven;
    }
    if raw_aliases.i32_relation_truth(&expected, ResourceI32RelationOp::Ne, &actual) == Some(true) {
        return OwnerExtentProof::Mismatch;
    }
    match (
        raw_aliases.i32_scaled_source(&expected),
        raw_aliases.i32_scaled_source(&actual),
    ) {
        (Some(left), Some(right)) if left == right => OwnerExtentProof::Proven,
        (Some(_), Some(_)) => OwnerExtentProof::Mismatch,
        _ => OwnerExtentProof::Unknown,
    }
}

fn prove_scaled_place_equals(
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    scale: usize,
    actual: &Place,
) -> OwnerExtentProof {
    if scale == 1 {
        return prove_scalar_places_equal(raw_aliases, source, actual);
    }
    if let Some((actual_source, actual_scale)) = raw_aliases.i32_scaled_source(actual) {
        if actual_scale == scale && raw_aliases.canonicalize_scalar(source) == actual_source {
            return OwnerExtentProof::Proven;
        }
    }
    match (raw_aliases.i32_value(source), raw_aliases.i32_value(actual)) {
        (Some(source), Some(actual)) => {
            let Some(scale) = i32::try_from(scale).ok() else {
                return OwnerExtentProof::Mismatch;
            };
            match source.checked_mul(scale) {
                Some(expected) if expected == actual => OwnerExtentProof::Proven,
                Some(_) | None => OwnerExtentProof::Mismatch,
            }
        }
        _ => OwnerExtentProof::Unknown,
    }
}

fn owner_requirement_sources(
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    owner: &Place,
) -> Vec<OwnerProjectionSource> {
    let mut owner_aliases = raw_aliases.aliases_for(owner);
    push_unique_place(&mut owner_aliases, owner);
    let mut out = Vec::new();
    for owner_alias in owner_aliases {
        for source in parameter_storage_sources {
            let mut source_aliases = raw_aliases.aliases_for(&source.place);
            push_unique_place(&mut source_aliases, &source.place);
            for source_alias in source_aliases {
                let Some(suffix) = place_suffix_after_prefix(&owner_alias, &source_alias) else {
                    continue;
                };
                push_unique_owner_projection_source(
                    &mut out,
                    extend_owner_projection_source(&source.source, suffix, owner_alias.ty),
                );
            }
        }
    }
    out
}

fn push_or_merge_consumed_extent_requirement(
    out: &mut Vec<OwnerConsumedExtentRequirement>,
    requirement: OwnerConsumedExtentRequirement,
) {
    if let Some(existing) = out.iter_mut().find(|existing| {
        existing.owner == requirement.owner && existing.operation == requirement.operation
    }) {
        existing.extent = merge_owner_extent_summaries(existing.extent.clone(), requirement.extent);
        return;
    }
    out.push(requirement);
}

fn push_unique_owner_projection_source(
    out: &mut Vec<OwnerProjectionSource>,
    source: OwnerProjectionSource,
) {
    if !out.iter().any(|existing| existing == &source) {
        out.push(source);
    }
}

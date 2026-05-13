use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place, ResourceI32RelationOp};
use super::owner_return_apply_source::owner_projection_source_place;
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
        let extent = summarize_owner_storage_extent(
            raw_aliases,
            parameter_condition_sources,
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

pub(super) fn summarize_owner_storage_extent(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    extent: &OwnerStorageExtent,
) -> OwnerExtentSummary {
    match extent {
        OwnerStorageExtent::Unknown => OwnerExtentSummary::Unknown,
        OwnerStorageExtent::PayloadBytes { bytes } => {
            if let Some(value) = raw_aliases.i32_value(bytes) {
                return OwnerExtentSummary::PayloadBytesI32Constant {
                    value,
                    ty: bytes.ty,
                };
            }
            for place_alias in raw_aliases.scalar_aliases_for_value(bytes) {
                for source in parameter_condition_sources {
                    for param_alias in raw_aliases.scalar_aliases_for_value(&source.place) {
                        let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias)
                        else {
                            continue;
                        };
                        return OwnerExtentSummary::PayloadBytesParameter(
                            extend_owner_projection_source(&source.source, suffix, place_alias.ty),
                        );
                    }
                }
            }
            OwnerExtentSummary::Unknown
        }
    }
}

pub(super) fn instantiate_owner_extent_summary(
    args: &[Place],
    summary: &OwnerExtentSummary,
) -> OwnerStorageExtent {
    match summary {
        OwnerExtentSummary::Unknown => OwnerStorageExtent::Unknown,
        OwnerExtentSummary::PayloadBytesParameter(source) => {
            owner_projection_source_place(args, source)
                .map(|place| OwnerStorageExtent::payload_bytes(&place))
                .unwrap_or(OwnerStorageExtent::Unknown)
        }
        OwnerExtentSummary::PayloadBytesI32Constant { value, ty } => {
            OwnerStorageExtent::payload_bytes(&Place::i32_constant(*value, *ty))
        }
    }
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
        _ => OwnerExtentProof::Mismatch,
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

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceProjection};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{owner_source_for_storage, OwnerParameterStorageSource};
use super::summary::{
    OwnerProjectionSource, OwnerVariantCondition, OwnerVariantParameterIndex,
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
    if out.iter().any(|existing| {
        existing.variant == entry.variant
            && existing.suffix == entry.suffix
            && existing.ty == entry.ty
            && existing.owner == entry.owner
    }) {
        return;
    }
    let Some(existing) = out.iter_mut().find(|existing| {
        existing.variant == entry.variant
            && existing.suffix == entry.suffix
            && existing.ty == entry.ty
            && !return_conditions_are_mutually_exclusive(
                existing.source_condition.as_ref(),
                entry.source_condition.as_ref(),
            )
    }) else {
        out.push(entry);
        return;
    };
    if existing.source_condition != entry.source_condition {
        existing.source_condition = None;
    }
    let entry_owner_extent = projection_return_owner_extent(&entry.owner);
    match (&mut existing.owner, entry.owner) {
        (
            super::summary::OwnerProjectionReturnOwner::Parameter {
                source: existing_source,
                returned_extent,
            },
            super::summary::OwnerProjectionReturnOwner::Parameter {
                source,
                returned_extent: next_extent,
            },
        ) if existing_source == &source => {
            *returned_extent = super::owner_extent::merge_owner_extent_summaries(
                returned_extent.clone(),
                next_extent,
            );
        }
        (
            super::summary::OwnerProjectionReturnOwner::Fresh { extent },
            super::summary::OwnerProjectionReturnOwner::Fresh {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (
            super::summary::OwnerProjectionReturnOwner::UnknownSource { extent },
            super::summary::OwnerProjectionReturnOwner::UnknownSource {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (super::summary::OwnerProjectionReturnOwner::Maybe, _) => {}
        (_, super::summary::OwnerProjectionReturnOwner::Maybe) => {
            existing.owner = super::summary::OwnerProjectionReturnOwner::Maybe;
        }
        _ => {
            existing.owner = super::summary::OwnerProjectionReturnOwner::UnknownSource {
                extent: super::owner_extent::merge_owner_extent_summaries(
                    projection_return_owner_extent(&existing.owner),
                    entry_owner_extent,
                ),
            };
        }
    }
}

pub(super) fn return_conditions_are_mutually_exclusive(
    left: Option<&OwnerProjectionSource>,
    right: Option<&OwnerProjectionSource>,
) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };
    mutually_exclusive_parameter_sources(
        &super::summary::OwnerProjectionReturnOwner::Parameter {
            source: left.clone(),
            returned_extent: super::summary::OwnerExtentSummary::Unknown,
        },
        &super::summary::OwnerProjectionReturnOwner::Parameter {
            source: right.clone(),
            returned_extent: super::summary::OwnerExtentSummary::Unknown,
        },
    )
}

pub(super) fn mutually_exclusive_parameter_sources(
    left: &super::summary::OwnerProjectionReturnOwner,
    right: &super::summary::OwnerProjectionReturnOwner,
) -> bool {
    let (
        super::summary::OwnerProjectionReturnOwner::Parameter {
            source: left_source,
            ..
        },
        super::summary::OwnerProjectionReturnOwner::Parameter {
            source: right_source,
            ..
        },
    ) = (left, right)
    else {
        return false;
    };
    if left_source.parameter_index != right_source.parameter_index {
        return false;
    }
    for (left, right) in left_source.suffix.iter().zip(&right_source.suffix) {
        if left == right {
            continue;
        }
        return match (left, right) {
            (
                PlaceProjection::EnumPayload { variant: left },
                PlaceProjection::EnumPayload { variant: right },
            ) => normalize_variant_name(left) != normalize_variant_name(right),
            _ => false,
        };
    }
    false
}

pub(super) fn source_condition_for_projection_source(
    source: &OwnerProjectionSource,
) -> Option<OwnerProjectionSource> {
    let end = source
        .suffix
        .iter()
        .rposition(|projection| matches!(projection, PlaceProjection::EnumPayload { .. }))?;
    Some(OwnerProjectionSource {
        parameter_index: source.parameter_index,
        suffix: source.suffix[..=end].to_vec(),
        ty: source.ty,
    })
}

fn projection_return_owner_extent(
    owner: &super::summary::OwnerProjectionReturnOwner,
) -> super::summary::OwnerExtentSummary {
    match owner {
        super::summary::OwnerProjectionReturnOwner::Parameter {
            returned_extent, ..
        } => returned_extent.clone(),
        super::summary::OwnerProjectionReturnOwner::Fresh { extent }
        | super::summary::OwnerProjectionReturnOwner::UnknownSource { extent } => extent.clone(),
        super::summary::OwnerProjectionReturnOwner::Maybe => {
            super::summary::OwnerExtentSummary::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeId;
    use alloc::vec;

    use super::super::summary::{
        OwnerExtentSummary, OwnerProjectionReturnOwner, OwnerProjectionSource,
        OwnerVariantProjectionReturn,
    };

    fn parameter_owner(
        parameter_index: usize,
        suffix: Vec<PlaceProjection>,
    ) -> OwnerProjectionReturnOwner {
        OwnerProjectionReturnOwner::Parameter {
            source: OwnerProjectionSource {
                parameter_index,
                suffix,
                ty: TypeId(1),
            },
            returned_extent: OwnerExtentSummary::Unknown,
        }
    }

    fn variant(name: &str) -> PlaceProjection {
        PlaceProjection::EnumPayload {
            variant: String::from(name),
        }
    }

    fn field(index: usize) -> PlaceProjection {
        PlaceProjection::Field {
            index,
            offset_bytes: index * 4,
        }
    }

    #[test]
    fn parameter_sources_are_exclusive_only_at_a_shared_enum_projection() {
        assert!(mutually_exclusive_parameter_sources(
            &parameter_owner(0, vec![field(0), variant("A")]),
            &parameter_owner(0, vec![field(0), variant("B")]),
        ));
        assert!(!mutually_exclusive_parameter_sources(
            &parameter_owner(0, vec![variant("A")]),
            &parameter_owner(1, vec![variant("B")]),
        ));
        assert!(!mutually_exclusive_parameter_sources(
            &parameter_owner(0, vec![field(0), variant("A")]),
            &parameter_owner(0, vec![field(1), variant("B")]),
        ));
        assert!(!mutually_exclusive_parameter_sources(
            &parameter_owner(0, vec![variant("B"), field(0)]),
            &parameter_owner(0, vec![variant("B"), field(1)]),
        ));
        assert!(!mutually_exclusive_parameter_sources(
            &parameter_owner(0, vec![variant("Result::A")]),
            &parameter_owner(0, vec![variant("A")]),
        ));
    }

    #[test]
    fn same_variant_ambiguity_merges_after_an_exclusive_alternative() {
        let a = parameter_owner(0, vec![variant("A")]);
        let b0 = parameter_owner(0, vec![variant("B"), field(0)]);
        let b1 = parameter_owner(0, vec![variant("B"), field(1)]);
        for owners in [[a.clone(), b0.clone(), b1.clone()], [b0, b1, a]] {
            let mut returns = Vec::new();
            for owner in owners {
                let source_condition = match &owner {
                    OwnerProjectionReturnOwner::Parameter { source, .. } => {
                        source_condition_for_projection_source(source)
                    }
                    _ => None,
                };
                push_unique_variant_projection_return(
                    &mut returns,
                    OwnerVariantProjectionReturn {
                        variant: String::from("Ok"),
                        suffix: vec![variant("Ok"), field(0)],
                        ty: TypeId(1),
                        source_condition,
                        owner,
                    },
                );
            }
            assert_eq!(returns.len(), 2, "{returns:#?}");
            assert_eq!(
                returns
                    .iter()
                    .filter(|entry| matches!(
                        entry.owner,
                        OwnerProjectionReturnOwner::UnknownSource { .. }
                    ))
                    .count(),
                1,
                "{returns:#?}"
            );
        }
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

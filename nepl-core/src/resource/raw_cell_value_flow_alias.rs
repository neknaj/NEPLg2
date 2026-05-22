extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_rank::owner_cell_alias_rank;
use super::model::{Place, PlaceProjection, PlaceRoot, ResourceOffset};
use super::raw_cell_value_flow::{RawCellValueFlowEntry, RawCellValueFlowKind};
use super::type_pattern::type_pattern_matches;

pub(super) fn raw_cell_places_equivalent(left: &Place, right: &Place) -> bool {
    let left = place_without_zero_storage_offsets(left);
    let right = place_without_zero_storage_offsets(right);
    left.root == right.root && left.projections == right.projections
}

pub(super) fn canonical_raw_cell_place_with_aliases(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    raw_cell_place_alias_candidates(place, raw_aliases)
        .into_iter()
        .min_by_key(owner_cell_alias_rank)
        .unwrap_or_else(|| place.clone())
}

pub(super) fn raw_cell_place_alias_candidates(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut candidates = Vec::new();
    push_unique_place(&mut candidates, place);
    let mut index = 0;
    while index < candidates.len() {
        let candidate = candidates[index].clone();
        index += 1;
        push_raw_cell_alias_candidate_closure(&mut candidates, &candidate, raw_aliases);
    }
    candidates
}

fn push_raw_cell_alias_candidate_closure(
    candidates: &mut Vec<Place>,
    candidate: &Place,
    raw_aliases: &RawCellAddressAliases,
) {
    for alias in raw_aliases.raw_address_aliases_for_value(candidate) {
        push_unique_place(candidates, &alias);
    }
    for alias in raw_aliases.prefix_aliases_for(candidate) {
        push_unique_place(candidates, &alias);
    }

    let canonical_owner = raw_aliases.canonicalize_owner_cell_address(candidate);
    push_unique_place(candidates, &canonical_owner);

    let canonical_offset = raw_cell_place_with_canonical_symbolic_offsets(candidate, raw_aliases);
    push_unique_place(candidates, &canonical_offset);

    let canonical_owner_offset =
        raw_cell_place_with_canonical_symbolic_offsets(&canonical_owner, raw_aliases);
    push_unique_place(candidates, &canonical_owner_offset);
}

pub(super) fn raw_cell_place_with_canonical_symbolic_offsets(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    place_with_canonical_symbolic_offsets(&place_without_zero_storage_offsets(place), raw_aliases)
}

pub(super) fn place_with_canonical_symbolic_offsets(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let mut out = place.clone();
    for projection in &mut out.projections {
        match projection {
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }) => {
                if let Some((source, offset)) = canonical_offset_source_place(place, raw_aliases) {
                    *projection = PlaceProjection::StorageOffset(ResourceOffset::Offset {
                        place: Box::new(source),
                        offset,
                    });
                } else if let Some((source, scale)) =
                    canonical_scaled_symbolic_offset_place(place, raw_aliases)
                {
                    *projection = PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                        place: Box::new(source),
                        scale,
                    });
                } else {
                    *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                }
            }
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale }) => {
                if let Some((source, offset)) = canonical_offset_source_place(place, raw_aliases) {
                    *projection = PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                        place: Box::new(source),
                        offset,
                        scale: *scale,
                    });
                } else if let Some((source, source_scale)) =
                    canonical_scaled_symbolic_offset_place(place, raw_aliases)
                {
                    if let Some(combined_scale) = scale.checked_mul(source_scale) {
                        *place = Box::new(source);
                        *scale = combined_scale;
                    } else {
                        *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                    }
                } else {
                    *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                }
            }
            PlaceProjection::StorageOffset(ResourceOffset::Offset { place, offset }) => {
                if let Some((source, source_offset)) =
                    canonical_offset_source_place(place, raw_aliases)
                {
                    if let Some(combined_offset) = source_offset.checked_add(*offset) {
                        if combined_offset == 0 {
                            *projection =
                                PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                                    place: Box::new(source),
                                });
                        } else {
                            *place = Box::new(source);
                            *offset = combined_offset;
                        }
                    }
                } else {
                    *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                }
            }
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place,
                offset,
                scale,
            }) => {
                if let Some((source, source_offset)) =
                    canonical_offset_source_place(place, raw_aliases)
                {
                    if let Some(combined_offset) = source_offset.checked_add(*offset) {
                        if combined_offset == 0 {
                            *projection =
                                PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                                    place: Box::new(source),
                                    scale: *scale,
                                });
                        } else {
                            *place = Box::new(source);
                            *offset = combined_offset;
                        }
                    }
                } else if let Some((source, source_scale)) =
                    canonical_scaled_symbolic_offset_place(place, raw_aliases)
                {
                    if let Some(combined_scale) = scale.checked_mul(source_scale) {
                        *place = Box::new(source);
                        *scale = combined_scale;
                    } else {
                        *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                    }
                } else {
                    *place = Box::new(canonical_symbolic_offset_place(place, raw_aliases));
                }
            }
            PlaceProjection::StorageOffset(ResourceOffset::Known(_) | ResourceOffset::Unknown)
            | PlaceProjection::Field { .. }
            | PlaceProjection::TupleField { .. }
            | PlaceProjection::EnumPayload { .. }
            | PlaceProjection::Deref => {}
        }
    }
    out
}

pub(super) fn value_flow_entry_matches(
    entry: &RawCellValueFlowEntry,
    cell: &Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    raw_cell_places_equivalent(&entry.cell, cell)
        && entry.kind == kind
        && (type_pattern_matches(types, entry.ty, ty) || type_pattern_matches(types, ty, entry.ty))
}

pub(super) fn value_flow_entry_matches_any_cell(
    entry: &RawCellValueFlowEntry,
    raw_aliases: &RawCellAddressAliases,
    cells: &[Place],
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    entry.kind == kind
        && (type_pattern_matches(types, entry.ty, ty) || type_pattern_matches(types, ty, entry.ty))
        && cells
            .iter()
            .any(|cell| raw_cell_places_equivalent_with_aliases(&entry.cell, cell, raw_aliases))
}

pub(super) fn place_without_zero_storage_offsets(place: &Place) -> Place {
    let mut out = place.clone();
    out.projections.retain(|projection| {
        !matches!(
            projection,
            PlaceProjection::StorageOffset(ResourceOffset::Known(0))
        )
    });
    out
}

fn raw_cell_places_equivalent_with_aliases(
    left: &Place,
    right: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let left = raw_cell_place_with_canonical_symbolic_offsets(left, raw_aliases);
    let right = raw_cell_place_with_canonical_symbolic_offsets(right, raw_aliases);
    raw_cell_places_equivalent(&left, &right)
}

fn canonical_symbolic_offset_place(place: &Place, raw_aliases: &RawCellAddressAliases) -> Place {
    raw_aliases
        .scalar_aliases_for(place)
        .into_iter()
        .min_by(|left, right| {
            symbolic_offset_place_rank(left)
                .cmp(&symbolic_offset_place_rank(right))
                .then_with(|| left.cmp(right))
        })
        .unwrap_or_else(|| place.clone())
}

fn canonical_scaled_symbolic_offset_place(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<(Place, usize)> {
    let (source, scale) = raw_aliases.i32_scaled_source(place)?;
    Some((canonical_symbolic_offset_place(&source, raw_aliases), scale))
}

fn canonical_offset_source_place(
    place: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<(Place, i64)> {
    raw_aliases
        .i32_offset_sources(place)
        .into_iter()
        .map(|(source, offset)| {
            (
                canonical_symbolic_offset_place(&source, raw_aliases),
                offset,
            )
        })
        .min_by(|(left_place, left_offset), (right_place, right_offset)| {
            symbolic_offset_place_rank(left_place)
                .cmp(&symbolic_offset_place_rank(right_place))
                .then_with(|| left_place.cmp(right_place))
                .then_with(|| left_offset.cmp(right_offset))
        })
}

fn push_unique_place(out: &mut Vec<Place>, place: &Place) {
    if !out.iter().any(|existing| existing == place) {
        out.push(place.clone());
    }
}

fn symbolic_offset_place_rank(place: &Place) -> (u8, usize) {
    let root_rank = match &place.root {
        PlaceRoot::Local(_) | PlaceRoot::I32Constant(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    };
    (root_rank, place.projections.len())
}

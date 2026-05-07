extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::replace_place_prefix;

pub(super) fn substitute_summary_projection_offsets(
    raw_aliases: &RawCellAddressAliases,
    projections: &[PlaceProjection],
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
) -> Vec<PlaceProjection> {
    projections
        .iter()
        .map(|projection| match projection {
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }) => {
                substitute_symbolic_offset(raw_aliases, place, summary, args, projection)
            }
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale }) => {
                substitute_scaled_symbolic_offset(
                    raw_aliases,
                    place,
                    *scale,
                    summary,
                    args,
                    projection,
                )
            }
            _ => projection.clone(),
        })
        .collect()
}

fn substitute_symbolic_offset(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
    fallback: &PlaceProjection,
) -> PlaceProjection {
    let Some(actual) = substitute_summary_place(place, summary, args) else {
        return fallback.clone();
    };
    if let Some(value) = raw_aliases.i32_value(&actual) {
        return PlaceProjection::StorageOffset(resource_offset_from_i32(value));
    }
    PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
        place: Box::new(actual),
    })
}

fn substitute_scaled_symbolic_offset(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    scale: usize,
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
    fallback: &PlaceProjection,
) -> PlaceProjection {
    let Some(actual) = substitute_summary_place(place, summary, args) else {
        return fallback.clone();
    };
    if let Some(value) = raw_aliases.i32_value(&actual) {
        return PlaceProjection::StorageOffset(resource_offset_from_scaled_i32(value, scale));
    }
    PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
        place: Box::new(actual),
        scale,
    })
}

fn substitute_summary_place(
    place: &Place,
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
) -> Option<Place> {
    for (index, parameter) in summary.parameters.iter().enumerate() {
        let Some(arg) = args.get(index) else {
            continue;
        };
        if let Some(replaced) = replace_place_prefix(place, parameter, arg) {
            return Some(replaced);
        }
    }
    None
}

fn resource_offset_from_i32(value: i32) -> ResourceOffset {
    usize::try_from(value)
        .map(ResourceOffset::Known)
        .unwrap_or(ResourceOffset::Unknown)
}

fn resource_offset_from_scaled_i32(value: i32, scale: usize) -> ResourceOffset {
    let Ok(value) = usize::try_from(value) else {
        return ResourceOffset::Unknown;
    };
    value
        .checked_mul(scale)
        .map(ResourceOffset::Known)
        .unwrap_or(ResourceOffset::Unknown)
}

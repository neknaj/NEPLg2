use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{
    AggregateKind, Place, PlaceProjection, PlaceRoot, ResourceMatchArm, ResourceMatchPattern,
};

pub(super) fn should_track(place: &Place) -> bool {
    !matches!(place.root, PlaceRoot::Unknown)
}

pub(super) fn raw_memory_cell_place(address: &Place, ty: TypeId) -> Place {
    address.clone().with_projection(PlaceProjection::Deref, ty)
}

pub(super) fn raw_memory_unknown_offset_cell_place(address: &Place, ty: TypeId) -> Place {
    let address = address.clone().with_projection(
        PlaceProjection::StorageOffset(super::model::ResourceOffset::Dynamic),
        ty,
    );
    raw_memory_cell_place(&address, ty)
}

pub(super) fn construct_aggregate_field_place(
    output: &Place,
    kind: &AggregateKind,
    index: usize,
    input: &Place,
) -> Place {
    let mut place = output.clone();
    match kind {
        AggregateKind::Struct { field_offsets, .. } => {
            place.projections.push(PlaceProjection::Field {
                index,
                offset_bytes: field_offsets[index],
            });
        }
        AggregateKind::Tuple { field_offsets } => {
            place.projections.push(PlaceProjection::TupleField {
                index,
                offset_bytes: field_offsets[index],
            });
        }
        AggregateKind::Enum { variant, .. } => {
            place.projections.push(PlaceProjection::EnumPayload {
                variant: variant.clone(),
            });
            if index > 0 {
                place.projections.push(PlaceProjection::TupleField {
                    index,
                    offset_bytes: 0,
                });
            }
        }
    }
    place.ty = input.ty;
    place
}

pub(super) fn replace_place_prefix(
    place: &Place,
    prefix: &Place,
    replacement: &Place,
) -> Option<Place> {
    place_suffix_after_prefix(place, prefix)
        .map(|suffix| place_with_suffix(replacement, &suffix, place.ty))
}

pub(super) fn places_overlap(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}

pub(super) fn place_suffix_after_prefix(
    place: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    if place.root != prefix.root || place.projections.len() < prefix.projections.len() {
        return None;
    }
    if place.projections[..prefix.projections.len()] != prefix.projections[..] {
        return None;
    }
    Some(place.projections[prefix.projections.len()..].to_vec())
}

pub(super) fn place_with_suffix(base: &Place, suffix: &[PlaceProjection], ty: TypeId) -> Place {
    let mut out = base.clone();
    out.projections.extend_from_slice(suffix);
    out.ty = ty;
    out
}

pub(super) fn match_bind_payload_place(
    scrutinee: &Place,
    arm: &ResourceMatchArm,
    bind_local: &Place,
) -> Option<Place> {
    let variant = match_arm_variant_payload_name(arm)?;
    Some(scrutinee.clone().with_projection(
        PlaceProjection::EnumPayload {
            variant: String::from(variant),
        },
        bind_local.ty,
    ))
}

pub(super) fn match_arm_variant_payload_name(arm: &ResourceMatchArm) -> Option<&str> {
    let ResourceMatchPattern::Variant(variant) = &arm.pattern else {
        return None;
    };
    Some(variant.rsplit("::").next().unwrap_or(variant))
}

pub(super) fn push_unique_place(places: &mut Vec<Place>, place: &Place) {
    if !places.iter().any(|existing| existing == place) {
        places.push(place.clone());
    }
}

pub(super) fn push_unique_usize(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}

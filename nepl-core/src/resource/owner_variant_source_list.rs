use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{Place, PlaceProjection};
use super::owner_return_apply_place::summary_projection_place;
use super::place_utils::places_overlap;

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

pub(super) fn source_list_overlaps(
    sources: &[(Place, Vec<PlaceProjection>, TypeId)],
    arg: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> bool {
    let source = summary_projection_place(arg, suffix, ty);
    sources
        .iter()
        .any(|(existing_arg, existing_suffix, existing_ty)| {
            let existing = summary_projection_place(existing_arg, existing_suffix, *existing_ty);
            places_overlap(&existing, &source)
        })
}

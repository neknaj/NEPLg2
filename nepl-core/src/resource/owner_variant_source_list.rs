use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{Place, PlaceProjection};

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

use super::model::{Place, PlaceProjection};
use super::place_utils::place_with_checked_suffix;
use crate::types::{TypeCtx, TypeId};

pub(super) fn summary_projection_is_valid(
    types: &TypeCtx,
    base: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> bool {
    place_with_checked_suffix(Some(types), base, suffix, ty)
        .is_some_and(|place| types.same_type(place.ty, ty))
}

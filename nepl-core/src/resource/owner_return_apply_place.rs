use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::place_utils::place_with_suffix;
use super::summary::OwnerProjectionSource;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn place_is_copy_owner_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        self.types.is_copy(place.ty)
            && !self.has_transferable_owner(owners, raw_aliases, place)
            && owner_leaf_places(self.types, place)
                .iter()
                .any(|leaf| leaf.place == *place)
    }
}

pub(super) fn owner_projection_source_place(
    args: &[Place],
    source: &OwnerProjectionSource,
) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(owner_projection_source_place_for_arg(arg, source))
}

pub(super) fn owner_projection_source_place_for_arg(
    arg: &Place,
    source: &OwnerProjectionSource,
) -> Place {
    summary_projection_place(arg, &source.suffix, source.ty)
}

pub(super) fn summary_projection_place(
    base: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> Place {
    let ty = if suffix.is_empty() { base.ty } else { ty };
    place_with_suffix(base, suffix, ty)
}

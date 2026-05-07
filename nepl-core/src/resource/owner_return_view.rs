use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn place_is_non_owning_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        place: &Place,
    ) -> bool {
        (self.types.resolve_id(place.ty) == self.types.i32()
            || raw_views.contains_non_owning(place))
            && !self.has_transferable_owner(owners, raw_aliases, place)
            && !owners.has_tracked_state_under(place)
            && (raw_views.contains_non_owning(place)
                || raw_aliases
                    .aliases_for(place)
                    .iter()
                    .any(|alias| alias != place && place_has_raw_address_projection(alias)))
    }
}

fn place_has_raw_address_projection(place: &Place) -> bool {
    place
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
}

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_address_view_candidate_bases;

impl RawCellAddressAliases {
    pub(super) fn raw_address_view_source_is_known(&self, place: &Place) -> bool {
        self.value_is_known_raw_address(place)
            || raw_address_view_candidate_bases(place)
                .iter()
                .any(|prefix| self.value_is_known_raw_address(prefix))
    }
}

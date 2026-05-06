use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_address_view_candidate_bases;

impl ResourceCheckEngine<'_> {
    pub(super) fn raw_address_view_source_is_known(
        &self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        source: &Place,
    ) -> bool {
        raw_aliases.raw_address_view_source_is_known(source)
            || raw_address_view_candidate_bases(source)
                .iter()
                .map(|base| raw_aliases.canonicalize(base))
                .any(|base| cells.raw_address_has_tracked_storage(&base))
    }
}

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};

impl ResourceCheckEngine<'_> {
    pub(super) fn copy_raw_alias_and_rekey_cells(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.copy_raw_alias_and_rekey_cells_with_mode(
            cells,
            raw_aliases,
            source,
            target,
            false,
            false,
        );
    }

    pub(super) fn copy_raw_alias_and_rekey_cells_preferring_target(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.copy_raw_alias_and_rekey_cells_with_mode(
            cells,
            raw_aliases,
            source,
            target,
            false,
            true,
        );
    }

    pub(super) fn copy_raw_address_alias_and_rekey_cells(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.copy_raw_alias_and_rekey_cells_with_mode(
            cells,
            raw_aliases,
            source,
            target,
            true,
            false,
        );
    }

    fn copy_raw_alias_and_rekey_cells_with_mode(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
        force_raw_address: bool,
        prefer_target: bool,
    ) {
        let source_tracks_raw_address = raw_aliases.contains_exact(source);
        let source_canonical = raw_aliases.canonicalize(source);
        let source_aliases = raw_aliases.aliases_for(source);
        let source_is_known_raw_address =
            force_raw_address || source_tracks_raw_address || source_aliases.len() > 1;
        let source_is_external_raw_storage = source_is_known_raw_address
            && source_aliases
                .iter()
                .any(|alias| cells.external_raw_storage_overlaps(alias));
        let source_is_raw_address_view = place_has_storage_offset(&source_canonical)
            || source_aliases.iter().any(place_has_storage_offset);
        if force_raw_address {
            raw_aliases.copy_explicit_raw_address_alias(source, target);
        } else {
            raw_aliases.copy_alias_if_tracked(source, target);
        }
        if prefer_target {
            raw_aliases.prefer_canonical(target);
        }
        let target_canonical = raw_aliases.canonicalize(target);
        if source_is_external_raw_storage {
            for alias in &source_aliases {
                cells.mark_external_raw_storage_root(alias);
            }
            cells.mark_external_raw_storage_root(target);
            cells.mark_external_raw_storage_root(&target_canonical);
        }
        if source_is_known_raw_address {
            if source_is_raw_address_view {
                return;
            }
            if prefer_target {
                for alias in &source_aliases {
                    cells.rekey_raw_cells(alias, &target_canonical);
                }
            } else {
                cells.rekey_raw_cells(&source_canonical, &target_canonical);
            }
        }
    }
}

fn place_has_storage_offset(place: &Place) -> bool {
    place
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
}

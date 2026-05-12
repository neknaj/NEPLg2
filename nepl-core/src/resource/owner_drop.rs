extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::drop_plan::auto_drop_candidates_for_end_scope;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::place_utils::place_suffix_after_prefix;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn drop_owner_obligation(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        place: &Place,
        span: Span,
    ) {
        if self.has_transferable_owner(owners, raw_aliases, place) {
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                place,
                ResourceOwnerOperation::Drop,
                span,
            );
        }
        raw_aliases.clear(place);
        raw_views.clear(place);
        storage_origins.clear(place);
        pending_reallocs.clear_result(place);
    }

    pub(super) fn auto_drop_scope_owner_obligations(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        locals: &[Place],
        result: Option<&Place>,
        span: Span,
    ) {
        let mut drop_places = Vec::new();
        for candidate in auto_drop_candidates_for_end_scope(self.types, locals, span) {
            if scope_result_preserves_place(
                owners,
                raw_aliases,
                storage_origins,
                result,
                &candidate.place,
            ) {
                push_owned_leaf_drop_places(
                    self,
                    owners,
                    raw_aliases,
                    storage_origins,
                    result,
                    &candidate.place,
                    &mut drop_places,
                );
            } else {
                push_unique_drop_place(&mut drop_places, candidate.place);
            }
        }
        for local in locals.iter().rev() {
            push_owned_leaf_drop_places(
                self,
                owners,
                raw_aliases,
                storage_origins,
                result,
                local,
                &mut drop_places,
            );
        }
        for place in drop_places {
            self.drop_owner_obligation(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                pending_reallocs,
                &place,
                span,
            );
        }
    }
}

fn push_owned_leaf_drop_places(
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    storage_origins: &StorageOriginTable,
    result: Option<&Place>,
    local: &Place,
    drop_places: &mut Vec<Place>,
) {
    if planned_drop_covers(drop_places, local) {
        return;
    }
    for leaf in owner_leaf_places(engine.types, local) {
        if planned_drop_covers(drop_places, &leaf.place)
            || scope_result_preserves_place(
                owners,
                raw_aliases,
                storage_origins,
                result,
                &leaf.place,
            )
            || !engine.has_transferable_owner(owners, raw_aliases, &leaf.place)
        {
            continue;
        }
        push_unique_drop_place(drop_places, leaf.place);
    }
}

fn push_unique_drop_place(drop_places: &mut Vec<Place>, place: Place) {
    if !drop_places.iter().any(|existing| existing == &place) {
        drop_places.push(place);
    }
}

fn planned_drop_covers(drop_places: &[Place], place: &Place) -> bool {
    drop_places.iter().any(|drop_place| {
        place == drop_place || place_suffix_after_prefix(place, drop_place).is_some()
    })
}

fn scope_result_preserves_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    storage_origins: &StorageOriginTable,
    result: Option<&Place>,
    place: &Place,
) -> bool {
    let Some(result) = result else {
        return false;
    };
    if places_overlap_result(place, result) {
        return true;
    }
    let resolved = resolve_owner_alias_place(owners, raw_aliases, place);
    if places_overlap_result(&resolved, result) {
        return true;
    }
    if storage_origins.has_origin_source_under(result, place) {
        return true;
    }
    raw_aliases
        .aliases_for(place)
        .iter()
        .any(|alias| places_overlap_result(alias, result))
}

fn places_overlap_result(place: &Place, result: &Place) -> bool {
    place == result
        || place_suffix_after_prefix(place, result).is_some()
        || place_suffix_after_prefix(result, place).is_some()
}

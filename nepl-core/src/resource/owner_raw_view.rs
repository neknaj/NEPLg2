use alloc::vec::Vec;

use crate::types::{TypeId, TypeKind};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;
use super::place_utils::{place_suffix_after_prefix, push_unique_place, replace_place_prefix};

#[derive(Clone, Default)]
pub(super) struct RawAddressViewTable {
    places: Vec<Place>,
}

impl RawAddressViewTable {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        self.places.push(place.clone());
    }

    pub(super) fn copy(&mut self, source: &Place, target: &Place) {
        let copied = self
            .places
            .iter()
            .filter_map(|place| replace_place_prefix(place, source, target))
            .collect::<Vec<_>>();
        self.clear(target);
        for place in copied {
            push_unique_place(&mut self.places, &place);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        self.places
            .retain(|entry| place_suffix_after_prefix(entry, place).is_none());
    }

    pub(super) fn contains(&self, place: &Place) -> bool {
        self.places.iter().any(|entry| entry == place)
    }

    pub(super) fn contains_under(&self, prefix: &Place) -> bool {
        self.places
            .iter()
            .any(|entry| place_suffix_after_prefix(entry, prefix).is_some())
    }

    pub(super) fn merge_paths(paths: &[RawAddressViewTable]) -> Self {
        let mut out = RawAddressViewTable::default();
        for path in paths {
            for place in &path.places {
                if !out.contains(place) {
                    out.places.push(place.clone());
                }
            }
        }
        out
    }
}

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn raw_memory_load_is_non_owning_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        cell: &Place,
        output_ty: TypeId,
    ) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(output_ty)),
            TypeKind::I32
        ) && (raw_aliases.contains_marked_alias(cell)
            || raw_aliases.aliases_for(cell).iter().any(|alias| {
                matches!(
                    owners.state(alias),
                    Some(OwnerState::Live { .. } | OwnerState::MaybeFreed { .. })
                )
            }))
    }

    pub(super) fn raw_store_value_is_non_owning_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        value: &Place,
    ) -> bool {
        self.types.resolve_id(value.ty) == self.types.i32()
            && raw_views.contains(value)
            && !owners.has_transferable_owner(value)
            && !owners.has_tracked_state_under(value)
            && raw_aliases
                .aliases_for(value)
                .iter()
                .any(|alias| alias != value)
    }
}

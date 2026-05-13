use crate::types::{TypeId, TypeKind};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, RawAddressViewKind};
use super::owner_check::ResourceOwnerCheckEngine;
pub(super) use super::owner_raw_view_table::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::place_utils::raw_address_view_candidate_bases;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
        kind: RawAddressViewKind,
    ) {
        let source_is_known =
            self.raw_address_view_source_is_known(owners, raw_aliases, storage_origins, source);
        if source_is_known {
            raw_aliases.copy_explicit_raw_address_alias(source, target);
            storage_origins.copy_origin(source, target);
        } else {
            raw_aliases.clear_raw_address_facts(target);
            storage_origins.clear(target);
        }
        if matches!(kind, RawAddressViewKind::NonOwningProjection) {
            raw_views.mark_non_owning_projection(target);
        } else if raw_views.contains_non_owning_projection(source) {
            raw_views.mark_non_owning_projection(target);
        } else if raw_views.contains_non_owning(source) {
            raw_views.mark_non_owning(target);
        } else if source_is_known {
            raw_views.mark(target);
        } else {
            raw_views.clear(target);
        }
    }

    pub(super) fn raw_address_view_source_is_known(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        source: &Place,
    ) -> bool {
        raw_aliases.raw_address_view_source_is_known(source)
            || raw_address_view_candidate_bases(source)
                .iter()
                .map(|base| raw_aliases.canonicalize(base))
                .any(|base| {
                    owners.has_tracked_state_under(&base) || storage_origins.origin(&base).is_some()
                })
    }

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
        (matches!(
            self.types.get_ref(self.types.resolve_id(value.ty)),
            TypeKind::I32
        ) || raw_views.contains_non_owning(value))
            && raw_views.contains_non_owning(value)
            && !self.has_transferable_owner(owners, raw_aliases, value)
            && !owners.has_tracked_state_under(value)
            && raw_aliases
                .aliases_for(value)
                .iter()
                .any(|alias| alias != value)
    }

    pub(super) fn initializer_is_non_owning_raw_alias_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) -> bool {
        if !matches!(
            self.types.get_ref(self.types.resolve_id(source.ty)),
            TypeKind::I32
        ) || !matches!(
            self.types.get_ref(self.types.resolve_id(target.ty)),
            TypeKind::I32
        ) || owners.has_transferable_owner(source)
            || owners.has_tracked_state_under(source)
        {
            return false;
        }
        raw_aliases
            .aliases_for(source)
            .iter()
            .any(|alias| alias != source)
    }
}

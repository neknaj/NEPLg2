extern crate alloc;

use alloc::vec::Vec;

use crate::resource_primitives::type_is_raw_pointer;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::drop_plan::auto_drop_candidates_for_end_scope;
use super::drop_requirement::ResourceDropRequirement;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::place_utils::place_suffix_after_prefix;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn scope_auto_drop_owner_obligation_places(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        locals: &[Place],
        result: Option<&Place>,
        span: Span,
    ) -> Vec<Place> {
        let mut drop_places = Vec::new();
        self.collect_drop_requirement_scope_places(
            owners,
            raw_aliases,
            storage_origins,
            locals,
            result,
            span,
            &mut drop_places,
        );
        self.collect_state_only_scope_places(
            owners,
            raw_aliases,
            storage_origins,
            locals,
            result,
            &mut drop_places,
        );
        drop_places
    }

    fn collect_drop_requirement_scope_places(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        locals: &[Place],
        result: Option<&Place>,
        span: Span,
        drop_places: &mut Vec<Place>,
    ) {
        for candidate in auto_drop_candidates_for_end_scope(self.types, locals, span) {
            if matches!(candidate.requirement, ResourceDropRequirement::StateOnly) {
                continue;
            }
            if scope_result_preserves_place(
                owners,
                raw_aliases,
                storage_origins,
                result,
                &candidate.place,
            ) {
                self.push_owned_leaf_drop_places(
                    owners,
                    raw_aliases,
                    storage_origins,
                    result,
                    &candidate.place,
                    drop_places,
                );
            } else {
                push_unique_drop_place(drop_places, candidate.place);
            }
        }
    }

    fn collect_state_only_scope_places(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        locals: &[Place],
        result: Option<&Place>,
        drop_places: &mut Vec<Place>,
    ) {
        for local in locals.iter().rev() {
            for leaf in owner_leaf_places(self.types, local) {
                if self.state_only_leaf_can_auto_drop(leaf.place.ty) {
                    self.push_owned_leaf_drop_place(
                        owners,
                        raw_aliases,
                        storage_origins,
                        result,
                        leaf.place,
                        drop_places,
                    );
                }
            }
        }
    }

    fn push_owned_leaf_drop_places(
        &self,
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
        for leaf in owner_leaf_places(self.types, local) {
            self.push_owned_leaf_drop_place(
                owners,
                raw_aliases,
                storage_origins,
                result,
                leaf.place,
                drop_places,
            );
        }
    }

    fn push_owned_leaf_drop_place(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        result: Option<&Place>,
        place: Place,
        drop_places: &mut Vec<Place>,
    ) {
        if planned_drop_covers(drop_places, &place)
            || scope_result_preserves_place(owners, raw_aliases, storage_origins, result, &place)
            || !self.has_transferable_owner(owners, raw_aliases, &place)
        {
            return;
        }
        push_unique_drop_place(drop_places, place);
    }

    fn state_only_leaf_can_auto_drop(&self, ty: TypeId) -> bool {
        match self
            .types
            .get_ref(self.types.resolve_named_type_id(self.types.resolve_id(ty)))
        {
            TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Unit
            | TypeKind::Never
            | TypeKind::Reference(_, _)
            | TypeKind::Function { .. } => false,
            TypeKind::Struct { .. } if type_is_raw_pointer(self.types, ty) => false,
            TypeKind::Apply { .. } => !type_is_raw_pointer(self.types, ty),
            TypeKind::Struct { .. }
            | TypeKind::Tuple { .. }
            | TypeKind::Enum { .. }
            | TypeKind::Str
            | TypeKind::Named(_)
            | TypeKind::Box(_)
            | TypeKind::Var(_) => true,
        }
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
    storage_origins.has_origin_source_under(result, place)
        || raw_aliases
            .aliases_for(place)
            .iter()
            .any(|alias| places_overlap_result(alias, result))
}

fn places_overlap_result(place: &Place, result: &Place) -> bool {
    place == result
        || place_suffix_after_prefix(place, result).is_some()
        || place_suffix_after_prefix(result, place).is_some()
}

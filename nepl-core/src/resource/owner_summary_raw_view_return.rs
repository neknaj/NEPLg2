use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{Place, PlaceProjection};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_raw_view_model::RawAddressViewOwnership;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};
use super::summary::{OwnerNonOwningRawViewKind, OwnerNonOwningRawViewReturn};

pub(super) fn returned_projection_is_non_owning_raw_view(
    raw_views: &RawAddressViewTable,
    return_value: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> bool {
    raw_views.contains_non_owning_projection(&place_with_suffix(return_value, suffix, ty))
}

pub(super) fn record_non_owning_raw_view_returns(
    raw_views: &RawAddressViewTable,
    root: &Place,
    out: &mut Vec<OwnerNonOwningRawViewReturn>,
) {
    for (raw_view, ownership) in raw_views.non_owning_entries() {
        if let Some(suffix) = place_suffix_after_prefix(raw_view, root) {
            record_non_owning_raw_view_return(
                out,
                suffix,
                raw_view.ty,
                non_owning_raw_view_return_kind(ownership),
            );
        }
    }
}

fn record_non_owning_raw_view_return(
    out: &mut Vec<OwnerNonOwningRawViewReturn>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    kind: OwnerNonOwningRawViewKind,
) {
    if out
        .iter()
        .any(|entry| entry.suffix == suffix && entry.ty == ty && entry.kind == kind)
    {
        return;
    }
    out.push(OwnerNonOwningRawViewReturn { suffix, ty, kind });
}

fn non_owning_raw_view_return_kind(
    ownership: RawAddressViewOwnership,
) -> OwnerNonOwningRawViewKind {
    match ownership {
        RawAddressViewOwnership::NonOwning => OwnerNonOwningRawViewKind::AliasView,
        RawAddressViewOwnership::NonOwningProjection => OwnerNonOwningRawViewKind::ProjectionView,
        RawAddressViewOwnership::AddressView => OwnerNonOwningRawViewKind::AliasView,
    }
}

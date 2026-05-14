use alloc::vec::Vec;

use super::model::{Place, RawAddressViewKind};
use super::owner_raw_view::RawAddressViewTable;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix, push_unique_place};

pub(super) fn push_transferred_aliases(
    aliases: &mut Vec<Place>,
    source: &Place,
    target: &Place,
) -> bool {
    let transferred = transferred_aliases(source, target, aliases);
    let has_transferred = !transferred.is_empty();
    for alias in transferred {
        push_unique_place(aliases, &alias);
    }
    has_transferred
}

pub(super) fn push_transferred_aliases_from(
    aliases: &mut Vec<Place>,
    source: &Place,
    target: &Place,
    source_aliases: &[Place],
) -> bool {
    let transferred = transferred_aliases(source, target, source_aliases);
    let has_transferred = !transferred.is_empty();
    for alias in transferred {
        push_unique_place(aliases, &alias);
    }
    has_transferred
}

pub(super) fn push_transferred_value_aliases(
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    source: &Place,
    target: &Place,
) -> bool {
    raw_views.copy_non_owning(source, target);
    push_transferred_aliases(aliases, source, target)
}

pub(super) fn push_transferred_value_aliases_from(
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    source: &Place,
    target: &Place,
    source_aliases: &[Place],
    source_raw_views: &RawAddressViewTable,
) -> bool {
    raw_views.copy_non_owning(source, target);
    copy_non_owning_raw_views_from(raw_views, source, target, source_raw_views);
    push_transferred_aliases_from(aliases, source, target, source_aliases)
}

pub(super) fn push_transferred_raw_owner_view_aliases(
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    source: &Place,
    target: &Place,
    kind: RawAddressViewKind,
) -> bool {
    match raw_owner_alias_transfer_kind(raw_views, source, kind) {
        RawOwnerAliasTransferKind::NonOwningProjection => {
            raw_views.mark_non_owning_projection(target);
            false
        }
        RawOwnerAliasTransferKind::NonOwning => {
            raw_views.mark_non_owning(target);
            false
        }
        RawOwnerAliasTransferKind::OwnerAlias => {
            raw_views.clear(target);
            push_transferred_aliases(aliases, source, target)
        }
    }
}

pub(super) fn place_matches_any_alias(place: &Place, aliases: &[Place]) -> bool {
    aliases.iter().any(|alias| {
        place == alias
            || place_suffix_after_prefix(place, alias).is_some()
            || place_suffix_after_prefix(alias, place).is_some()
    })
}

fn transferred_aliases(source: &Place, target: &Place, aliases: &[Place]) -> Vec<Place> {
    let mut out = Vec::new();
    for alias in aliases {
        if alias == source {
            push_unique_place(&mut out, target);
        } else if let Some(suffix) = place_suffix_after_prefix(alias, source) {
            push_unique_place(&mut out, &place_with_suffix(target, &suffix, alias.ty));
        } else if place_suffix_after_prefix(source, alias).is_some() {
            push_unique_place(&mut out, target);
        }
    }
    out
}

enum RawOwnerAliasTransferKind {
    OwnerAlias,
    NonOwningProjection,
    NonOwning,
}

fn raw_owner_alias_transfer_kind(
    raw_views: &RawAddressViewTable,
    source: &Place,
    kind: RawAddressViewKind,
) -> RawOwnerAliasTransferKind {
    match kind {
        RawAddressViewKind::NonOwningProjection => RawOwnerAliasTransferKind::NonOwningProjection,
        RawAddressViewKind::Offset if raw_views.contains_non_owning_projection(source) => {
            RawOwnerAliasTransferKind::NonOwningProjection
        }
        RawAddressViewKind::Offset if raw_views.contains_non_owning(source) => {
            RawOwnerAliasTransferKind::NonOwning
        }
        RawAddressViewKind::Offset => RawOwnerAliasTransferKind::OwnerAlias,
    }
}

fn copy_non_owning_raw_views_from(
    raw_views: &mut RawAddressViewTable,
    source: &Place,
    target: &Place,
    source_raw_views: &RawAddressViewTable,
) {
    if source_raw_views.contains_non_owning_projection(source) {
        raw_views.mark_non_owning_projection(target);
    } else if source_raw_views.contains_non_owning(source) {
        raw_views.mark_non_owning(target);
    }
}

#[cfg(test)]
#[path = "owner_summary_raw_transfer_tests.rs"]
mod tests;

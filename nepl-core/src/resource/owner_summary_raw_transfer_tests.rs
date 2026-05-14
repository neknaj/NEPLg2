use alloc::string::String;
use alloc::vec;

use super::*;

fn place(name: &str) -> Place {
    Place::local(String::from(name), crate::types::TypeId(0))
}

#[test]
fn non_owning_raw_address_view_does_not_transfer_owner_alias() {
    let source = place("source");
    let target = place("target");
    let mut aliases = vec![source.clone()];
    let mut raw_views = RawAddressViewTable::default();

    let transferred = push_transferred_raw_owner_view_aliases(
        &mut aliases,
        &mut raw_views,
        &source,
        &target,
        RawAddressViewKind::NonOwningProjection,
    );

    assert!(!transferred);
    assert!(!aliases.contains(&target));
    assert!(raw_views.contains_non_owning_projection(&target));
}

#[test]
fn offset_raw_address_view_transfers_owner_alias() {
    let source = place("source");
    let target = place("target");
    let mut aliases = vec![source.clone()];
    let mut raw_views = RawAddressViewTable::default();

    let transferred = push_transferred_raw_owner_view_aliases(
        &mut aliases,
        &mut raw_views,
        &source,
        &target,
        RawAddressViewKind::Offset,
    );

    assert!(transferred);
    assert!(aliases.contains(&target));
}

#[test]
fn offset_from_non_owning_projection_stays_non_owning() {
    let source = place("source");
    let target = place("target");
    let mut aliases = vec![source.clone()];
    let mut raw_views = RawAddressViewTable::default();
    raw_views.mark_non_owning_projection(&source);

    let transferred = push_transferred_raw_owner_view_aliases(
        &mut aliases,
        &mut raw_views,
        &source,
        &target,
        RawAddressViewKind::Offset,
    );

    assert!(!transferred);
    assert!(!aliases.contains(&target));
    assert!(raw_views.contains_non_owning_projection(&target));
}

#[test]
fn offset_from_non_owning_stays_non_owning() {
    let source = place("source");
    let target = place("target");
    let mut aliases = vec![source.clone()];
    let mut raw_views = RawAddressViewTable::default();
    raw_views.mark_non_owning(&source);

    let transferred = push_transferred_raw_owner_view_aliases(
        &mut aliases,
        &mut raw_views,
        &source,
        &target,
        RawAddressViewKind::Offset,
    );

    assert!(!transferred);
    assert!(!aliases.contains(&target));
    assert!(raw_views.contains_non_owning(&target));
}

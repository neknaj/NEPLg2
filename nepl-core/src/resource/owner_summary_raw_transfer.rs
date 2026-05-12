use alloc::vec::Vec;

use super::model::{Place, RawAddressViewKind};
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

pub(super) fn push_transferred_raw_owner_view_aliases(
    aliases: &mut Vec<Place>,
    source: &Place,
    target: &Place,
    kind: RawAddressViewKind,
) -> bool {
    if !raw_address_view_carries_owner_alias(kind) {
        return false;
    }
    push_transferred_aliases(aliases, source, target)
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

fn raw_address_view_carries_owner_alias(kind: RawAddressViewKind) -> bool {
    match kind {
        RawAddressViewKind::Offset => true,
        RawAddressViewKind::NonOwningProjection => false,
    }
}

#[cfg(test)]
mod tests {
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

        let transferred = push_transferred_raw_owner_view_aliases(
            &mut aliases,
            &source,
            &target,
            RawAddressViewKind::NonOwningProjection,
        );

        assert!(!transferred);
        assert!(!aliases.contains(&target));
    }

    #[test]
    fn offset_raw_address_view_transfers_owner_alias() {
        let source = place("source");
        let target = place("target");
        let mut aliases = vec![source.clone()];

        let transferred = push_transferred_raw_owner_view_aliases(
            &mut aliases,
            &source,
            &target,
            RawAddressViewKind::Offset,
        );

        assert!(transferred);
        assert!(aliases.contains(&target));
    }
}

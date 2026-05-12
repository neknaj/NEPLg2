use alloc::vec;

use super::model::{Place, ResourceFunction, ResourceTerminator};
use super::owner_summary_raw_alias::{collect_raw_owner_aliases, place_matches_any_alias};
use super::owner_summary_raw_use::ops_use_raw_owner_alias;

pub(super) fn function_consumes_raw_owner_from(function: &ResourceFunction, place: &Place) -> bool {
    function.blocks.iter().any(|block| {
        let mut aliases = vec![place.clone()];
        ops_use_raw_owner_alias(&block.ops, &mut aliases)
    })
}

pub(super) fn function_returns_raw_owner_from(function: &ResourceFunction, place: &Place) -> bool {
    function.blocks.iter().any(|block| {
        let mut aliases = vec![place.clone()];
        collect_raw_owner_aliases(&block.ops, &mut aliases);
        matches!(
            &block.terminator,
            ResourceTerminator::Return {
                value: Some(value),
                ..
            } if place_matches_any_alias(value, &aliases)
        )
    })
}

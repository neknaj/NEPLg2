use alloc::vec;
use alloc::vec::Vec;

use crate::resource_primitives::type_is_raw_pointer;
use crate::types::TypeCtx;

use super::model::{Place, ResourceFunction, ResourceTerminator};
use super::owner_summary_i32_leaf::raw_i32_owner_leaf_places;
use super::owner_summary_leaf::{owner_leaf_places, OwnerLeafPlace};
use super::owner_summary_owner_token_type::type_contains_owner_token;
use super::owner_summary_raw_alias::collect_raw_owner_aliases;
use super::owner_summary_raw_consumption::{
    function_consumes_raw_owner_from, function_returns_raw_owner_from,
};
use super::place_utils::type_can_seed_raw_address_alias;
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn owner_seed_leaf_places(
    types: &TypeCtx,
    function: &ResourceFunction,
    summaries: &OwnerReturnSummaryIndex<'_>,
    _parameter_index: usize,
    base: &Place,
) -> Vec<OwnerLeafPlace> {
    let mut leaves = if type_can_seed_structural_owner_leaf(types, base) {
        owner_leaf_places(types, base)
    } else {
        Vec::new()
    };
    for leaf in raw_i32_owner_leaf_places(types, base) {
        if raw_i32_leaf_is_copy_metadata(types, base, &leaf) {
            continue;
        }
        let consumes_raw_owner = function_consumes_raw_owner_from(function, &leaf.place, summaries);
        let returns_raw_owner = function_returns_raw_owner_from(function, &leaf.place, summaries);
        let returns_seedable_raw_owner = if leaf.suffix.is_empty() {
            function_returns_raw_owner_inside_returned_owner_leaf(
                types,
                function,
                &leaf.place,
                summaries,
            )
        } else {
            returns_raw_owner
        };
        if (consumes_raw_owner || returns_seedable_raw_owner)
            && !leaves
                .iter()
                .any(|existing| existing.place == leaf.place && existing.suffix == leaf.suffix)
        {
            leaves.push(leaf);
        }
    }
    leaves
}

fn type_can_seed_structural_owner_leaf(types: &TypeCtx, base: &Place) -> bool {
    type_is_raw_pointer(types, base.ty)
        || type_contains_owner_token(types, base.ty)
        || !types.is_copy(base.ty)
}

fn function_returns_raw_owner_inside_returned_owner_leaf(
    types: &TypeCtx,
    function: &ResourceFunction,
    place: &Place,
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> bool {
    function.blocks.iter().any(|block| {
        let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        else {
            return false;
        };
        let mut aliases = vec![place.clone()];
        collect_raw_owner_aliases(&block.ops, &mut aliases, summaries);
        raw_i32_owner_leaf_places(types, value)
            .into_iter()
            .any(|leaf| aliases.iter().any(|alias| alias == &leaf.place))
    })
}

pub(super) fn raw_i32_leaf_is_copy_metadata(
    types: &TypeCtx,
    base: &Place,
    leaf: &OwnerLeafPlace,
) -> bool {
    if leaf.suffix.is_empty()
        || type_is_raw_pointer(types, base.ty)
        || type_contains_owner_token(types, base.ty)
    {
        return false;
    }
    if types.is_copy(base.ty) {
        return true;
    }
    // A non-Copy aggregate can contain ordinary scalar counters next to owned
    // fields. Those counters may flow through arithmetic and be returned in a
    // rebuilt aggregate, but they are not raw owners unless the aggregate is a
    // raw-address carrier.
    !type_can_seed_raw_address_alias(types, base.ty)
}

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::Place;
use super::owner_summary_i32_condition_leaf::i32_leaf_places_for_conditions;
use super::owner_summary_leaf::OwnerLeafPlace;
use super::owner_summary_raw_i32_leaf::raw_i32_owner_leaf_places_for_summary;

pub(super) fn i32_leaf_places(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    i32_leaf_places_for_conditions(types, base)
}

pub(super) fn raw_i32_owner_leaf_places(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    raw_i32_owner_leaf_places_for_summary(types, base)
}

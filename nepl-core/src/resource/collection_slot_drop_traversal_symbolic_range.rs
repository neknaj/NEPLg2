use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp};

pub(super) fn symbolic_slot_offset_is_inside_initialized_count(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    index: &Place,
    scale: usize,
    known: usize,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> bool {
    let stride = storage_size_bytes(types, expected_ty);
    let non_negative = raw_aliases.i32_condition_truth(index, I32ValueCondition::NonNegative);
    let relation =
        raw_aliases.i32_relation_truth(index, ResourceI32RelationOp::Lt, initialized_count);
    stride > 0
        && known == 0
        && scale == stride
        && non_negative == Some(true)
        && relation == Some(true)
}

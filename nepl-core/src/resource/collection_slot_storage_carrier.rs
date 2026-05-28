extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::collection_slot_storage_carrier_walk::type_can_carry_collection_slot_storage_mapped;

pub(super) fn type_can_carry_collection_slot_storage(types: &TypeCtx, ty: TypeId) -> bool {
    type_can_carry_collection_slot_storage_mapped(types, ty, &BTreeMap::new(), &mut Vec::new())
}

#[cfg(test)]
#[path = "collection_slot_storage_carrier_tests.rs"]
mod tests;

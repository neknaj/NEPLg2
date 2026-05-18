use crate::resource_primitives::type_is_raw_pointer;
use crate::types::TypeCtx;

use super::compiler_memory_place::mem_ptr_raw_field_place;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn host_memory_address_place(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
) -> Place {
    let raw_address = if type_is_raw_pointer(types, address.ty) {
        mem_ptr_raw_field_place(types, address, types.i32())
    } else {
        address.clone()
    };
    raw_aliases.canonicalize(&raw_address)
}

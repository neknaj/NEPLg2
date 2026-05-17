use crate::layout::aggregate_fields_with_offsets;
use crate::resource_primitives::{
    compiler_memory_type_field_index, compiler_memory_type_field_offset_bytes,
    type_is_compiler_memory_type, CompilerMemoryFieldSpec,
};
use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeId};

use super::model::{Place, PlaceProjection};

pub(super) fn mem_ptr_raw_field_place(types: &TypeCtx, ptr: &Place, raw_ty: TypeId) -> Place {
    compiler_memory_field_place(
        types,
        ptr,
        CompilerMemoryType::RawPointer,
        CompilerMemoryFieldSpec::RawI32,
        raw_ty,
    )
    .expect("MemPtr raw projection requires proven compiler memory pointer type")
}

pub(super) fn region_token_raw_field_place(
    types: &TypeCtx,
    token: &Place,
    raw_ty: TypeId,
) -> Place {
    compiler_memory_field_place(
        types,
        token,
        CompilerMemoryType::OwnerToken,
        CompilerMemoryFieldSpec::RawI32,
        raw_ty,
    )
    .expect("RegionToken raw projection requires proven compiler owner token type")
}

pub(super) fn region_token_size_field_for_raw_owner(raw: &Place) -> Option<Place> {
    let raw_projection = compiler_memory_field_projection(
        CompilerMemoryType::OwnerToken,
        CompilerMemoryFieldSpec::RawI32,
    )?;
    let size_projection = compiler_memory_field_projection(
        CompilerMemoryType::OwnerToken,
        CompilerMemoryFieldSpec::SizeI32,
    )?;
    let mut size = raw.clone();
    let projection = size.projections.last_mut()?;
    if *projection == raw_projection {
        *projection = size_projection;
        Some(size)
    } else {
        None
    }
}

fn compiler_memory_field_place(
    types: &TypeCtx,
    place: &Place,
    memory_type: CompilerMemoryType,
    field: CompilerMemoryFieldSpec,
    field_ty: TypeId,
) -> Option<Place> {
    if !type_is_compiler_memory_type(types, place.ty, memory_type) {
        return None;
    }
    let index = compiler_memory_type_field_index(memory_type, field)?;
    let field_layout = *aggregate_fields_with_offsets(types, place.ty).get(index)?;
    if field.requires_i32() && types.resolve_id(field_layout.ty) != types.i32() {
        return None;
    }
    Some(place.clone().with_projection(
        PlaceProjection::Field {
            index,
            offset_bytes: field_layout.offset,
        },
        field_ty,
    ))
}

fn compiler_memory_field_projection(
    memory_type: CompilerMemoryType,
    field: CompilerMemoryFieldSpec,
) -> Option<PlaceProjection> {
    Some(PlaceProjection::Field {
        index: compiler_memory_type_field_index(memory_type, field)?,
        offset_bytes: compiler_memory_type_field_offset_bytes(memory_type, field)?,
    })
}

#[cfg(test)]
#[path = "compiler_memory_place_tests.rs"]
mod tests;

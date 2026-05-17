use crate::source_map::{CompilerMemoryField, CompilerMemoryType};
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(crate) const RAW_POINTER_TYPE_NAME: &str = "MemPtr";
pub(crate) const OWNER_TOKEN_TYPE_NAME: &str = "RegionToken";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompilerMemoryFieldSpec {
    RawI32,
    SizeI32,
}

const RAW_POINTER_FIELDS: &[CompilerMemoryFieldSpec] = &[CompilerMemoryFieldSpec::RawI32];
const OWNER_TOKEN_FIELDS: &[CompilerMemoryFieldSpec] = &[
    CompilerMemoryFieldSpec::RawI32,
    CompilerMemoryFieldSpec::SizeI32,
];
const COMPILER_MEMORY_I32_FIELD_BYTES: usize = 4;

impl CompilerMemoryFieldSpec {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::RawI32 => "raw",
            Self::SizeI32 => "size",
        }
    }

    pub(crate) const fn requires_i32(self) -> bool {
        match self {
            Self::RawI32 | Self::SizeI32 => true,
        }
    }

    pub(crate) const fn proof_field(self) -> CompilerMemoryField {
        match self {
            Self::RawI32 => CompilerMemoryField::Raw,
            Self::SizeI32 => CompilerMemoryField::Size,
        }
    }
}

pub(crate) fn compiler_memory_type_from_constructor_name(name: &str) -> Option<CompilerMemoryType> {
    match name {
        RAW_POINTER_TYPE_NAME => Some(CompilerMemoryType::RawPointer),
        OWNER_TOKEN_TYPE_NAME => Some(CompilerMemoryType::OwnerToken),
        _ => None,
    }
}

pub(crate) fn compiler_memory_type_field_specs(
    memory_type: CompilerMemoryType,
) -> &'static [CompilerMemoryFieldSpec] {
    match memory_type {
        CompilerMemoryType::RawPointer => RAW_POINTER_FIELDS,
        CompilerMemoryType::OwnerToken => OWNER_TOKEN_FIELDS,
    }
}

pub(crate) fn compiler_memory_type_field_index(
    memory_type: CompilerMemoryType,
    field: CompilerMemoryFieldSpec,
) -> Option<usize> {
    compiler_memory_type_field_specs(memory_type)
        .iter()
        .position(|spec| *spec == field)
}

pub(crate) fn compiler_memory_type_field_offset_bytes(
    memory_type: CompilerMemoryType,
    field: CompilerMemoryFieldSpec,
) -> Option<usize> {
    let mut offset = 0usize;
    for spec in compiler_memory_type_field_specs(memory_type) {
        if *spec == field {
            return Some(offset);
        }
        if spec.requires_i32() {
            offset += COMPILER_MEMORY_I32_FIELD_BYTES;
        }
    }
    None
}

pub(crate) fn compiler_memory_type_of_type(
    types: &TypeCtx,
    ty: TypeId,
) -> Option<CompilerMemoryType> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } => types.compiler_memory_type(resolved),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            types.compiler_memory_type(base)
        }
        _ => None,
    }
}

pub(crate) fn type_is_compiler_memory_type(
    types: &TypeCtx,
    ty: TypeId,
    memory_type: CompilerMemoryType,
) -> bool {
    compiler_memory_type_of_type(types, ty) == Some(memory_type)
}

pub(crate) fn type_is_raw_pointer(types: &TypeCtx, ty: TypeId) -> bool {
    type_is_compiler_memory_type(types, ty, CompilerMemoryType::RawPointer)
}

pub(crate) fn type_is_owner_token(types: &TypeCtx, ty: TypeId) -> bool {
    type_is_compiler_memory_type(types, ty, CompilerMemoryType::OwnerToken)
}

pub(crate) fn type_preserves_raw_address_identity(types: &TypeCtx, ty: TypeId) -> bool {
    matches!(
        compiler_memory_type_of_type(types, ty),
        Some(CompilerMemoryType::RawPointer | CompilerMemoryType::OwnerToken)
    )
}

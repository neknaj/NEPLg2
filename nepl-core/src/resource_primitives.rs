use crate::runtime_helpers::helper_base_name;
use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(crate) const RAW_POINTER_TYPE_NAME: &str = "MemPtr";
pub(crate) const OWNER_TOKEN_TYPE_NAME: &str = "RegionToken";

pub(crate) fn compiler_memory_type_from_constructor_name(name: &str) -> Option<CompilerMemoryType> {
    match name {
        RAW_POINTER_TYPE_NAME => Some(CompilerMemoryType::RawPointer),
        OWNER_TOKEN_TYPE_NAME => Some(CompilerMemoryType::OwnerToken),
        _ => None,
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryHelperPrimitive {
    MemPtrWrap,
    MemPtrAddr,
    MemPtrAdd,
    RegionNew,
    RegionPtr,
    RegionPtrAt,
    RegionTokenRawRef,
    StrAddr,
    StrFromAddrUnchecked,
}

impl MemoryHelperPrimitive {
    pub(crate) fn from_symbol(name: &str) -> Option<Self> {
        Self::from_base_name(helper_base_name(name))
    }

    pub(crate) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "mem_ptr_wrap" => Some(Self::MemPtrWrap),
            "mem_ptr_addr" => Some(Self::MemPtrAddr),
            "mem_ptr_add" => Some(Self::MemPtrAdd),
            "region_new" => Some(Self::RegionNew),
            "region_ptr" => Some(Self::RegionPtr),
            "region_ptr_at" => Some(Self::RegionPtrAt),
            "region_token_raw_ref" => Some(Self::RegionTokenRawRef),
            "str_addr" => Some(Self::StrAddr),
            "str_from_addr_unchecked" => Some(Self::StrFromAddrUnchecked),
            _ => None,
        }
    }

    pub(crate) const fn has_resource_call_lowering(self) -> bool {
        matches!(
            self,
            Self::MemPtrWrap
                | Self::MemPtrAddr
                | Self::MemPtrAdd
                | Self::RegionNew
                | Self::RegionPtr
                | Self::RegionPtrAt
                | Self::RegionTokenRawRef
        )
    }

    pub(crate) const fn has_dedicated_raw_address_lowering(self) -> bool {
        self.has_resource_call_lowering()
    }

    pub(crate) const fn returns_non_owning_address_view(self) -> bool {
        matches!(self, Self::MemPtrAddr | Self::RegionPtr | Self::StrAddr)
    }

    pub(crate) const fn is_raw_address_view_boundary_evidence(self) -> bool {
        match self {
            Self::MemPtrWrap
            | Self::MemPtrAddr
            | Self::MemPtrAdd
            | Self::RegionPtr
            | Self::RegionPtrAt
            | Self::RegionTokenRawRef
            | Self::StrAddr
            | Self::StrFromAddrUnchecked => true,
            Self::RegionNew => false,
        }
    }
}

#[cfg(test)]
#[path = "resource_primitives_tests.rs"]
mod tests;

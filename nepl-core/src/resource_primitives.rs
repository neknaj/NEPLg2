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
        TypeKind::Struct { name, .. } => compiler_memory_type_from_constructor_name(name),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { name, .. } => compiler_memory_type_from_constructor_name(name),
                _ => None,
            }
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

    pub(crate) const fn has_dedicated_raw_address_lowering(self) -> bool {
        matches!(self, Self::MemPtrAddr | Self::RegionPtr)
    }

    pub(crate) const fn returns_non_owning_address_view(self) -> bool {
        matches!(self, Self::MemPtrAddr | Self::RegionPtr | Self::StrAddr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn register_memory_struct(types: &mut TypeCtx, name: &str, field_names: &[&str]) -> TypeId {
        let type_param = types.fresh_var(Some("T".to_string()));
        let i32_ty = types.i32();
        let fields = field_names.iter().map(|_| i32_ty).collect();
        let field_names = field_names.iter().map(|name| (*name).to_string()).collect();
        types.register_named(
            name.to_string(),
            TypeKind::Struct {
                doc: None,
                name: name.to_string(),
                type_params: vec![type_param],
                fields,
                field_names,
            },
        )
    }

    #[test]
    fn compiler_memory_type_classifies_base_and_applied_types() {
        let mut types = TypeCtx::new();
        let mem_ptr = register_memory_struct(&mut types, RAW_POINTER_TYPE_NAME, &["raw"]);
        let region = register_memory_struct(&mut types, OWNER_TOKEN_TYPE_NAME, &["raw", "size"]);
        let u8_ty = types.u8();
        let applied_mem_ptr = types.apply(mem_ptr, vec![u8_ty]);
        let applied_region = types.apply(region, vec![u8_ty]);

        assert_eq!(
            compiler_memory_type_of_type(&types, mem_ptr),
            Some(CompilerMemoryType::RawPointer)
        );
        assert_eq!(
            compiler_memory_type_of_type(&types, applied_mem_ptr),
            Some(CompilerMemoryType::RawPointer)
        );
        assert_eq!(
            compiler_memory_type_of_type(&types, region),
            Some(CompilerMemoryType::OwnerToken)
        );
        assert_eq!(
            compiler_memory_type_of_type(&types, applied_region),
            Some(CompilerMemoryType::OwnerToken)
        );
        assert!(!type_is_raw_pointer(&types, applied_region));
        assert!(!type_is_owner_token(&types, applied_mem_ptr));
    }

    #[test]
    fn memory_helper_primitive_classifies_suffixed_symbols() {
        assert_eq!(
            MemoryHelperPrimitive::from_symbol("core/mem::mem_ptr_addr__u8"),
            Some(MemoryHelperPrimitive::MemPtrAddr)
        );
        assert_eq!(
            MemoryHelperPrimitive::from_symbol("region_ptr_at__i32"),
            Some(MemoryHelperPrimitive::RegionPtrAt)
        );
        assert_eq!(MemoryHelperPrimitive::from_symbol("alloc_region"), None);
    }
}

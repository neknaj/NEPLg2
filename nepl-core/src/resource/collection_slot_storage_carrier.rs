use crate::resource_primitives::type_is_raw_pointer;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_owner_carrier::type_carries_collection_slot_owner;

pub(super) fn type_can_carry_collection_slot_storage(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    if type_carries_collection_slot_owner(types, resolved) {
        return true;
    }
    if type_is_raw_pointer(types, resolved) {
        return false;
    }
    match types.get_ref(resolved) {
        TypeKind::Struct { .. }
        | TypeKind::Enum { .. }
        | TypeKind::Tuple { .. }
        | TypeKind::Apply { .. }
        | TypeKind::Box(_)
        | TypeKind::Var(_) => !types.is_copy(resolved),
        TypeKind::Named(_) => false,
        TypeKind::Reference(_, _)
        | TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::source_map::CompilerMemoryType;

    use super::*;

    fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
        types.register_named(
            name.to_string(),
            TypeKind::Struct {
                name: name.to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        )
    }

    #[test]
    fn non_copy_aggregate_can_carry_collection_slot_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let storage_ty = register_empty_struct(&mut types, "CollectionStorage");

        assert!(type_can_carry_collection_slot_storage(&types, storage_ty));
    }

    #[test]
    fn copy_scalar_and_raw_pointer_do_not_carry_collection_slot_storage() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let mem_ptr_ty = register_empty_struct(&mut types, "MemPtr");
        types.mark_compiler_memory_type(mem_ptr_ty, CompilerMemoryType::RawPointer);

        assert!(!type_can_carry_collection_slot_storage(&types, types.i32()));
        assert!(!type_can_carry_collection_slot_storage(&types, mem_ptr_ty));
    }
}

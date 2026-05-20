use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(crate) fn compiler_memory_value_type(
    types: &TypeCtx,
    ty: TypeId,
) -> Option<(CompilerMemoryType, TypeId)> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Apply { base, args } if args.len() == 1 => {
            let memory_type = types.compiler_memory_type(*base)?;
            Some((memory_type, args[0]))
        }
        TypeKind::Reference(target, _) => compiler_memory_value_type(types, *target),
        _ => None,
    }
}

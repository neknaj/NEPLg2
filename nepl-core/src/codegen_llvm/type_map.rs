use crate::types::{TypeCtx, TypeId, TypeKind};

use super::LlTy;

pub(super) fn llty_for_type(types: &TypeCtx, ty: TypeId) -> LlTy {
    match types.get(types.resolve_id(ty)) {
        TypeKind::Unit | TypeKind::Never => LlTy::Void,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Char | TypeKind::Str => LlTy::I32,
        TypeKind::F32 => LlTy::F32,
        TypeKind::Named(name) if name == "i64" || name == "u64" => LlTy::I64,
        TypeKind::Named(name) if name == "f64" => LlTy::F64,
        TypeKind::Reference(_, _) => LlTy::I32,
        TypeKind::Box(_) => LlTy::I32,
        TypeKind::Tuple { .. } => LlTy::I32,
        TypeKind::Struct { .. } => LlTy::I32,
        TypeKind::Enum { .. } => LlTy::I32,
        TypeKind::Apply { .. } => LlTy::I32,
        TypeKind::Function { .. } => LlTy::I32,
        TypeKind::Var(_) => LlTy::I32,
        TypeKind::Named(_) => LlTy::I32,
    }
}

use crate::backend_scalar_type::BackendScalarType;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::LlTy;

pub(super) fn llty_for_type(types: &TypeCtx, ty: TypeId) -> LlTy {
    match types.get(types.resolve_id(ty)) {
        TypeKind::Unit | TypeKind::Never => LlTy::Void,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Char | TypeKind::Str => LlTy::I32,
        TypeKind::F32 => LlTy::F32,
        TypeKind::Named(name) => match BackendScalarType::from_name(name.as_str()) {
            Some(scalar) if scalar.is_wasm_i64() => LlTy::I64,
            Some(scalar) if scalar.is_wasm_f64() => LlTy::F64,
            Some(_) | None => LlTy::I32,
        },
        TypeKind::Reference(_, _) => LlTy::I32,
        TypeKind::Box(_) => LlTy::I32,
        TypeKind::Tuple { .. } => LlTy::I32,
        TypeKind::Struct { .. } => LlTy::I32,
        TypeKind::Enum { .. } => LlTy::I32,
        TypeKind::Apply { .. } => LlTy::I32,
        TypeKind::Function { .. } => LlTy::I32,
        TypeKind::Var(_) => LlTy::I32,
    }
}

//! Backend-relevant named scalar type classification.

extern crate alloc;

use alloc::string::String;

use crate::ast::TypeExpr;
use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackendScalarType {
    U32,
    I64,
    U64,
    F64,
}

impl BackendScalarType {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "u32" => Some(Self::U32),
            "i64" => Some(Self::I64),
            "u64" => Some(Self::U64),
            "f64" => Some(Self::F64),
            _ => None,
        }
    }

    pub(crate) fn from_type_kind(kind: &TypeKind) -> Option<Self> {
        match kind {
            TypeKind::Named(name) => Self::from_name(name.as_str()),
            _ => None,
        }
    }

    pub(crate) fn from_type_expr(expr: &TypeExpr) -> Option<Self> {
        match expr.as_unspanned() {
            TypeExpr::Named(name) => Self::from_name(name.as_str()),
            _ => None,
        }
    }

    pub(crate) const fn source_name(self) -> &'static str {
        match self {
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F64 => "f64",
        }
    }

    pub(crate) fn type_id(self, types: &mut TypeCtx) -> TypeId {
        let name = String::from(self.source_name());
        types
            .lookup_named(name.as_str())
            .unwrap_or_else(|| types.register_named(name.clone(), TypeKind::Named(name)))
    }

    pub(crate) const fn storage_size_bytes(self) -> usize {
        match self {
            Self::U32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
        }
    }

    pub(crate) const fn storage_align_bytes(self) -> usize {
        match self {
            Self::U32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
        }
    }

    pub(crate) const fn is_wasm_i64(self) -> bool {
        matches!(self, Self::I64 | Self::U64)
    }

    pub(crate) const fn is_wasm_f64(self) -> bool {
        matches!(self, Self::F64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_named_backend_scalars() {
        assert_eq!(
            BackendScalarType::from_name("u32"),
            Some(BackendScalarType::U32)
        );
        assert_eq!(
            BackendScalarType::from_name("i64"),
            Some(BackendScalarType::I64)
        );
        assert_eq!(
            BackendScalarType::from_name("u64"),
            Some(BackendScalarType::U64)
        );
        assert_eq!(
            BackendScalarType::from_name("f64"),
            Some(BackendScalarType::F64)
        );
        assert_eq!(BackendScalarType::from_name("i32"), None);
    }

    #[test]
    fn owns_backend_storage_contracts() {
        assert_eq!(BackendScalarType::U32.storage_size_bytes(), 4);
        assert_eq!(BackendScalarType::U32.storage_align_bytes(), 4);
        assert_eq!(BackendScalarType::I64.storage_size_bytes(), 8);
        assert_eq!(BackendScalarType::U64.storage_align_bytes(), 8);
        assert!(BackendScalarType::I64.is_wasm_i64());
        assert!(BackendScalarType::U64.is_wasm_i64());
        assert!(BackendScalarType::F64.is_wasm_f64());
    }

    #[test]
    fn registers_named_type_ids_from_the_same_domain() {
        let mut types = TypeCtx::new();
        let first = BackendScalarType::U64.type_id(&mut types);
        let second = BackendScalarType::from_name("u64")
            .expect("u64 must be a backend scalar")
            .type_id(&mut types);
        assert_eq!(first, second);
        assert_eq!(types.get(first), TypeKind::Named(String::from("u64")));
    }

    #[test]
    fn classifies_type_syntax_and_type_kind() {
        let named = TypeExpr::Named(String::from("f64"));
        assert_eq!(
            BackendScalarType::from_type_expr(&named),
            Some(BackendScalarType::F64)
        );
        assert_eq!(
            BackendScalarType::from_type_kind(&TypeKind::Named(String::from("i64"))),
            Some(BackendScalarType::I64)
        );
        assert_eq!(BackendScalarType::from_type_kind(&TypeKind::I32), None);
    }
}

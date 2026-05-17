use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Visibility;
use crate::diagnostic::Diagnostic;
use crate::hir::{HirExpr, HirFunction};
use crate::types::{EnumVariantInfo, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StructConstructorPolicy {
    Public,
    RawMemoryBoundaryOnly(RestrictedStructConstructor),
    OwnerBackedAggregateBoundaryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RestrictedStructConstructor {
    OwnerToken,
    RawPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScalarMatchKind {
    I32,
    U8,
    Bool,
    Char,
}

#[derive(Debug)]
pub(super) struct CheckedFunction {
    pub(super) function: HirFunction,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub(super) struct EnumInfo {
    pub(super) ty: TypeId,
    pub(super) visibility: Visibility,
    pub(super) type_params: Vec<TypeId>,
    pub(super) variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
pub(super) struct StructInfo {
    pub(super) ty: TypeId,
    pub(super) visibility: Visibility,
    pub(super) type_params: Vec<TypeId>,
    pub(super) fields: Vec<TypeId>,
    pub(super) field_names: Vec<String>,
    pub(super) constructor_policy: StructConstructorPolicy,
}

#[derive(Debug, Clone)]
pub(super) enum FieldIdx {
    Index(usize),
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldAccessorKind {
    Get,
    GetRef,
    Put,
}

impl FieldAccessorKind {
    pub(super) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "get_field" => Some(Self::Get),
            "get_field_ref" => Some(Self::GetRef),
            "set_field" => Some(Self::Put),
            _ => None,
        }
    }

    pub(super) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::Get => "get_field",
            Self::GetRef => "get_field_ref",
            Self::Put => "set_field",
        }
    }

    pub(super) const fn argument_count(self) -> usize {
        match self {
            Self::Get | Self::GetRef => 2,
            Self::Put => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum AssignKind {
    Let,
    Set,
    AddrOf(bool),
    Deref,
}

#[cfg(test)]
mod tests {
    use super::FieldAccessorKind;

    #[test]
    fn field_accessor_intrinsic_names_round_trip_through_kind() {
        for kind in [
            FieldAccessorKind::Get,
            FieldAccessorKind::GetRef,
            FieldAccessorKind::Put,
        ] {
            assert_eq!(
                FieldAccessorKind::from_intrinsic_name(kind.intrinsic_name()),
                Some(kind)
            );
        }
        assert_eq!(FieldAccessorKind::from_intrinsic_name("get"), None);
    }

    #[test]
    fn field_accessor_intrinsic_argument_counts_are_kind_owned() {
        assert_eq!(FieldAccessorKind::Get.argument_count(), 2);
        assert_eq!(FieldAccessorKind::GetRef.argument_count(), 2);
        assert_eq!(FieldAccessorKind::Put.argument_count(), 3);
    }
}

#[derive(Debug, Clone)]
pub(super) struct StackEntry {
    pub(super) ty: TypeId,
    pub(super) expr: HirExpr,
    pub(super) type_args: Vec<TypeId>,
    pub(super) assign: Option<AssignKind>,
    pub(super) auto_call: bool,
}

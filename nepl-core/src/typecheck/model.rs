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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CoreIntrinsicKind {
    SizeOf,
    AlignOf,
    Load,
    Store,
    CallsiteSpan,
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CoreIntrinsicResultKind {
    I32,
    Unit,
    Never,
    FirstTypeArgOrUnit,
    FirstTypeArgOrDiagnostic,
}

impl CoreIntrinsicKind {
    pub(super) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "size_of" => Some(Self::SizeOf),
            "align_of" => Some(Self::AlignOf),
            "load" => Some(Self::Load),
            "store" => Some(Self::Store),
            "callsite_span" => Some(Self::CallsiteSpan),
            "unreachable" => Some(Self::Unreachable),
            _ => None,
        }
    }

    pub(super) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::SizeOf => "size_of",
            Self::AlignOf => "align_of",
            Self::Load => "load",
            Self::Store => "store",
            Self::CallsiteSpan => "callsite_span",
            Self::Unreachable => "unreachable",
        }
    }

    pub(super) const fn result_kind(self) -> CoreIntrinsicResultKind {
        match self {
            Self::SizeOf | Self::AlignOf => CoreIntrinsicResultKind::I32,
            Self::Load => CoreIntrinsicResultKind::FirstTypeArgOrUnit,
            Self::Store => CoreIntrinsicResultKind::Unit,
            Self::CallsiteSpan => CoreIntrinsicResultKind::FirstTypeArgOrDiagnostic,
            Self::Unreachable => CoreIntrinsicResultKind::Never,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScalarIntrinsicType {
    I32,
    I64,
    U8,
    U32,
    U64,
    F32,
    Char,
    Str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScalarIntrinsicKind {
    I32ToF32,
    I32ToU8,
    I32ToU32,
    F32ToI32,
    U8ToI32,
    CharToI32,
    I32ToChar,
    U32ToI32,
    I64ToU64,
    U64ToI64,
    ReinterpretI32F32,
    ReinterpretF32I32,
    StrAddr,
    StrFromAddrUnchecked,
}

impl ScalarIntrinsicKind {
    pub(super) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "i32_to_f32" => Some(Self::I32ToF32),
            "i32_to_u8" => Some(Self::I32ToU8),
            "i32_to_u32" => Some(Self::I32ToU32),
            "f32_to_i32" => Some(Self::F32ToI32),
            "u8_to_i32" => Some(Self::U8ToI32),
            "char_to_i32" => Some(Self::CharToI32),
            "i32_to_char" => Some(Self::I32ToChar),
            "u32_to_i32" => Some(Self::U32ToI32),
            "i64_to_u64" => Some(Self::I64ToU64),
            "u64_to_i64" => Some(Self::U64ToI64),
            "reinterpret_i32_f32" => Some(Self::ReinterpretI32F32),
            "reinterpret_f32_i32" => Some(Self::ReinterpretF32I32),
            "str_addr" => Some(Self::StrAddr),
            "str_from_addr_unchecked" => Some(Self::StrFromAddrUnchecked),
            _ => None,
        }
    }

    pub(super) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::I32ToF32 => "i32_to_f32",
            Self::I32ToU8 => "i32_to_u8",
            Self::I32ToU32 => "i32_to_u32",
            Self::F32ToI32 => "f32_to_i32",
            Self::U8ToI32 => "u8_to_i32",
            Self::CharToI32 => "char_to_i32",
            Self::I32ToChar => "i32_to_char",
            Self::U32ToI32 => "u32_to_i32",
            Self::I64ToU64 => "i64_to_u64",
            Self::U64ToI64 => "u64_to_i64",
            Self::ReinterpretI32F32 => "reinterpret_i32_f32",
            Self::ReinterpretF32I32 => "reinterpret_f32_i32",
            Self::StrAddr => "str_addr",
            Self::StrFromAddrUnchecked => "str_from_addr_unchecked",
        }
    }

    pub(super) const fn argument_count(self) -> usize {
        match self {
            Self::I32ToF32
            | Self::I32ToU8
            | Self::I32ToU32
            | Self::F32ToI32
            | Self::U8ToI32
            | Self::CharToI32
            | Self::I32ToChar
            | Self::U32ToI32
            | Self::I64ToU64
            | Self::U64ToI64
            | Self::ReinterpretI32F32
            | Self::ReinterpretF32I32
            | Self::StrAddr
            | Self::StrFromAddrUnchecked => 1,
        }
    }

    pub(super) const fn input_type(self) -> ScalarIntrinsicType {
        match self {
            Self::I32ToF32
            | Self::I32ToU8
            | Self::I32ToU32
            | Self::I32ToChar
            | Self::ReinterpretI32F32 => ScalarIntrinsicType::I32,
            Self::F32ToI32 | Self::ReinterpretF32I32 => ScalarIntrinsicType::F32,
            Self::U8ToI32 => ScalarIntrinsicType::U8,
            Self::CharToI32 => ScalarIntrinsicType::Char,
            Self::U32ToI32 => ScalarIntrinsicType::U32,
            Self::I64ToU64 => ScalarIntrinsicType::I64,
            Self::U64ToI64 => ScalarIntrinsicType::U64,
            Self::StrAddr => ScalarIntrinsicType::Str,
            Self::StrFromAddrUnchecked => ScalarIntrinsicType::I32,
        }
    }

    pub(super) const fn output_type(self) -> ScalarIntrinsicType {
        match self {
            Self::I32ToF32 | Self::ReinterpretI32F32 => ScalarIntrinsicType::F32,
            Self::I32ToU8 => ScalarIntrinsicType::U8,
            Self::I32ToU32 => ScalarIntrinsicType::U32,
            Self::F32ToI32
            | Self::U8ToI32
            | Self::CharToI32
            | Self::U32ToI32
            | Self::ReinterpretF32I32
            | Self::StrAddr => ScalarIntrinsicType::I32,
            Self::I32ToChar => ScalarIntrinsicType::Char,
            Self::I64ToU64 => ScalarIntrinsicType::U64,
            Self::U64ToI64 => ScalarIntrinsicType::I64,
            Self::StrFromAddrUnchecked => ScalarIntrinsicType::Str,
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
    use super::{
        CoreIntrinsicKind, CoreIntrinsicResultKind, FieldAccessorKind, ScalarIntrinsicKind,
        ScalarIntrinsicType,
    };

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

    #[test]
    fn core_intrinsic_result_kinds_round_trip_through_kind() {
        for (kind, result_kind) in [
            (CoreIntrinsicKind::SizeOf, CoreIntrinsicResultKind::I32),
            (CoreIntrinsicKind::AlignOf, CoreIntrinsicResultKind::I32),
            (
                CoreIntrinsicKind::Load,
                CoreIntrinsicResultKind::FirstTypeArgOrUnit,
            ),
            (CoreIntrinsicKind::Store, CoreIntrinsicResultKind::Unit),
            (
                CoreIntrinsicKind::CallsiteSpan,
                CoreIntrinsicResultKind::FirstTypeArgOrDiagnostic,
            ),
            (
                CoreIntrinsicKind::Unreachable,
                CoreIntrinsicResultKind::Never,
            ),
        ] {
            assert_eq!(
                CoreIntrinsicKind::from_intrinsic_name(kind.intrinsic_name()),
                Some(kind)
            );
            assert_eq!(kind.result_kind(), result_kind);
        }
        assert_eq!(CoreIntrinsicKind::from_intrinsic_name("i32_to_f32"), None);
    }

    #[test]
    fn scalar_intrinsic_signatures_round_trip_through_kind() {
        for kind in [
            ScalarIntrinsicKind::I32ToF32,
            ScalarIntrinsicKind::I32ToU8,
            ScalarIntrinsicKind::I32ToU32,
            ScalarIntrinsicKind::F32ToI32,
            ScalarIntrinsicKind::U8ToI32,
            ScalarIntrinsicKind::CharToI32,
            ScalarIntrinsicKind::I32ToChar,
            ScalarIntrinsicKind::U32ToI32,
            ScalarIntrinsicKind::I64ToU64,
            ScalarIntrinsicKind::U64ToI64,
            ScalarIntrinsicKind::ReinterpretI32F32,
            ScalarIntrinsicKind::ReinterpretF32I32,
            ScalarIntrinsicKind::StrAddr,
            ScalarIntrinsicKind::StrFromAddrUnchecked,
        ] {
            assert_eq!(
                ScalarIntrinsicKind::from_intrinsic_name(kind.intrinsic_name()),
                Some(kind)
            );
            assert_eq!(kind.argument_count(), 1);
        }
        assert_eq!(ScalarIntrinsicKind::from_intrinsic_name("i32_add"), None);
    }

    #[test]
    fn scalar_intrinsic_signatures_are_kind_owned() {
        assert_eq!(
            ScalarIntrinsicKind::I32ToF32.input_type(),
            ScalarIntrinsicType::I32
        );
        assert_eq!(
            ScalarIntrinsicKind::I32ToF32.output_type(),
            ScalarIntrinsicType::F32
        );
        assert_eq!(
            ScalarIntrinsicKind::StrFromAddrUnchecked.input_type(),
            ScalarIntrinsicType::I32
        );
        assert_eq!(
            ScalarIntrinsicKind::StrFromAddrUnchecked.output_type(),
            ScalarIntrinsicType::Str
        );
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

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Visibility;
use crate::diagnostic::Diagnostic;
use crate::hir::{HirExpr, HirFunction};
use crate::types::{EnumVariantInfo, TypeId};

use super::struct_shape::StructConstructorShape;

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
    pub(super) constructor_shape: StructConstructorShape,
    pub(super) constructor_policy: StructConstructorPolicy,
}

#[derive(Debug, Clone)]
pub(super) enum FieldIdx {
    Index(usize),
    Name(String),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum AssignKind {
    Let,
    Set,
    AddrOf(bool),
    Deref,
}

#[derive(Debug, Clone)]
pub(super) struct StackEntry {
    pub(super) ty: TypeId,
    pub(super) expr: HirExpr,
    pub(super) type_args: Vec<TypeId>,
    pub(super) assign: Option<AssignKind>,
    pub(super) auto_call: bool,
}

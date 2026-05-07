extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::layout::aggregate_fields_with_offsets;
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::PlaceProjection;
use super::type_pattern::field_type_matches_result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AggregateFieldSelector<'a> {
    Index(usize),
    Name(&'a str),
    Unsupported,
}

pub(super) fn non_negative_i32_literal(expr: &HirExpr) -> Option<usize> {
    match expr.kind {
        HirExprKind::LiteralI32(value) if value >= 0 => Some(value as usize),
        _ => None,
    }
}

pub(super) fn aggregate_field_selector<'a>(
    selector: &HirExpr,
    string_literals: &'a [String],
) -> AggregateFieldSelector<'a> {
    if let Some(index) = non_negative_i32_literal(selector) {
        return AggregateFieldSelector::Index(index);
    }
    match &selector.kind {
        HirExprKind::LiteralStr(index) => string_literals
            .get(*index as usize)
            .map(String::as_str)
            .map(AggregateFieldSelector::Name)
            .unwrap_or(AggregateFieldSelector::Unsupported),
        _ => AggregateFieldSelector::Unsupported,
    }
}

pub(super) fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

pub(super) fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() == 2 => {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if matches!(callee_base_name(callee), Some("add")) && args.len() == 2 =>
        {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

pub(super) fn aggregate_field_projection(
    types: &TypeCtx,
    owner_ty: TypeId,
    offset_bytes: usize,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    let fields = aggregate_fields_with_offsets(types, owner_ty);
    let (index, _) = fields.iter().enumerate().find(|(_, field)| {
        field.offset == offset_bytes && field_type_matches_result(types, field.ty, field_ty)
    })?;
    Some(match kind {
        AggregateProjectionKind::Struct => PlaceProjection::Field {
            index,
            offset_bytes,
        },
        AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
            index,
            offset_bytes,
        },
    })
}

pub(super) fn aggregate_field_projection_by_name(
    types: &TypeCtx,
    owner_ty: TypeId,
    field_name: &str,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    let index = match kind {
        AggregateProjectionKind::Struct => aggregate_struct_field_names(types, owner_ty)?
            .iter()
            .position(|name| name == field_name)?,
        AggregateProjectionKind::Tuple => field_name.parse::<usize>().ok()?,
    };
    aggregate_field_projection_by_index_with_kind(types, owner_ty, field_ty, kind, index)
}

pub(super) fn aggregate_field_projection_by_selector(
    types: &TypeCtx,
    owner_ty: TypeId,
    selector: &HirExpr,
    field_ty: TypeId,
    string_literals: &[String],
) -> Option<PlaceProjection> {
    match aggregate_field_selector(selector, string_literals) {
        AggregateFieldSelector::Index(index) => {
            aggregate_field_projection_by_index(types, owner_ty, index, field_ty)
        }
        AggregateFieldSelector::Name(field_name) => {
            aggregate_field_projection_by_name(types, owner_ty, field_name, field_ty)
        }
        AggregateFieldSelector::Unsupported => None,
    }
}

fn aggregate_field_projection_by_index(
    types: &TypeCtx,
    owner_ty: TypeId,
    index: usize,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    aggregate_field_projection_by_index_with_kind(types, owner_ty, field_ty, kind, index)
}

fn aggregate_field_projection_by_index_with_kind(
    types: &TypeCtx,
    owner_ty: TypeId,
    field_ty: TypeId,
    kind: AggregateProjectionKind,
    index: usize,
) -> Option<PlaceProjection> {
    let fields = aggregate_fields_with_offsets(types, owner_ty);
    let field = fields.get(index)?;
    if !field_type_matches_result(types, field.ty, field_ty) {
        return None;
    }
    Some(match kind {
        AggregateProjectionKind::Struct => PlaceProjection::Field {
            index,
            offset_bytes: field.offset,
        },
        AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
            index,
            offset_bytes: field.offset,
        },
    })
}

fn aggregate_struct_field_names(types: &TypeCtx, ty: TypeId) -> Option<&Vec<String>> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => Some(field_names),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => Some(field_names),
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AggregateProjectionKind {
    Struct,
    Tuple,
}

fn aggregate_projection_kind(types: &TypeCtx, ty: TypeId) -> Option<AggregateProjectionKind> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } => Some(AggregateProjectionKind::Struct),
        TypeKind::Tuple { .. } => Some(AggregateProjectionKind::Tuple),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { .. } => Some(AggregateProjectionKind::Struct),
                TypeKind::Tuple { .. } => Some(AggregateProjectionKind::Tuple),
                _ => None,
            }
        }
        _ => None,
    }
}

fn callee_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::HirExpr;
use crate::layout::aggregate_fields_with_offsets;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::lower_aggregate_selector::{aggregate_field_selector, AggregateFieldSelector};
use super::model::PlaceProjection;
use super::type_pattern::field_type_matches_result;

pub(super) fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
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

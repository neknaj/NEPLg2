//! Shared storage layout queries for compiler checks and backends.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

pub const ENUM_TAG_SIZE_BYTES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldLayout {
    pub offset: usize,
    pub ty: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedFieldLayout {
    pub index: usize,
    pub field: FieldLayout,
}

pub fn enum_payload_offset_bytes() -> usize {
    ENUM_TAG_SIZE_BYTES
}

pub fn mapped_type_id(types: &TypeCtx, ty: TypeId, mapping: &BTreeMap<TypeId, TypeId>) -> TypeId {
    let mut resolved = types.resolve_id(ty);
    let mut depth = 0usize;
    while let Some(mapped) = mapping.get(&resolved).copied() {
        if depth > 5000 {
            break;
        }
        let next = types.resolve_id(mapped);
        if next == resolved {
            break;
        }
        resolved = next;
        depth += 1;
    }
    types.resolve_named_type_id(resolved)
}

pub fn extend_type_mapping(
    types: &TypeCtx,
    parent: &BTreeMap<TypeId, TypeId>,
    type_params: &[TypeId],
    args: &[TypeId],
) -> BTreeMap<TypeId, TypeId> {
    let mut mapping = parent.clone();
    for (param, arg) in type_params.iter().copied().zip(args.iter().copied()) {
        mapping.insert(types.resolve_id(param), mapped_type_id(types, arg, parent));
    }
    mapping
}

pub fn storage_align_bytes(types: &TypeCtx, ty: TypeId) -> usize {
    storage_align_bytes_mapped(types, ty, &BTreeMap::new())
}

pub fn storage_size_bytes(types: &TypeCtx, ty: TypeId) -> usize {
    storage_size_bytes_mapped(types, ty, &BTreeMap::new())
}

pub fn storage_size_bytes_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> usize {
    let ty = mapped_type_id(types, ty, mapping);
    match types.get_ref(ty) {
        TypeKind::Unit | TypeKind::Never => 0,
        TypeKind::U8 => 1,
        TypeKind::Named(name) if name == "i64" || name == "u64" || name == "f64" => 8,
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .map(|field| storage_size_bytes_mapped(types, *field, mapping))
            .sum(),
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| storage_size_bytes_mapped(types, *item, mapping))
            .sum(),
        TypeKind::Enum { variants, .. } => {
            let payload = variants
                .iter()
                .filter_map(|variant| variant.payload)
                .map(|payload| storage_size_bytes_mapped(types, payload, mapping))
                .max()
                .unwrap_or(0);
            ENUM_TAG_SIZE_BYTES + payload
        }
        TypeKind::Apply { base, args } => storage_size_bytes_for_apply(types, *base, args, mapping),
        TypeKind::Var(var) => var
            .binding
            .map(|binding| storage_size_bytes_mapped(types, binding, mapping))
            .unwrap_or(4),
        _ => 4,
    }
}

fn storage_align_bytes_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> usize {
    let ty = mapped_type_id(types, ty, mapping);
    match types.get_ref(ty) {
        TypeKind::Unit | TypeKind::Never => 1,
        TypeKind::U8 => 1,
        TypeKind::Named(name) if name == "i64" || name == "u64" || name == "f64" => 8,
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .map(|field| storage_align_bytes_mapped(types, *field, mapping))
            .max()
            .unwrap_or(1),
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| storage_align_bytes_mapped(types, *item, mapping))
            .max()
            .unwrap_or(1),
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .filter_map(|variant| variant.payload)
            .map(|payload| storage_align_bytes_mapped(types, payload, mapping))
            .max()
            .unwrap_or(ENUM_TAG_SIZE_BYTES)
            .max(ENUM_TAG_SIZE_BYTES),
        TypeKind::Apply { base, args } => {
            storage_align_bytes_for_apply(types, *base, args, mapping)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| storage_align_bytes_mapped(types, binding, mapping))
            .unwrap_or(4),
        _ => 4,
    }
}

fn storage_size_bytes_for_apply(
    types: &TypeCtx,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
) -> usize {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            fields
                .iter()
                .map(|field| storage_size_bytes_mapped(types, *field, &nested_mapping))
                .sum()
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            let payload = variants
                .iter()
                .filter_map(|variant| variant.payload)
                .map(|payload| storage_size_bytes_mapped(types, payload, &nested_mapping))
                .max()
                .unwrap_or(0);
            ENUM_TAG_SIZE_BYTES + payload
        }
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| storage_size_bytes_mapped(types, *item, mapping))
            .sum(),
        _ => 4,
    }
}

fn storage_align_bytes_for_apply(
    types: &TypeCtx,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
) -> usize {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            fields
                .iter()
                .map(|field| storage_align_bytes_mapped(types, *field, &nested_mapping))
                .max()
                .unwrap_or(1)
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            variants
                .iter()
                .filter_map(|variant| variant.payload)
                .map(|payload| storage_align_bytes_mapped(types, payload, &nested_mapping))
                .max()
                .unwrap_or(ENUM_TAG_SIZE_BYTES)
                .max(ENUM_TAG_SIZE_BYTES)
        }
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| storage_align_bytes_mapped(types, *item, mapping))
            .max()
            .unwrap_or(1),
        _ => 4,
    }
}

pub fn is_aggregate_storage_type(types: &TypeCtx, ty: TypeId) -> bool {
    let ty = types.resolve_named_type_id(ty);
    match types.get_ref(ty) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } | TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => matches!(
            types.get_ref(types.resolve_named_type_id(*base)),
            TypeKind::Struct { .. } | TypeKind::Tuple { .. } | TypeKind::Enum { .. }
        ),
        _ => false,
    }
}

pub fn intrinsic_storage_type(types: &TypeCtx, annotated: TypeId, inferred: TypeId) -> TypeId {
    let annotated = types.resolve_id(annotated);
    match types.get_ref(annotated) {
        TypeKind::Var(_) => types.resolve_id(inferred),
        _ => annotated,
    }
}

pub fn composite_field_offset_bytes(types: &TypeCtx, field_tys: &[TypeId], index: usize) -> usize {
    field_tys
        .iter()
        .take(index)
        .map(|ty| storage_size_bytes(types, *ty))
        .sum()
}

pub fn tuple_field_layout(types: &TypeCtx, ty: TypeId, index: usize) -> Option<FieldLayout> {
    aggregate_fields_with_offsets(types, ty).get(index).copied()
}

pub fn tuple_field_layouts_by_result(
    types: &TypeCtx,
    ty: TypeId,
    result_ty: TypeId,
) -> Vec<IndexedFieldLayout> {
    let mut out = Vec::new();
    let want = types.resolve_named_type_id(result_ty);
    for (index, field) in aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .enumerate()
    {
        if types.resolve_named_type_id(field.ty) == want {
            out.push(IndexedFieldLayout { index, field });
        }
    }
    out
}

pub fn struct_field_layout_by_name(
    types: &TypeCtx,
    ty: TypeId,
    field_name: &str,
) -> Option<FieldLayout> {
    fn inner(
        types: &TypeCtx,
        ty: TypeId,
        field_name: &str,
        mapping: &BTreeMap<TypeId, TypeId>,
    ) -> Option<FieldLayout> {
        let ty = mapped_type_id(types, ty, mapping);
        match types.get_ref(ty) {
            TypeKind::Struct {
                type_params: _,
                field_names,
                ..
            } => {
                let index = field_names.iter().position(|name| name == field_name)?;
                aggregate_fields_with_offsets_mapped(types, ty, mapping)
                    .get(index)
                    .copied()
            }
            TypeKind::Apply { base, args } => {
                let base = types.resolve_named_type_id(*base);
                match types.get_ref(base) {
                    TypeKind::Struct {
                        type_params,
                        field_names,
                        ..
                    } => {
                        let index = field_names.iter().position(|name| name == field_name)?;
                        let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                        aggregate_fields_with_offsets_mapped(types, base, &nested_mapping)
                            .get(index)
                            .copied()
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    inner(types, ty, field_name, &BTreeMap::new())
}

pub fn aggregate_fields_with_offsets(types: &TypeCtx, ty: TypeId) -> Vec<FieldLayout> {
    aggregate_fields_with_offsets_mapped(types, ty, &BTreeMap::new())
}

fn aggregate_fields_with_offsets_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> Vec<FieldLayout> {
    let ty = mapped_type_id(types, ty, mapping);
    match types.get_ref(ty) {
        TypeKind::Struct { fields, .. } => fields_with_offsets(types, fields, mapping),
        TypeKind::Tuple { items } => fields_with_offsets(types, items, mapping),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    fields_with_offsets(types, fields, &nested_mapping)
                }
                TypeKind::Tuple { items } => fields_with_offsets(types, items, mapping),
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

fn fields_with_offsets(
    types: &TypeCtx,
    fields: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
) -> Vec<FieldLayout> {
    let mut offset = 0usize;
    let mut out = Vec::with_capacity(fields.len());
    for field in fields {
        let field_ty = mapped_type_id(types, *field, mapping);
        out.push(FieldLayout {
            offset,
            ty: field_ty,
        });
        offset += storage_size_bytes_mapped(types, *field, mapping);
    }
    out
}

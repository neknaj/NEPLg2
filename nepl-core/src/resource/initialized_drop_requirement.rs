extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::layout::aggregate_fields_with_offsets;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::drop_requirement::{ResourceDropField, ResourceDropRequirement};
use super::model::{CellState, Place, PlaceProjection};

pub(super) fn partial_drop_requirement_for_initialized_descendants(
    types: &TypeCtx,
    cells: &CellTable,
    place: &Place,
) -> ResourceDropRequirement {
    partial_drop_requirement_inner(types, cells, place, place.ty, 0, &mut BTreeSet::new())
}

fn partial_drop_requirement_inner(
    types: &TypeCtx,
    cells: &CellTable,
    place: &Place,
    ty: TypeId,
    base_offset: usize,
    visiting: &mut BTreeSet<TypeId>,
) -> ResourceDropRequirement {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    if !visiting.insert(resolved) {
        return ResourceDropRequirement::StateOnly;
    }

    let mut fields = Vec::new();
    let mut dynamic_enum_fields = Vec::new();
    for (index, field) in aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .enumerate()
    {
        let Some(projection) = aggregate_place_projection(types, ty, index, field.offset) else {
            continue;
        };
        let field_place = place
            .clone()
            .with_projection(projection, types.resolve_id(field.ty));
        let offset = base_offset + field.offset;
        if matches!(
            cells.availability_state_with_types(types, &field_place),
            CellState::Initialized(_)
        ) {
            push_drop_requirement_at_offset(
                types,
                field.ty,
                offset,
                &mut fields,
                &mut dynamic_enum_fields,
            );
        } else {
            push_nested_partial_requirement(
                types,
                cells,
                &field_place,
                field.ty,
                offset,
                visiting,
                &mut fields,
                &mut dynamic_enum_fields,
            );
        }
    }

    visiting.remove(&resolved);
    structural_requirement_or_state_only(fields, dynamic_enum_fields)
}

fn push_nested_partial_requirement(
    types: &TypeCtx,
    cells: &CellTable,
    field_place: &Place,
    ty: TypeId,
    offset: usize,
    visiting: &mut BTreeSet<TypeId>,
    fields: &mut Vec<ResourceDropField>,
    dynamic_enum_fields: &mut Vec<ResourceDropField>,
) {
    match partial_drop_requirement_inner(types, cells, field_place, ty, offset, visiting) {
        ResourceDropRequirement::StateOnly => {}
        ResourceDropRequirement::WholeValue => fields.push(ResourceDropField {
            offset,
            ty: types.resolve_id(ty),
        }),
        ResourceDropRequirement::DynamicEnumPayload => {
            dynamic_enum_fields.push(ResourceDropField {
                offset,
                ty: types.resolve_id(ty),
            });
        }
        ResourceDropRequirement::Structural {
            fields: nested_fields,
            dynamic_enum_fields: nested_dynamic,
        } => {
            fields.extend(nested_fields);
            dynamic_enum_fields.extend(nested_dynamic);
        }
    }
}

fn push_drop_requirement_at_offset(
    types: &TypeCtx,
    ty: TypeId,
    offset: usize,
    fields: &mut Vec<ResourceDropField>,
    dynamic_enum_fields: &mut Vec<ResourceDropField>,
) {
    match super::drop_requirement::resource_drop_requirement_for_type(types, ty) {
        ResourceDropRequirement::StateOnly => {}
        ResourceDropRequirement::WholeValue => fields.push(ResourceDropField {
            offset,
            ty: types.resolve_id(ty),
        }),
        ResourceDropRequirement::DynamicEnumPayload => {
            dynamic_enum_fields.push(ResourceDropField {
                offset,
                ty: types.resolve_id(ty),
            });
        }
        ResourceDropRequirement::Structural {
            fields: nested_fields,
            dynamic_enum_fields: nested_dynamic,
        } => {
            fields.extend(offset_drop_fields(nested_fields, offset));
            dynamic_enum_fields.extend(offset_drop_fields(nested_dynamic, offset));
        }
    }
}

fn offset_drop_fields(
    fields: Vec<ResourceDropField>,
    base_offset: usize,
) -> Vec<ResourceDropField> {
    fields
        .into_iter()
        .map(|field| ResourceDropField {
            offset: base_offset + field.offset,
            ty: field.ty,
        })
        .collect()
}

fn structural_requirement_or_state_only(
    fields: Vec<ResourceDropField>,
    dynamic_enum_fields: Vec<ResourceDropField>,
) -> ResourceDropRequirement {
    if fields.is_empty() && dynamic_enum_fields.is_empty() {
        ResourceDropRequirement::StateOnly
    } else {
        ResourceDropRequirement::Structural {
            fields,
            dynamic_enum_fields,
        }
    }
}

fn aggregate_place_projection(
    types: &TypeCtx,
    ty: TypeId,
    index: usize,
    offset_bytes: usize,
) -> Option<PlaceProjection> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } => Some(PlaceProjection::Field {
            index,
            offset_bytes,
        }),
        TypeKind::Tuple { .. } => Some(PlaceProjection::TupleField {
            index,
            offset_bytes,
        }),
        TypeKind::Apply { base, .. } => {
            aggregate_place_projection(types, *base, index, offset_bytes)
        }
        _ => None,
    }
}

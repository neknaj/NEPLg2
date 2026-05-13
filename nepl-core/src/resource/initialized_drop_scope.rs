extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::layout::aggregate_fields_with_offsets;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::drop_model::ResourceAutoDrop;
use super::drop_plan::auto_drop_candidates_for_end_scope;
use super::drop_requirement::{ResourceDropField, ResourceDropRequirement};
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, Place, PlaceProjection};
use super::raw_realloc::PendingRawReallocs;

pub(super) fn auto_drop_scope_locals_with_record(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    locals: &[Place],
    span: Span,
) -> Vec<ResourceAutoDrop> {
    let mut auto_drops = Vec::new();
    for candidate in auto_drop_candidates_for_end_scope(types, locals, span) {
        let local = &candidate.place;
        let requirement = if matches!(
            cells.availability_state_with_types(types, local),
            CellState::Initialized(_)
        ) {
            candidate.requirement.clone()
        } else {
            partial_drop_requirement_for_initialized_descendants(types, cells, local)
        };
        if matches!(requirement, ResourceDropRequirement::StateOnly) {
            continue;
        }
        cells.set_state(local, CellState::Dropped);
        raw_aliases.clear(local);
        function_aliases.clear_alias(local);
        pending_reallocs.clear_result(local);
        variant_initializations.clear_result(local);
        auto_drops.push(ResourceAutoDrop {
            requirement,
            ..candidate
        });
    }
    auto_drops
}

fn partial_drop_requirement_for_initialized_descendants(
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
            match partial_drop_requirement_inner(
                types,
                cells,
                &field_place,
                field.ty,
                offset,
                visiting,
            ) {
                ResourceDropRequirement::StateOnly => {}
                ResourceDropRequirement::WholeValue => fields.push(ResourceDropField {
                    offset,
                    ty: types.resolve_id(field.ty),
                }),
                ResourceDropRequirement::DynamicEnumPayload => {
                    dynamic_enum_fields.push(ResourceDropField {
                        offset,
                        ty: types.resolve_id(field.ty),
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
    }

    visiting.remove(&resolved);
    structural_requirement_or_state_only(fields, dynamic_enum_fields)
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

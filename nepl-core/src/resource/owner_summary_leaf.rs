use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection, RawMemoryOp, ResourceFunction, ResourceOp};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};

pub(super) struct OwnerLeafPlace {
    pub(super) place: Place,
    pub(super) suffix: Vec<PlaceProjection>,
}

pub(super) fn owner_leaf_places(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    owner_leaf_projections(types, base.ty)
        .into_iter()
        .map(|leaf| OwnerLeafPlace {
            place: place_with_suffix(base, &leaf.suffix, leaf.ty),
            suffix: leaf.suffix,
        })
        .collect()
}

pub(super) fn owner_seed_leaf_places(
    types: &TypeCtx,
    function: &ResourceFunction,
    _parameter_index: usize,
    base: &Place,
) -> Vec<OwnerLeafPlace> {
    let mut leaves = owner_leaf_places(types, base);
    if types.resolve_id(base.ty) == types.i32() && function_has_raw_owner_consumption(function) {
        leaves.push(OwnerLeafPlace {
            place: base.clone(),
            suffix: Vec::new(),
        });
    }
    leaves
}

struct OwnerLeafProjection {
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
}

fn owner_leaf_projections(types: &TypeCtx, ty: TypeId) -> Vec<OwnerLeafProjection> {
    owner_leaf_projections_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new())
}

fn owner_leaf_projections_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mapped = mapped_type_id(types, ty, mapping);
    if !seen.insert(mapped) {
        return vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }];
    }
    let out = match types.get_ref(mapped) {
        TypeKind::Unit | TypeKind::Never | TypeKind::Reference(_, _) => Vec::new(),
        TypeKind::Struct { name, .. } if name == "MemPtr" => {
            mem_ptr_owner_leaf_projections(types, mapped, AggregateProjectionKind::Struct)
        }
        TypeKind::Struct { .. } => aggregate_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Struct,
            mapping,
            seen,
        ),
        TypeKind::Tuple { .. } => aggregate_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
        ),
        TypeKind::Enum { variants, .. } => {
            enum_owner_leaf_projections(types, variants, mapping, seen)
        }
        TypeKind::Apply { base, args } => {
            apply_owner_leaf_projections(types, mapped, *base, args, mapping, seen)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| owner_leaf_projections_mapped(types, binding, mapping, seen))
            .unwrap_or_else(|| {
                vec![OwnerLeafProjection {
                    suffix: Vec::new(),
                    ty: mapped,
                }]
            }),
        TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Function { .. } => Vec::new(),
        TypeKind::Str | TypeKind::Named(_) | TypeKind::Box(_) => {
            vec![OwnerLeafProjection {
                suffix: Vec::new(),
                ty: mapped,
            }]
        }
    };
    seen.remove(&mapped);
    out
}

#[derive(Clone, Copy)]
enum AggregateProjectionKind {
    Struct,
    Tuple,
}

fn aggregate_owner_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mut out = Vec::new();
    for (index, field) in aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .enumerate()
    {
        let projection = match kind {
            AggregateProjectionKind::Struct => PlaceProjection::Field {
                index,
                offset_bytes: field.offset,
            },
            AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
                index,
                offset_bytes: field.offset,
            },
        };
        push_nested_owner_leaf_projections(
            &mut out,
            projection,
            owner_leaf_projections_mapped(types, field.ty, mapping, seen),
        );
    }
    out
}

fn mem_ptr_owner_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
) -> Vec<OwnerLeafProjection> {
    aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .next()
        .map(|field| {
            let projection = match kind {
                AggregateProjectionKind::Struct => PlaceProjection::Field {
                    index: 0,
                    offset_bytes: field.offset,
                },
                AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
                    index: 0,
                    offset_bytes: field.offset,
                },
            };
            vec![OwnerLeafProjection {
                suffix: vec![projection],
                ty: field.ty,
            }]
        })
        .unwrap_or_default()
}

fn apply_owner_leaf_projections(
    types: &TypeCtx,
    apply_ty: TypeId,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { name, .. } if name == "MemPtr" => {
            mem_ptr_owner_leaf_projections(types, apply_ty, AggregateProjectionKind::Struct)
        }
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            aggregate_owner_leaf_projections(
                types,
                apply_ty,
                AggregateProjectionKind::Struct,
                &nested_mapping,
                seen,
            )
        }
        TypeKind::Tuple { .. } => aggregate_owner_leaf_projections(
            types,
            apply_ty,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
        ),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            enum_owner_leaf_projections(types, variants, &nested_mapping, seen)
        }
        _ => vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped_type_id(types, base, mapping),
        }],
    }
}

fn function_has_raw_owner_consumption(function: &ResourceFunction) -> bool {
    function.blocks.iter().any(|block| {
        function.params.iter().any(|param| {
            let mut aliases = vec![param.place.clone()];
            ops_use_raw_owner_alias(&block.ops, &mut aliases)
        })
    })
}

fn ops_use_raw_owner_alias(ops: &[ResourceOp], aliases: &mut Vec<Place>) -> bool {
    for op in ops {
        match op {
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::RawAddressAlias {
                source,
                target: output,
                ..
            }
            | ResourceOp::RawAddressView {
                source,
                target: output,
                ..
            } => {
                if place_matches_any_alias(source, aliases) {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Assign { target, value, .. } => {
                if place_matches_any_alias(value, aliases) {
                    push_unique_place(aliases, target);
                }
            }
            ResourceOp::RawMemory {
                operation, args, ..
            } => match operation {
                RawMemoryOp::Dealloc | RawMemoryOp::Realloc => {
                    if args
                        .first()
                        .is_some_and(|arg| place_matches_any_alias(arg, aliases))
                    {
                        return true;
                    }
                }
                _ => {}
            },
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                let mut then_aliases = aliases.clone();
                if ops_use_raw_owner_alias(then_ops, &mut then_aliases) {
                    return true;
                }
                let mut else_aliases = aliases.clone();
                if ops_use_raw_owner_alias(else_ops, &mut else_aliases) {
                    return true;
                }
                if place_matches_any_alias(then_value, &then_aliases)
                    || place_matches_any_alias(else_value, &else_aliases)
                {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut loop_aliases = aliases.clone();
                if ops_use_raw_owner_alias(condition_ops, &mut loop_aliases)
                    || ops_use_raw_owner_alias(body_ops, &mut loop_aliases)
                {
                    return true;
                }
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut output_alias = false;
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    if ops_use_raw_owner_alias(&arm.ops, &mut arm_aliases) {
                        return true;
                    }
                    output_alias |= place_matches_any_alias(&arm.value, &arm_aliases);
                }
                if output_alias {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
    false
}

fn place_matches_any_alias(place: &Place, aliases: &[Place]) -> bool {
    aliases.iter().any(|alias| {
        place == alias
            || place_suffix_after_prefix(place, alias).is_some()
            || place_suffix_after_prefix(alias, place).is_some()
    })
}

fn push_unique_place(places: &mut Vec<Place>, place: &Place) {
    if !places.iter().any(|existing| existing == place) {
        places.push(place.clone());
    }
}

fn enum_owner_leaf_projections(
    types: &TypeCtx,
    variants: &[crate::types::EnumVariantInfo],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mut out = Vec::new();
    for variant in variants {
        let Some(payload) = variant.payload else {
            continue;
        };
        let payload_ty = mapped_type_id(types, payload, mapping);
        let projection = PlaceProjection::EnumPayload {
            variant: variant.name.clone(),
        };
        push_nested_owner_leaf_projections(
            &mut out,
            projection,
            owner_leaf_projections_mapped(types, payload_ty, mapping, seen),
        );
    }
    out
}

fn push_nested_owner_leaf_projections(
    out: &mut Vec<OwnerLeafProjection>,
    projection: PlaceProjection,
    children: Vec<OwnerLeafProjection>,
) {
    for mut child in children {
        child.suffix.insert(0, projection.clone());
        out.push(child);
    }
}

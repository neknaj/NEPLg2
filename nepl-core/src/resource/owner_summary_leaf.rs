use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{
    Place, PlaceProjection, RawMemoryOp, ResourceFunction, ResourceLocal, ResourceOp,
};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};

pub(super) struct OwnerLeafPlace {
    pub(super) place: Place,
    pub(super) suffix: Vec<PlaceProjection>,
}

pub(super) struct OwnerParameterLeafPlace {
    pub(super) parameter_index: usize,
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

pub(super) fn owner_parameter_leaf_places(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> Vec<OwnerParameterLeafPlace> {
    let mut out = Vec::new();
    for (parameter_index, param) in function.params.iter().enumerate() {
        for leaf in owner_leaf_places(types, &param.place) {
            push_unique_parameter_leaf(
                &mut out,
                OwnerParameterLeafPlace {
                    parameter_index,
                    place: leaf.place,
                    suffix: leaf.suffix,
                },
            );
        }
    }
    for raw_source in raw_consumed_parameter_leaf_places(function) {
        push_unique_parameter_leaf(&mut out, raw_source);
    }
    out
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
    if is_mem_ptr_type(types, mapped) {
        return vec![OwnerLeafProjection {
            suffix: vec![PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            ty: types.i32(),
        }];
    }
    if !seen.insert(mapped) {
        return vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }];
    }
    let out = match types.get_ref(mapped) {
        TypeKind::Unit | TypeKind::Never | TypeKind::Reference(_, _) => Vec::new(),
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
        TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Function { .. }
        | TypeKind::Str => Vec::new(),
        TypeKind::Named(_) | TypeKind::Box(_) => {
            vec![OwnerLeafProjection {
                suffix: Vec::new(),
                ty: mapped,
            }]
        }
        TypeKind::I32 => Vec::new(),
    };
    seen.remove(&mapped);
    out
}

fn is_mem_ptr_type(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "MemPtr")
        }
        _ => false,
    }
}

fn raw_consumed_parameter_leaf_places(function: &ResourceFunction) -> Vec<OwnerParameterLeafPlace> {
    let mut out = Vec::new();
    let mut aliases = Vec::new();
    for block in &function.blocks {
        collect_raw_consumed_parameter_leaf_places(
            &mut out,
            &mut aliases,
            &function.params,
            &block.ops,
        );
    }
    out
}

fn collect_raw_consumed_parameter_leaf_places(
    out: &mut Vec<OwnerParameterLeafPlace>,
    aliases: &mut Vec<(Place, Place)>,
    params: &[ResourceLocal],
    ops: &[ResourceOp],
) {
    for op in ops {
        match op {
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::Borrow { source, output, .. } => {
                record_place_alias(aliases, output, &resolve_place_alias(aliases, source));
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            }
            | ResourceOp::Assign {
                target: place,
                value: initializer,
                ..
            } => {
                record_place_alias(aliases, place, &resolve_place_alias(aliases, initializer));
            }
            ResourceOp::RawAddressAlias { source, target, .. }
            | ResourceOp::RawAddressView { source, target, .. } => {
                record_place_alias(aliases, target, &resolve_place_alias(aliases, source));
            }
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc | RawMemoryOp::Realloc,
                args,
                ..
            } => {
                if let Some(source) = args.first() {
                    let source = resolve_place_alias(aliases, source);
                    if source
                        .projections
                        .iter()
                        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
                    {
                        continue;
                    }
                    push_raw_consumed_parameter_source(out, params, &source);
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                let mut then_aliases = aliases.clone();
                collect_raw_consumed_parameter_leaf_places(
                    out,
                    &mut then_aliases,
                    params,
                    then_ops,
                );
                let mut else_aliases = aliases.clone();
                collect_raw_consumed_parameter_leaf_places(
                    out,
                    &mut else_aliases,
                    params,
                    else_ops,
                );
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_aliases = aliases.clone();
                collect_raw_consumed_parameter_leaf_places(
                    out,
                    &mut condition_aliases,
                    params,
                    condition_ops,
                );
                let mut body_aliases = condition_aliases;
                collect_raw_consumed_parameter_leaf_places(
                    out,
                    &mut body_aliases,
                    params,
                    body_ops,
                );
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    collect_raw_consumed_parameter_leaf_places(
                        out,
                        &mut arm_aliases,
                        params,
                        &arm.ops,
                    );
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal {
                initializer: None, ..
            }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

fn record_place_alias(aliases: &mut Vec<(Place, Place)>, target: &Place, source: &Place) {
    aliases.retain(|(existing, _)| existing != target);
    aliases.push((target.clone(), source.clone()));
}

fn resolve_place_alias(aliases: &[(Place, Place)], place: &Place) -> Place {
    let mut current = place.clone();
    for _ in 0..aliases.len() {
        let mut changed = false;
        for (target, source) in aliases {
            let Some(suffix) = place_suffix_after_prefix(&current, target) else {
                continue;
            };
            current = place_with_suffix(source, &suffix, current.ty);
            changed = true;
            break;
        }
        if !changed {
            break;
        }
    }
    current
}

fn push_raw_consumed_parameter_source(
    out: &mut Vec<OwnerParameterLeafPlace>,
    params: &[ResourceLocal],
    source: &Place,
) {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(source, &param.place) else {
            continue;
        };
        push_unique_parameter_leaf(
            out,
            OwnerParameterLeafPlace {
                parameter_index,
                place: source.clone(),
                suffix,
            },
        );
        return;
    }
}

fn push_unique_parameter_leaf(
    out: &mut Vec<OwnerParameterLeafPlace>,
    leaf: OwnerParameterLeafPlace,
) {
    if out.iter().any(|existing| {
        existing.parameter_index == leaf.parameter_index
            && existing.place == leaf.place
            && existing.suffix == leaf.suffix
    }) {
        return;
    }
    out.push(leaf);
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

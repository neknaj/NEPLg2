use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::borrow_check::ResourceBorrowCheckEngine;
use super::borrow_state::BorrowTable;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    OwnerState, Place, PlaceProjection, ResourceFunction, ResourceModule, ResourceTerminator,
    StorageId,
};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_flow::resolve_owner_alias_place;
use super::owner_state::OwnerTable;
use super::place_utils::{place_suffix_after_prefix, push_unique_usize};
use super::report::{ResourceBorrowCheckDeferred, ResourceOwnerCheckDeferred};
use super::storage_origin::StorageOriginTable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowTokenReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) consumed_parameter_indices: Vec<usize>,
    pub(super) consumed_parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) returns_fresh_owner: bool,
    pub(super) projection_returns: Vec<OwnerProjectionReturnSummary>,
    pub(super) projection_markers: Vec<OwnerProjectionMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionSource {
    pub(super) parameter_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionMarker {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionReturnSummary {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) returns_fresh_owner: bool,
}

pub(super) fn compute_borrow_token_return_summaries(
    module: &ResourceModule,
) -> Vec<BorrowTokenReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if function_returns_borrow_token(function, &param.place, &summaries) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(BorrowTokenReturnSummary {
                    function: function.name.clone(),
                    parameter_indices,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_returns_borrow_token(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &[BorrowTokenReturnSummary],
) -> bool {
    let mut engine = ResourceBorrowCheckEngine {
        function: function.name.as_str(),
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceBorrowCheckDeferred::default(),
    };
    let mut borrows = BorrowTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    borrows.add_shared(parameter, parameter);
    for block in &function.blocks {
        engine.check_ops(&mut borrows, &mut function_aliases, &block.ops);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if borrows
                .binding(value)
                .is_some_and(|binding| binding.source == *parameter)
            {
                return true;
            }
        }
    }
    false
}

pub(super) fn compute_owner_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<OwnerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_owner_return_summary(function, types, &summaries);
            if summary.returns_fresh_owner
                || !summary.parameter_indices.is_empty()
                || !summary.parameter_sources.is_empty()
                || !summary.consumed_parameter_indices.is_empty()
                || !summary.consumed_parameter_sources.is_empty()
                || !summary.projection_returns.is_empty()
                || !summary.projection_markers.is_empty()
            {
                next.push(summary);
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_owner_return_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    summaries: &[OwnerReturnSummary],
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut storage_origins = StorageOriginTable::default();
    let mut parameter_storage_sources = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        for leaf in owner_leaf_places(types, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
            if let Some(OwnerState::Live { storage }) = owners.state(&leaf.place) {
                parameter_storage_sources.push(OwnerParameterStorageSource {
                    storage,
                    source: OwnerProjectionSource {
                        parameter_index: index,
                        suffix: leaf.suffix,
                        ty: leaf.place.ty,
                    },
                    place: leaf.place,
                });
            }
        }
    }

    let mut parameter_indices = Vec::new();
    let mut parameter_sources = Vec::new();
    let mut returns_fresh_owner = false;
    let mut projection_returns = Vec::new();
    let mut projection_markers = Vec::new();
    let mut returned_sources = Vec::new();
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut storage_origins,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            let resolved_value = resolve_owner_alias_place(&owners, &raw_aliases, value);
            match owners.state(&resolved_value) {
                Some(OwnerState::Live { storage }) => {
                    if let Some(source) =
                        owner_source_for_storage(storage, &parameter_storage_sources)
                    {
                        record_root_owner_return(
                            &mut parameter_indices,
                            &mut parameter_sources,
                            &mut returned_sources,
                            source,
                        );
                    } else {
                        returns_fresh_owner = true;
                    }
                }
                Some(OwnerState::MaybeFreed) => {}
                Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
                | None => {}
            }
            for entry in owners.descendant_entries(&resolved_value) {
                if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
                    match entry.state {
                        OwnerState::Live { storage } => {
                            record_projection_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                                storage,
                                &parameter_storage_sources,
                                &mut returned_sources,
                            );
                        }
                        OwnerState::NoFreeObligation => {
                            record_projection_marker(
                                &mut projection_markers,
                                suffix,
                                entry.place.ty,
                            );
                        }
                        OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {}
                    }
                }
            }
        }
    }

    let (consumed_parameter_indices, consumed_parameter_sources) =
        consumed_owner_parameters(&owners, &parameter_storage_sources, &returned_sources);
    OwnerReturnSummary {
        function: function.name.clone(),
        parameter_indices,
        parameter_sources,
        consumed_parameter_indices,
        consumed_parameter_sources,
        returns_fresh_owner,
        projection_returns,
        projection_markers,
    }
}

fn consumed_owner_parameters(
    owners: &OwnerTable,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &[OwnerProjectionSource],
) -> (Vec<usize>, Vec<OwnerProjectionSource>) {
    let mut indices = Vec::new();
    let mut sources = Vec::new();
    for entry in parameter_storage_sources {
        let source = &entry.source;
        if returned_sources.iter().any(|returned| returned == source) {
            continue;
        }
        match owners.state(&entry.place) {
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(OwnerState::Live { .. } | OwnerState::NoFreeObligation) | None => {}
        }
    }
    (indices, sources)
}

fn record_projection_owner_return(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    storage: StorageId,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &mut Vec<OwnerProjectionSource>,
) {
    let entry_index = projection_returns
        .iter()
        .position(|entry| entry.suffix == suffix && entry.ty == ty)
        .unwrap_or_else(|| {
            projection_returns.push(OwnerProjectionReturnSummary {
                suffix: suffix.clone(),
                ty,
                parameter_indices: Vec::new(),
                parameter_sources: Vec::new(),
                returns_fresh_owner: false,
            });
            projection_returns.len() - 1
        });
    if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
        record_projection_owner_source(
            &mut projection_returns[entry_index],
            returned_sources,
            source,
        );
    } else {
        projection_returns[entry_index].returns_fresh_owner = true;
    }
}

fn record_projection_marker(
    projection_markers: &mut Vec<OwnerProjectionMarker>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !projection_markers
        .iter()
        .any(|marker| marker.suffix == suffix && marker.ty == ty)
    {
        projection_markers.push(OwnerProjectionMarker { suffix, ty });
    }
}

fn record_root_owner_return(
    parameter_indices: &mut Vec<usize>,
    parameter_sources: &mut Vec<OwnerProjectionSource>,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        push_unique_usize(parameter_indices, source.parameter_index);
    } else {
        push_unique_owner_projection_source(parameter_sources, source);
    }
    push_unique_owner_projection_source(returned_sources, source);
}

fn record_projection_owner_source(
    summary: &mut OwnerProjectionReturnSummary,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        push_unique_usize(&mut summary.parameter_indices, source.parameter_index);
    } else {
        push_unique_owner_projection_source(&mut summary.parameter_sources, source);
    }
    push_unique_owner_projection_source(returned_sources, source);
}

fn owner_source_for_storage<'a>(
    storage: StorageId,
    parameter_storage_sources: &'a [OwnerParameterStorageSource],
) -> Option<&'a OwnerProjectionSource> {
    parameter_storage_sources
        .iter()
        .find_map(|source| (source.storage == storage).then_some(&source.source))
}

fn push_unique_owner_projection_source(
    sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if !sources.iter().any(|existing| existing == source) {
        sources.push(source.clone());
    }
}

struct OwnerParameterStorageSource {
    storage: StorageId,
    source: OwnerProjectionSource,
    place: Place,
}

struct OwnerLeafPlace {
    place: Place,
    suffix: Vec<PlaceProjection>,
}

fn owner_leaf_places(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    owner_leaf_projections(types, base.ty)
        .into_iter()
        .map(|leaf| OwnerLeafPlace {
            place: super::place_utils::place_with_suffix(base, &leaf.suffix, leaf.ty),
            suffix: leaf.suffix,
        })
        .collect()
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
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Function { .. }
        | TypeKind::Box(_) => vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }],
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

extern crate alloc;

use alloc::vec::Vec;

use crate::layout::storage_size_bytes;

use super::collection_slot_state_alias::storage_aliases_for_place;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_projection::{
    compose_translated_summary_suffix_for_params, instantiate_summary_suffix_on_base,
    summary_suffix_for_params,
};
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnRange, CollectionSlotLifecycleReturnRangeCount,
};
use super::collection_slot_summary_return_unique::push_return_range;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceLocal};
use super::owner_extent_summary::instantiate_summary_type;
use super::place_utils::place_suffix_after_prefix;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_return_ranges(
        &mut self,
        collection_slots: &mut super::collection_slot_state_table::CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        output: &Place,
        summary_type_params: &[crate::types::TypeId],
        type_args: &[crate::types::TypeId],
        ranges: &[CollectionSlotLifecycleReturnRange],
    ) {
        for range in ranges {
            let value_ty = instantiate_summary_type(summary_type_params, type_args, range.value_ty);
            if storage_size_bytes(self.types, value_ty) != range.element_stride {
                continue;
            }
            let storage_ty =
                instantiate_summary_type(summary_type_params, type_args, range.storage_ty);
            let Some(storage) = instantiate_summary_suffix_on_base(
                self,
                args,
                output,
                &range.storage_suffix,
                storage_ty,
            ) else {
                continue;
            };
            let Some(initialized_count) = instantiate_return_range_initialized_count(
                self,
                args,
                output,
                summary_type_params,
                type_args,
                range,
            ) else {
                continue;
            };
            collection_slots.mark_initialized_range_with_aliases(
                &storage,
                &initialized_count,
                value_ty,
                range.element_stride,
                raw_aliases,
            );
        }
    }
}

fn instantiate_return_range_initialized_count(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    output: &Place,
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    range: &CollectionSlotLifecycleReturnRange,
) -> Option<Place> {
    match &range.initialized_count {
        CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection { suffix, ty } => {
            let ty = instantiate_summary_type(summary_type_params, type_args, *ty);
            instantiate_summary_suffix_on_base(engine, args, output, suffix, ty)
        }
        CollectionSlotLifecycleReturnRangeCount::KnownI32 { value, ty } => {
            let ty = instantiate_summary_type(summary_type_params, type_args, *ty);
            Some(Place::i32_constant(*value, ty))
        }
    }
}

pub(super) fn translate_return_ranges(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    ranges: &[CollectionSlotLifecycleReturnRange],
    target_suffix: &[PlaceProjection],
) -> Vec<CollectionSlotLifecycleReturnRange> {
    let mut out = Vec::new();
    for range in ranges {
        let Some(storage_suffix) = compose_translated_summary_suffix_for_params(
            engine,
            args,
            params,
            target_suffix,
            &range.storage_suffix,
        ) else {
            continue;
        };
        let storage_ty = instantiate_summary_type(summary_type_params, type_args, range.storage_ty);
        let value_ty = instantiate_summary_type(summary_type_params, type_args, range.value_ty);
        let Some(initialized_count) = translate_return_range_initialized_count(
            engine,
            args,
            params,
            summary_type_params,
            type_args,
            target_suffix,
            &range.initialized_count,
        ) else {
            continue;
        };
        push_return_range(
            &mut out,
            CollectionSlotLifecycleReturnRange {
                storage_suffix,
                storage_ty,
                initialized_count,
                value_ty,
                element_stride: range.element_stride,
            },
        );
    }
    out
}

fn translate_return_range_initialized_count(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    target_suffix: &[PlaceProjection],
    count: &CollectionSlotLifecycleReturnRangeCount,
) -> Option<CollectionSlotLifecycleReturnRangeCount> {
    match count {
        CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection { suffix, ty } => Some(
            CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection {
                suffix: compose_translated_summary_suffix_for_params(
                    engine,
                    args,
                    params,
                    target_suffix,
                    suffix,
                )?,
                ty: instantiate_summary_type(summary_type_params, type_args, *ty),
            },
        ),
        CollectionSlotLifecycleReturnRangeCount::KnownI32 { value, ty } => {
            Some(CollectionSlotLifecycleReturnRangeCount::KnownI32 {
                value: *value,
                ty: instantiate_summary_type(summary_type_params, type_args, *ty),
            })
        }
    }
}

pub(super) fn collect_return_ranges_for_value(
    out: &mut Vec<CollectionSlotLifecycleReturnRange>,
    params: &[ResourceLocal],
    state: &CollectionSlotSummaryBuildState,
    value: &Place,
    target_suffix: &[PlaceProjection],
) {
    for range in state.collection_slots.initialized_ranges() {
        let storage_suffixes = collect_return_range_storage_suffixes(
            params,
            state,
            &range.storage,
            value,
            target_suffix,
        );
        if storage_suffixes.is_empty() {
            continue;
        }
        let count_sources = collect_return_range_count_sources(
            params,
            state,
            &range.initialized_count,
            value,
            target_suffix,
        );
        for (storage_suffix, storage_ty) in &storage_suffixes {
            for initialized_count in &count_sources {
                push_return_range(
                    out,
                    CollectionSlotLifecycleReturnRange {
                        storage_suffix: storage_suffix.clone(),
                        storage_ty: *storage_ty,
                        initialized_count: initialized_count.clone(),
                        value_ty: range.value_ty,
                        element_stride: range.element_stride,
                    },
                );
            }
        }
    }
}

fn collect_return_range_storage_suffixes(
    params: &[ResourceLocal],
    state: &CollectionSlotSummaryBuildState,
    storage: &Place,
    value: &Place,
    target_suffix: &[PlaceProjection],
) -> Vec<(
    Vec<super::summary_projection::SummaryProjection>,
    crate::types::TypeId,
)> {
    let mut out = Vec::new();
    let return_aliases = state.raw_aliases.aliases_for(value);
    for storage_alias in storage_aliases_for_place(storage, &state.raw_aliases) {
        for return_alias in &return_aliases {
            let Some(suffix) = place_suffix_after_prefix(&storage_alias, return_alias) else {
                continue;
            };
            let Some(suffix) = composed_summary_suffix(params, target_suffix, &suffix) else {
                continue;
            };
            push_unique_suffix(&mut out, suffix, storage_alias.ty);
        }
    }
    out
}

fn collect_return_range_count_sources(
    params: &[ResourceLocal],
    state: &CollectionSlotSummaryBuildState,
    count: &Place,
    value: &Place,
    target_suffix: &[PlaceProjection],
) -> Vec<CollectionSlotLifecycleReturnRangeCount> {
    let count = state.raw_aliases.canonicalize_scalar(count);
    let mut out = Vec::new();
    if let Some(value) = state.raw_aliases.i32_value(&count) {
        push_unique_count(
            &mut out,
            CollectionSlotLifecycleReturnRangeCount::KnownI32 {
                value,
                ty: count.ty,
            },
        );
    }
    let return_aliases = state.raw_aliases.aliases_for(value);
    for count_alias in state.raw_aliases.scalar_aliases_for(&count) {
        for return_alias in &return_aliases {
            let Some(suffix) = place_suffix_after_prefix(&count_alias, return_alias) else {
                continue;
            };
            let Some(suffix) = composed_summary_suffix(params, target_suffix, &suffix) else {
                continue;
            };
            push_unique_count(
                &mut out,
                CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection {
                    suffix,
                    ty: count_alias.ty,
                },
            );
        }
    }
    out
}

fn composed_summary_suffix(
    params: &[ResourceLocal],
    target_suffix: &[PlaceProjection],
    suffix: &[PlaceProjection],
) -> Option<Vec<super::summary_projection::SummaryProjection>> {
    let mut composed = target_suffix.to_vec();
    composed.extend_from_slice(suffix);
    summary_suffix_for_params(params, &composed)
}

fn push_unique_suffix(
    out: &mut Vec<(
        Vec<super::summary_projection::SummaryProjection>,
        crate::types::TypeId,
    )>,
    suffix: Vec<super::summary_projection::SummaryProjection>,
    ty: crate::types::TypeId,
) {
    if !out
        .iter()
        .any(|(existing_suffix, existing_ty)| existing_suffix == &suffix && *existing_ty == ty)
    {
        out.push((suffix, ty));
    }
}

fn push_unique_count(
    out: &mut Vec<CollectionSlotLifecycleReturnRangeCount>,
    count: CollectionSlotLifecycleReturnRangeCount,
) {
    if !out.iter().any(|existing| existing == &count) {
        out.push(count);
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec, vec::Vec};

    use crate::span::Span;
    use crate::types::{TypeCtx, TypeId, TypeKind};

    use super::*;
    use crate::resource::cell_state::CellTable;
    use crate::resource::collection_slot_lifecycle::CollectionSlotState;
    use crate::resource::collection_slot_state_table::CollectionSlotStateTable;
    use crate::resource::collection_slot_summary_model::{
        CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleReturnPath,
    };
    use crate::resource::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
    use crate::resource::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
    use crate::resource::initialized_summary::RawCellInitializationFunctionSummaryIndex;
    use crate::resource::initialized_variant::PendingVariantRawCellInitializations;
    use crate::resource::model::{
        Place, PlaceProjection, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceOffset,
    };
    use crate::resource::report::ResourceCheckDeferred;

    #[test]
    fn return_range_apply_restores_initialized_output_prefix() {
        let (types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let output = Place::local("returned_storage".to_string(), i32_ty);
        let slot0 = storage_slot(output.clone(), 0, i32_ty, owned_ty);
        let slot1 = storage_slot(output.clone(), 4, i32_ty, owned_ty);
        let raw_alias_summaries = [];
        let i32_scalar_summaries = [];
        let raw_init_summaries = [];
        let collection_slot_summaries = [];
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let mut engine = summary_test_engine(
            &types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_slot_summary_index,
        );
        let mut collection_slots = CollectionSlotStateTable::new();
        let raw_aliases = RawCellAddressAliases::default();

        engine.apply_collection_slot_return_ranges(
            &mut collection_slots,
            &raw_aliases,
            &[],
            &output,
            &[],
            &[],
            &[CollectionSlotLifecycleReturnRange {
                storage_suffix: Vec::new(),
                storage_ty: i32_ty,
                initialized_count: CollectionSlotLifecycleReturnRangeCount::KnownI32 {
                    value: 1,
                    ty: i32_ty,
                },
                value_ty: owned_ty,
                element_stride: 4,
            }],
        );

        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
            CollectionSlotState::Initialized(owned_ty)
        );
        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot1, &raw_aliases),
            CollectionSlotState::Uninitialized
        );
        assert!(
            collection_slots
                .release_storage_with_aliases(&output, &raw_aliases)
                .is_err(),
            "returned initialized range must block storage release until traversal cleanup"
        );
    }

    #[test]
    fn return_range_collects_storage_and_count_projection_under_return_value() {
        let (mut types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let return_ty = types.register_named(
            "VecLikeReturn".to_string(),
            TypeKind::Struct {
                name: "VecLikeReturn".to_string(),
                type_params: vec![],
                fields: vec![i32_ty, i32_ty],
                field_names: vec!["storage".to_string(), "len".to_string()],
            },
        );
        let value = Place::local("value".to_string(), return_ty);
        let storage = value.clone().with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            i32_ty,
        );
        let count = value.clone().with_projection(
            PlaceProjection::Field {
                index: 1,
                offset_bytes: 4,
            },
            i32_ty,
        );
        let function = empty_summary_function(return_ty);
        let mut state = CollectionSlotSummaryBuildState::new(&types, &function);
        state.collection_slots.mark_initialized_range_with_aliases(
            &storage,
            &count,
            owned_ty,
            4,
            &state.raw_aliases,
        );
        let mut ranges = Vec::new();

        collect_return_ranges_for_value(&mut ranges, &function.params, &state, &value, &[]);

        assert_eq!(ranges.len(), 1, "{ranges:#?}");
        assert_eq!(ranges[0].storage_ty, i32_ty);
        assert_eq!(ranges[0].value_ty, owned_ty);
        assert_eq!(ranges[0].element_stride, 4);
        assert!(matches!(
            &ranges[0].initialized_count,
            CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection { ty, .. }
                if *ty == i32_ty
        ));
    }

    #[test]
    fn return_range_apply_uses_return_value_count_projection() {
        let (mut types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let return_ty = types.register_named(
            "VecLikeApplyReturn".to_string(),
            TypeKind::Struct {
                name: "VecLikeApplyReturn".to_string(),
                type_params: vec![],
                fields: vec![i32_ty, i32_ty],
                field_names: vec!["storage".to_string(), "len".to_string()],
            },
        );
        let output = Place::local("returned".to_string(), return_ty);
        let storage = output.clone().with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            i32_ty,
        );
        let count = output.clone().with_projection(
            PlaceProjection::Field {
                index: 1,
                offset_bytes: 4,
            },
            i32_ty,
        );
        let slot0 = storage_slot(storage.clone(), 0, i32_ty, owned_ty);
        let slot1 = storage_slot(storage.clone(), 4, i32_ty, owned_ty);
        let raw_alias_summaries = [];
        let i32_scalar_summaries = [];
        let raw_init_summaries = [];
        let collection_slot_summaries = [];
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let mut engine = summary_test_engine(
            &types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_slot_summary_index,
        );
        let mut collection_slots = CollectionSlotStateTable::new();
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.set_i32_value(&count, 1);

        engine.apply_collection_slot_return_ranges(
            &mut collection_slots,
            &raw_aliases,
            &[],
            &output,
            &[],
            &[],
            &[CollectionSlotLifecycleReturnRange {
                storage_suffix: vec![super::super::summary_projection::SummaryProjection::Field {
                    index: 0,
                    offset_bytes: 0,
                }],
                storage_ty: i32_ty,
                initialized_count: CollectionSlotLifecycleReturnRangeCount::ReturnValueProjection {
                    suffix: vec![super::super::summary_projection::SummaryProjection::Field {
                        index: 1,
                        offset_bytes: 4,
                    }],
                    ty: i32_ty,
                },
                value_ty: owned_ty,
                element_stride: 4,
            }],
        );

        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
            CollectionSlotState::Initialized(owned_ty)
        );
        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot1, &raw_aliases),
            CollectionSlotState::Uninitialized
        );
    }

    #[test]
    fn return_path_merge_keeps_range_only_path_as_maybe_live() {
        let (types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let output = Place::local("returned_storage".to_string(), i32_ty);
        let slot0 = storage_slot(output.clone(), 0, i32_ty, owned_ty);
        let raw_alias_summaries = [];
        let i32_scalar_summaries = [];
        let raw_init_summaries = [];
        let collection_slot_summaries = [];
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let mut engine = summary_test_engine(
            &types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_slot_summary_index,
        );
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        let mut raw_aliases = RawCellAddressAliases::default();
        let initial_cells = cells.clone();
        let initial_collection_slots = collection_slots.clone();
        let initial_raw_aliases = raw_aliases.clone();
        let initial_variant_initializations = PendingVariantRawCellInitializations::default();
        let mut variant_initializations = initial_variant_initializations.clone();

        engine.apply_collection_slot_return_paths(
            &mut cells,
            &mut collection_slots,
            &mut raw_aliases,
            &initial_cells,
            &initial_collection_slots,
            &initial_raw_aliases,
            &initial_variant_initializations,
            &mut variant_initializations,
            &output,
            &[],
            &[],
            &[],
            &[
                CollectionSlotLifecycleReturnPath {
                    return_variant: None,
                    preconditions: Vec::new(),
                    ops: Vec::new(),
                    return_transfers: Vec::new(),
                    return_slots: Vec::new(),
                    return_ranges: vec![CollectionSlotLifecycleReturnRange {
                        storage_suffix: Vec::new(),
                        storage_ty: i32_ty,
                        initialized_count: CollectionSlotLifecycleReturnRangeCount::KnownI32 {
                            value: 1,
                            ty: i32_ty,
                        },
                        value_ty: owned_ty,
                        element_stride: 4,
                    }],
                    i32_scalar_facts: Default::default(),
                },
                CollectionSlotLifecycleReturnPath {
                    return_variant: None,
                    preconditions: Vec::new(),
                    ops: Vec::new(),
                    return_transfers: Vec::new(),
                    return_slots: Vec::new(),
                    return_ranges: Vec::new(),
                    i32_scalar_facts: Default::default(),
                },
            ],
            Span::dummy(),
        );

        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
            CollectionSlotState::MaybeInitialized(Some(owned_ty))
        );
        assert!(
            collection_slots
                .release_storage_with_aliases(&output, &raw_aliases)
                .is_err(),
            "range-only return path must not disappear during return path merge"
        );
    }

    #[test]
    fn return_range_apply_rejects_generic_stride_mismatch_after_instantiation() {
        let (mut types, _) = types_with_owned_payload();
        let i32_ty = types.i32();
        let payload_param = types.fresh_var(Some("T".to_string()));
        let wide_payload_ty = types.register_named(
            "WideOwnedForReturnRange".to_string(),
            TypeKind::Struct {
                name: "WideOwnedForReturnRange".to_string(),
                type_params: vec![],
                fields: vec![i32_ty, i32_ty],
                field_names: vec!["left".to_string(), "right".to_string()],
            },
        );
        types.register_drop_impl_target(wide_payload_ty);
        let output = Place::local("returned_storage".to_string(), i32_ty);
        let slot0 = storage_slot(output.clone(), 0, i32_ty, wide_payload_ty);
        let raw_alias_summaries = [];
        let i32_scalar_summaries = [];
        let raw_init_summaries = [];
        let collection_slot_summaries = [];
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let mut engine = summary_test_engine(
            &types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_slot_summary_index,
        );
        let mut collection_slots = CollectionSlotStateTable::new();
        let raw_aliases = RawCellAddressAliases::default();

        engine.apply_collection_slot_return_ranges(
            &mut collection_slots,
            &raw_aliases,
            &[],
            &output,
            &[payload_param],
            &[wide_payload_ty],
            &[CollectionSlotLifecycleReturnRange {
                storage_suffix: Vec::new(),
                storage_ty: i32_ty,
                initialized_count: CollectionSlotLifecycleReturnRangeCount::KnownI32 {
                    value: 1,
                    ty: i32_ty,
                },
                value_ty: payload_param,
                element_stride: 4,
            }],
        );

        assert_eq!(
            collection_slots.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
            CollectionSlotState::Uninitialized
        );
        assert!(
            collection_slots
                .release_storage_with_aliases(&output, &raw_aliases)
                .is_ok(),
            "stale generic stride must not install a returned initialized range"
        );
    }

    fn storage_slot(storage: Place, offset: usize, i32_ty: TypeId, owned_ty: TypeId) -> Place {
        storage
            .with_projection(
                PlaceProjection::StorageOffset(ResourceOffset::Known(offset)),
                i32_ty,
            )
            .with_projection(PlaceProjection::Deref, owned_ty)
    }

    fn types_with_owned_payload() -> (TypeCtx, TypeId) {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let i32_ty = types.i32();
        let owned_ty = types.register_named(
            "OwnedForReturnRange".to_string(),
            TypeKind::Struct {
                name: "OwnedForReturnRange".to_string(),
                type_params: vec![],
                fields: vec![i32_ty],
                field_names: vec!["value".to_string()],
            },
        );
        types.register_drop_impl_target(owned_ty);
        (types, owned_ty)
    }

    fn empty_summary_function(result: TypeId) -> ResourceFunction {
        ResourceFunction {
            name: "return_range_fixture".to_string(),
            origin_name: "return_range_fixture".to_string(),
            type_params: Vec::new(),
            params: Vec::<ResourceLocal>::new(),
            result,
            effect: crate::ast::Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: Vec::new(),
            span: Span::dummy(),
        }
    }

    fn summary_test_engine<'a>(
        types: &'a TypeCtx,
        raw_alias_summaries: &'a RawCellAddressReturnSummaryIndex<'a>,
        i32_scalar_summaries: &'a I32ScalarReturnSummaryIndex<'a>,
        raw_init_summaries: &'a RawCellInitializationFunctionSummaryIndex<'a>,
        collection_slot_summaries: &'a CollectionSlotLifecycleFunctionSummaryIndex<'a>,
    ) -> ResourceCheckEngine<'a> {
        ResourceCheckEngine {
            function: "return_range_caller",
            types,
            raw_alias_summaries,
            i32_scalar_summaries,
            raw_init_summaries,
            collection_slot_summaries,
            transform_range_certificates: None,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
            path_alternatives: Default::default(),
        }
    }
}

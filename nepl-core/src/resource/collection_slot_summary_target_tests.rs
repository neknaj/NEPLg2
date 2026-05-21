extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_projection::{
    instantiate_summary_suffix_on_base, summary_suffix_for_params,
    translate_summary_suffix_for_params, CollectionSlotLifecycleSummaryOffset,
    CollectionSlotLifecycleSummaryProjection,
};
use super::collection_slot_summary_target::{instantiate_summary_target, summary_place_for_params};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{Place, PlaceProjection, ResourceLocal, ResourceOffset};
use super::report::ResourceCheckDeferred;

fn local(name: &str, ty: crate::types::TypeId) -> Place {
    Place::local(name.to_string(), ty)
}

fn param(name: &str, place: Place) -> ResourceLocal {
    ResourceLocal {
        name: name.to_string(),
        ty: place.ty,
        mutable: false,
        place,
    }
}

#[test]
fn summary_target_substitutes_scaled_symbolic_operand_with_caller_argument() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let callee_storage = local("callee_storage", i32_ty);
    let callee_index = local("callee_i", i32_ty);
    let target = callee_storage
        .clone()
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                place: Box::new(callee_index.clone()),
                scale: 4,
            }),
            i32_ty,
        )
        .with_projection(PlaceProjection::Deref, i32_ty);
    let params = [param("storage", callee_storage), param("i", callee_index)];

    let summary = summary_place_for_params(&params, &target).expect("summary target");
    assert_scaled_operand_points_to_param(&summary, 1);

    let caller_storage = local("caller_storage", i32_ty);
    let caller_index = local("caller_i", i32_ty);
    let actual = with_engine(&types, |engine| {
        instantiate_summary_target(
            engine,
            &[caller_storage.clone(), caller_index.clone()],
            &summary,
        )
    })
    .expect("instantiated summary target");
    let expected = caller_storage
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                place: Box::new(caller_index),
                scale: 4,
            }),
            i32_ty,
        )
        .with_projection(PlaceProjection::Deref, i32_ty);
    assert_eq!(actual, expected);
}

#[test]
fn summary_target_rejects_non_parameter_symbolic_operand() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let callee_storage = local("callee_storage", i32_ty);
    let callee_local_index = local("local_i", i32_ty);
    let target = callee_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(callee_local_index),
            scale: 4,
        }),
        i32_ty,
    );
    let params = [param("storage", callee_storage)];

    assert!(summary_place_for_params(&params, &target).is_none());
}

#[test]
fn return_suffix_translation_rewrites_symbolic_operand_through_wrapper_args() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let callee_index = local("callee_i", i32_ty);
    let callee_params = [param("i", callee_index.clone())];
    let callee_suffix = summary_suffix_for_params(
        &callee_params,
        &[PlaceProjection::StorageOffset(
            ResourceOffset::ScaledSymbolic {
                place: Box::new(callee_index),
                scale: 4,
            },
        )],
    )
    .expect("callee suffix");

    let wrapper_index = local("wrapper_i", i32_ty);
    let wrapper_params = [param("i", wrapper_index.clone())];
    let translated = with_engine(&types, |engine| {
        translate_summary_suffix_for_params(
            engine,
            &[wrapper_index],
            &wrapper_params,
            &callee_suffix,
        )
    })
    .expect("translated suffix");

    let caller_index = local("caller_i", i32_ty);
    let output = local("output_storage", i32_ty);
    let actual = with_engine(&types, |engine| {
        instantiate_summary_suffix_on_base(
            engine,
            &[caller_index.clone()],
            &output,
            &translated,
            i32_ty,
        )
    })
    .expect("instantiated return suffix");
    let expected = output.with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(caller_index),
            scale: 4,
        }),
        i32_ty,
    );
    assert_eq!(actual, expected);
}

fn assert_scaled_operand_points_to_param(
    summary: &CollectionSlotLifecycleSummaryPlace,
    expected_parameter_index: usize,
) {
    let Some(CollectionSlotLifecycleSummaryProjection::StorageOffset(
        CollectionSlotLifecycleSummaryOffset::ScaledSymbolic { place, scale },
    )) = summary.suffix.first()
    else {
        panic!(
            "summary must keep scaled symbolic offset as typed summary projection: {summary:#?}"
        );
    };
    assert_eq!(*scale, 4);
    assert_eq!(place.parameter_index, expected_parameter_index);
    assert!(place.suffix.is_empty());
}

fn with_engine<R>(types: &TypeCtx, f: impl FnOnce(&ResourceCheckEngine<'_>) -> R) -> R {
    let raw_alias_summaries = [];
    let i32_scalar_summaries = [];
    let raw_init_summaries = [];
    let collection_slot_summaries = [];
    let raw_alias_summaries = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    let i32_scalar_summaries = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    let raw_init_summaries = RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    let collection_slot_summaries =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    let engine = ResourceCheckEngine {
        function: "summary_target_test",
        types,
        raw_alias_summaries: &raw_alias_summaries,
        i32_scalar_summaries: &i32_scalar_summaries,
        raw_init_summaries: &raw_init_summaries,
        collection_slot_summaries: &collection_slot_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    f(&engine)
}

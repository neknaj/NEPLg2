extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleSummaryOp, CollectionSlotLifecycleSummaryPlace,
};
use super::drop_point_path::ResourceDropPointPath;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_summary_engine::summary_check_engine;
use super::initialized_summary_seed::seed_summary_input_place;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    Place, ResourceBlockId, ResourceCallTarget, ResourceFunction, ResourceLocal, ResourceModule,
    ResourceOp,
};
use super::place_utils::{
    place_suffix_after_prefix, projected_place_with_concrete_type, reference_target_place,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::summary_worklist::SummaryWorklist;

pub(super) fn compute_collection_slot_lifecycle_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) -> Vec<CollectionSlotLifecycleFunctionSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    while let Some(function_index) = worklist.pop() {
        let collection_summary_index = CollectionSlotLifecycleFunctionSummaryIndex::new(&summaries);
        let summary = function_collection_slot_lifecycle_summary(
            &module.functions[function_index],
            types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_summary_index,
        );
        if update_collection_slot_lifecycle_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    summaries
}

fn update_collection_slot_lifecycle_summary(
    summaries: &mut Vec<CollectionSlotLifecycleFunctionSummary>,
    summary: CollectionSlotLifecycleFunctionSummary,
) -> bool {
    let has_facts = !summary.ops.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_collection_slot_lifecycle_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) -> CollectionSlotLifecycleFunctionSummary {
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut state = CollectionSlotSummaryBuildState::new(types, function);
    let mut ops = Vec::new();
    for block in &function.blocks {
        collect_summary_ops_from_ops(
            &mut ops,
            &mut engine,
            &mut state,
            &function.params,
            collection_slot_summaries,
            &block.ops,
        );
    }
    CollectionSlotLifecycleFunctionSummary {
        function: function.name.clone(),
        ops,
    }
}

#[derive(Clone)]
struct CollectionSlotSummaryBuildState {
    cells: CellTable,
    collection_slots: CollectionSlotStateTable,
    raw_aliases: RawCellAddressAliases,
    function_aliases: FunctionAliasTable,
    pending_reallocs: PendingRawReallocs,
    variant_initializations: PendingVariantRawCellInitializations,
}

impl CollectionSlotSummaryBuildState {
    fn new(types: &TypeCtx, function: &ResourceFunction) -> Self {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        for param in &function.params {
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = reference_target_type(types, param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
            }
        }
        Self {
            cells,
            collection_slots: CollectionSlotStateTable::new(),
            raw_aliases,
            function_aliases: FunctionAliasTable::default(),
            pending_reallocs: PendingRawReallocs::default(),
            variant_initializations: PendingVariantRawCellInitializations::default(),
        }
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

fn collect_summary_ops_from_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &mut ResourceCheckEngine<'_>,
    state: &mut CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    for op in ops {
        collect_summary_ops_from_op(out, engine, state, params, collection_slot_summaries, op);
        engine.check_ops(
            &mut state.cells,
            &mut state.collection_slots,
            &mut state.raw_aliases,
            &mut state.function_aliases,
            &mut state.pending_reallocs,
            &mut state.variant_initializations,
            core::slice::from_ref(op),
            ResourceDropPointPath {
                block: ResourceBlockId(usize::MAX),
                steps: Vec::new(),
            },
        );
        engine.auto_drop_points.clear();
    }
}

fn collect_summary_ops_from_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::CollectionSlotLifecycle { target, event, .. } => {
            if let Some(target) = summary_place_for_params(params, target) {
                out.push(CollectionSlotLifecycleSummaryOp::Event {
                    target,
                    event: *event,
                });
            }
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } => {
            if let (Some(old_storage), Some(new_storage)) = (
                summary_place_for_params(params, old_storage),
                summary_place_for_params(params, new_storage),
            ) {
                out.push(CollectionSlotLifecycleSummaryOp::Relocate {
                    old_storage,
                    new_storage,
                });
            }
        }
        ResourceOp::Call { target, args, .. } => {
            collect_direct_call_summary_ops(
                out,
                engine,
                target,
                args,
                params,
                collection_slot_summaries,
            );
        }
        ResourceOp::IndirectCall { callee, args, .. } => {
            collect_indirect_call_summary_ops(
                out,
                engine,
                state,
                callee,
                args,
                params,
                collection_slot_summaries,
            );
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            let then_path = collect_nested_summary_ops(
                engine,
                state,
                params,
                collection_slot_summaries,
                then_ops,
            );
            let else_path = collect_nested_summary_ops(
                engine,
                state,
                params,
                collection_slot_summaries,
                else_ops,
            );
            push_merge_summary(out, vec![then_path, else_path]);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let condition_ops = collect_nested_summary_ops(
                engine,
                state,
                params,
                collection_slot_summaries,
                condition_ops,
            );
            let body_ops = collect_nested_summary_ops(
                engine,
                state,
                params,
                collection_slot_summaries,
                body_ops,
            );
            if !condition_ops.is_empty() || !body_ops.is_empty() {
                out.push(CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops,
                    body_ops,
                });
            }
        }
        ResourceOp::Match { arms, .. } => {
            let mut paths = Vec::new();
            for arm in arms {
                paths.push(collect_nested_summary_ops(
                    engine,
                    state,
                    params,
                    collection_slot_summaries,
                    &arm.ops,
                ));
            }
            push_merge_summary(out, paths);
        }
        ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::Construct { .. }
        | ResourceOp::Expr { .. } => {}
    }
}

fn collect_nested_summary_ops(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    let mut path_engine = summary_check_engine(engine);
    let mut path_state = state.clone();
    let mut out = Vec::new();
    collect_summary_ops_from_ops(
        &mut out,
        &mut path_engine,
        &mut path_state,
        params,
        collection_slot_summaries,
        ops,
    );
    out
}

fn collect_direct_call_summary_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    target: &ResourceCallTarget,
    args: &[Place],
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = collection_slot_summaries.get(name) else {
        return;
    };
    translate_summary_ops_through_args(out, engine, args, params, &summary.ops);
}

fn collect_indirect_call_summary_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    callee: &Place,
    args: &[Place],
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) {
    let mut paths = Vec::new();
    for function in state.function_aliases.functions(callee) {
        let mut path = Vec::new();
        if let Some(summary) = collection_slot_summaries.get(function) {
            translate_summary_ops_through_args(&mut path, engine, args, params, &summary.ops);
        }
        paths.push(path);
    }
    push_merge_summary(out, paths);
}

fn translate_summary_ops_through_args(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    ops: &[CollectionSlotLifecycleSummaryOp],
) {
    for op in ops {
        match op {
            CollectionSlotLifecycleSummaryOp::Event { target, event } => {
                let Some(actual) = instantiate_summary_target(engine, args, target) else {
                    continue;
                };
                if let Some(target) = summary_place_for_params(params, &actual) {
                    out.push(CollectionSlotLifecycleSummaryOp::Event {
                        target,
                        event: *event,
                    });
                }
            }
            CollectionSlotLifecycleSummaryOp::Relocate {
                old_storage,
                new_storage,
            } => {
                let Some(actual_old) = instantiate_summary_target(engine, args, old_storage) else {
                    continue;
                };
                let Some(actual_new) = instantiate_summary_target(engine, args, new_storage) else {
                    continue;
                };
                if let (Some(old_storage), Some(new_storage)) = (
                    summary_place_for_params(params, &actual_old),
                    summary_place_for_params(params, &actual_new),
                ) {
                    out.push(CollectionSlotLifecycleSummaryOp::Relocate {
                        old_storage,
                        new_storage,
                    });
                }
            }
            CollectionSlotLifecycleSummaryOp::Merge { paths } => {
                let mut translated_paths = Vec::new();
                for path in paths {
                    let mut translated = Vec::new();
                    translate_summary_ops_through_args(&mut translated, engine, args, params, path);
                    translated_paths.push(translated);
                }
                push_merge_summary(out, translated_paths);
            }
            CollectionSlotLifecycleSummaryOp::Loop {
                condition_ops,
                body_ops,
            } => {
                let mut translated_condition = Vec::new();
                translate_summary_ops_through_args(
                    &mut translated_condition,
                    engine,
                    args,
                    params,
                    condition_ops,
                );
                let mut translated_body = Vec::new();
                translate_summary_ops_through_args(
                    &mut translated_body,
                    engine,
                    args,
                    params,
                    body_ops,
                );
                if !translated_condition.is_empty() || !translated_body.is_empty() {
                    out.push(CollectionSlotLifecycleSummaryOp::Loop {
                        condition_ops: translated_condition,
                        body_ops: translated_body,
                    });
                }
            }
        }
    }
}

fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    let arg = args.get(target.parameter_index)?;
    Some(projected_place_with_concrete_type(
        engine.types,
        arg,
        &target.suffix,
        target.ty,
    ))
}

fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(target, &param.place) else {
            continue;
        };
        return Some(CollectionSlotLifecycleSummaryPlace {
            parameter_index,
            suffix,
            ty: target.ty,
        });
    }
    None
}

fn push_merge_summary(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    paths: Vec<Vec<CollectionSlotLifecycleSummaryOp>>,
) {
    if paths.is_empty() || paths.iter().all(Vec::is_empty) {
        return;
    }
    out.push(CollectionSlotLifecycleSummaryOp::Merge { paths });
}

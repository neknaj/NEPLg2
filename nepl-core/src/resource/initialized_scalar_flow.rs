extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::condition_fact::record_condition_fact_value_constraints;
use super::function_alias::FunctionAliasTable;
use super::i32_scalar_return_facts::{
    apply_i32_scalar_return_facts,
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter,
    I32ScalarReturnFacts,
};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_op;
use super::initialized_scalar_flow_return_facts::{
    i32_scalar_return_fact_projections, merge_i32_scalar_parameter_condition_paths,
    merge_i32_scalar_return_fact_paths, merge_i32_scalar_return_relation_paths,
};
use super::initialized_scalar_flow_value_cache::{
    preseed_i32_scalar_return_summaries_from_value_cache,
    record_i32_scalar_return_summary_value_cache_candidates,
};
use super::initialized_scalar_flow_variant::{
    merge_i32_scalar_concrete_variants, propagate_i32_scalar_concrete_variant_op,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::model::{
    Place, PlaceProjection, ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::place_utils::{
    match_bind_payload_place, place_suffix_after_prefix, reference_target_place,
    type_can_seed_raw_address_alias,
};
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary_dependency::ResourceSummaryDependencyGraph;
use super::summary_index::{FunctionSummary, SummaryIndex, SummaryNameIndex};
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;
#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
use super::timing::{
    resource_i32_op_timing_enabled, resource_i32_return_timing_enabled,
    resource_timing_function_matches,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnSummary {
    pub(super) function: String,
    pub(super) parameters: Vec<Place>,
    pub(super) facts: I32ScalarReturnFacts,
}

pub(super) type I32ScalarReturnSummaryIndex<'a> = SummaryIndex<'a, I32ScalarReturnSummary>;

impl FunctionSummary for I32ScalarReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Clone, PartialEq, Eq)]
struct I32ScalarPathState {
    raw_aliases: RawCellAddressAliases,
    function_aliases: FunctionAliasTable,
    concrete_variants: I32ScalarConcreteVariants,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct I32ScalarConcreteVariants {
    pub(super) entries: Vec<I32ScalarConcreteVariant>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct I32ScalarConcreteVariant {
    pub(super) place: Place,
    pub(super) variant: String,
}

#[derive(Clone, PartialEq, Eq)]
struct I32ScalarReturnVariantFact {
    suffix: Vec<PlaceProjection>,
    variant: String,
}

pub(super) fn compute_i32_scalar_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    dependency_graph: &ResourceSummaryDependencyGraph,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> (Vec<I32ScalarReturnSummary>, usize) {
    let mut relevance_leaf_cache = I32LeafProjectionCache::default();
    let relevant: Vec<bool> = module
        .functions
        .iter()
        .map(|function| {
            function_i32_scalar_summary_relevant(types, function, &mut relevance_leaf_cache)
        })
        .collect();
    let mut worklist_relevant_functions = relevant.clone();
    let mut preseeded_functions = vec![false; module.functions.len()];
    let mut summaries = Vec::new();
    let mut replay_plan = match (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        (Some(cache), Some(context))
            if cache.stable_entry_collection_enabled()
                || cache.has_i32_scalar_return_replay_entries(context) =>
        {
            Some(cache.begin_i32_scalar_summary_replay_plan(
                context,
                types,
                module,
                dependency_graph,
                &relevant,
            ))
        }
        _ => None,
    };
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        if cache.has_i32_scalar_return_replay_entries(context) {
            preseed_i32_scalar_return_summaries_from_value_cache(
                cache,
                context,
                types,
                module,
                &relevant,
                dependency_graph.dependencies(),
                &mut worklist_relevant_functions,
                &mut preseeded_functions,
                &mut summaries,
                replay_plan.as_mut(),
            );
        }
    }
    let mut worklist = SummaryWorklist::new_filtered_with_dependency_graph(
        module,
        worklist_relevant_functions,
        dependency_graph,
    );
    let mut summary_name_index = SummaryNameIndex::from_entries(&summaries);
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let summary = {
            let scalar_summary_index = summary_name_index.as_summary_index(&summaries);
            function_i32_scalar_return_summary(
                function,
                &scalar_summary_index,
                raw_alias_summaries,
                types,
            )
        };
        log_i32_scalar_summary_fact_counts(function, &summary);
        function_start.log("i32_scalar_summary", function);
        if update_i32_scalar_return_summary(&mut summaries, &mut summary_name_index, summary) {
            worklist.notify_changed(function_index);
        }
    }
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        if cache.stable_entry_collection_enabled() {
            let candidate_skipped_functions =
                worklist.unrecomputed_initial_skips(&preseeded_functions);
            record_i32_scalar_return_summary_value_cache_candidates(
                cache,
                context,
                types,
                module,
                dependency_graph.dependencies(),
                &relevant,
                &candidate_skipped_functions,
                &summaries,
                replay_plan.as_mut(),
            );
        }
    }
    if let (Some(cache), Some(plan)) = (summary_value_cache.as_deref_mut(), replay_plan) {
        cache.finish_i32_scalar_summary_replay_plan(plan);
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_i32_scalar_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    let recomputations = worklist.recomputations();
    (summaries, recomputations)
}

fn log_i32_scalar_summary_fact_counts(
    function: &ResourceFunction,
    summary: &I32ScalarReturnSummary,
) {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_RESOURCE_I32_SUMMARY_FACTS").is_some() {
        let counts = summary.facts.fact_counts();
        std::eprintln!(
            "[resource-i32-summary-facts] function={} total={} aliases={} offsets={} relations={} constants={} return_conditions={} parameter_conditions={}",
            function.name,
            counts.total(),
            counts.aliases,
            counts.offsets,
            counts.relations,
            counts.constants,
            counts.return_conditions,
            counts.parameter_conditions
        );
    }
    #[cfg(any(target_os = "none", target_arch = "wasm32"))]
    {
        let _ = function;
        let _ = summary;
    }
}

fn function_i32_scalar_summary_relevant(
    types: &TypeCtx,
    function: &ResourceFunction,
    leaf_cache: &mut I32LeafProjectionCache,
) -> bool {
    // i32 scalar summary は call 境界を越えて i32 leaf の alias/offset/condition を
    // 伝播するための要約である。引数にも戻り値にも i32 leaf が存在しない関数は、
    // summary を適用できる場所がないため、再計算 worklist から外す。
    type_has_i32_scalar_leaf(types, function.result, leaf_cache)
        || function
            .params
            .iter()
            .any(|param| type_has_i32_scalar_leaf(types, param.place.ty, leaf_cache))
}

fn type_has_i32_scalar_leaf(
    types: &TypeCtx,
    ty: TypeId,
    leaf_cache: &mut I32LeafProjectionCache,
) -> bool {
    !leaf_cache
        .leaf_places_for_conditions(types, &Place::unknown(ty))
        .is_empty()
}

pub(super) fn apply_direct_call_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &I32ScalarReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries.get(name) else {
        return false;
    };
    apply_i32_scalar_summary(raw_aliases, output, args, summary, types)
}

fn update_i32_scalar_return_summary(
    summaries: &mut Vec<I32ScalarReturnSummary>,
    summary_name_index: &mut SummaryNameIndex,
    summary: I32ScalarReturnSummary,
) -> bool {
    let has_facts = !summary.facts.is_empty();
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summary_name_index.insert_at_end(&function, summaries.len());
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            summary_name_index.remove_and_shift(&function, index);
            true
        }
        (false, None) => false,
    }
}

fn function_i32_scalar_return_summary(
    function: &ResourceFunction,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> I32ScalarReturnSummary {
    let mut alias_paths = Vec::new();
    let mut offset_paths = Vec::new();
    let mut relation_paths = Vec::new();
    let mut constant_paths = Vec::new();
    let mut return_condition_paths = Vec::new();
    let mut parameter_condition_paths = Vec::new();
    let mut projection_paths = Vec::new();
    let mut i32_leaf_cache = I32LeafProjectionCache::default();
    let shared_initial_state = if function.blocks.len() > 1 {
        Some(initial_i32_scalar_path_state(function, types))
    } else {
        None
    };
    for block in &function.blocks {
        let initial_state = match &shared_initial_state {
            Some(state) => state.clone(),
            None => initial_i32_scalar_path_state(function, types),
        };
        let states = i32_scalar_path_states_after_ops(
            vec![initial_state],
            &block.ops,
            &function.name,
            scalar_summaries,
            raw_alias_summaries,
            types,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            for state in states {
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                let collect_start = i32_scalar_return_timing_start(function, "collect_facts");
                let path_facts = value
                    .as_ref()
                    .map(|value| {
                        collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
                            &function.params,
                            types,
                            &state.raw_aliases,
                            value,
                            &[],
                            &mut i32_leaf_cache,
                            |projection| {
                                state
                                    .concrete_variants
                                    .projection_is_possible(types, value, projection)
                            },
                        )
                    })
                    .unwrap_or_default();
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                i32_scalar_return_timing_finish(function, "collect_facts", collect_start);
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                let projection_start =
                    i32_scalar_return_timing_start(function, "collect_projections");
                projection_paths.push(
                    value
                        .as_ref()
                        .map(|value| {
                            i32_scalar_return_fact_projections(
                                types,
                                value,
                                &path_facts,
                                &state.concrete_variants,
                                &mut i32_leaf_cache,
                            )
                        })
                        .unwrap_or_default(),
                );
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                i32_scalar_return_timing_finish(function, "collect_projections", projection_start);
                alias_paths.push(path_facts.aliases);
                offset_paths.push(path_facts.offsets);
                relation_paths.push(path_facts.relations);
                constant_paths.push(path_facts.constants);
                return_condition_paths.push(path_facts.return_conditions);
                parameter_condition_paths.push(path_facts.parameter_conditions);
            }
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let merge_start = i32_scalar_return_timing_start(function, "merge_paths");
    let aliases = merge_i32_scalar_return_fact_paths(alias_paths, &projection_paths);
    let offsets = merge_i32_scalar_return_fact_paths(offset_paths, &projection_paths);
    let relations = merge_i32_scalar_return_relation_paths(relation_paths, &projection_paths);
    let constants = merge_i32_scalar_return_fact_paths(constant_paths, &projection_paths);
    let return_conditions =
        merge_i32_scalar_return_fact_paths(return_condition_paths, &projection_paths);
    let parameter_conditions =
        merge_i32_scalar_parameter_condition_paths(parameter_condition_paths);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_timing_finish(function, "merge_paths", merge_start);
    I32ScalarReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        facts: I32ScalarReturnFacts {
            aliases,
            offsets,
            relations,
            constants,
            return_conditions,
            parameter_conditions,
        },
    }
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_return_timing_start(
    function: &ResourceFunction,
    stage: &'static str,
) -> Option<std::time::Instant> {
    if !resource_i32_return_timing_enabled() || !resource_timing_function_matches(&function.name) {
        return None;
    }
    std::eprintln!(
        "[resource-i32-return-timing] start function={} stage={}",
        function.name,
        stage
    );
    Some(std::time::Instant::now())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_return_timing_finish(
    function: &ResourceFunction,
    stage: &'static str,
    start: Option<std::time::Instant>,
) {
    if let Some(start) = start {
        std::eprintln!(
            "[resource-i32-return-timing] end function={} stage={} elapsed_ms={}",
            function.name,
            stage,
            start.elapsed().as_millis()
        );
    }
}

fn initial_i32_scalar_path_state(
    function: &ResourceFunction,
    types: &TypeCtx,
) -> I32ScalarPathState {
    let mut raw_aliases = RawCellAddressAliases::default();
    for param in &function.params {
        if type_can_seed_raw_address_alias(types, param.place.ty) {
            raw_aliases.mark(&param.place);
        }
        let mut cells = CellTable::default();
        seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            if type_can_seed_raw_address_alias(types, target.ty) {
                raw_aliases.mark(&target);
            }
            seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &target);
        }
    }
    I32ScalarPathState {
        raw_aliases,
        function_aliases: FunctionAliasTable::default(),
        concrete_variants: I32ScalarConcreteVariants::default(),
    }
}

fn i32_scalar_path_states_after_ops(
    mut states: Vec<I32ScalarPathState>,
    ops: &[ResourceOp],
    function_name: &str,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> Vec<I32ScalarPathState> {
    for (index, op) in ops.iter().enumerate() {
        let _ = index;
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        let op_start = i32_scalar_op_timing_start(function_name, index, op, states.len());
        let mut next = Vec::new();
        for state in states {
            next.extend(i32_scalar_path_states_after_op(
                state,
                op,
                function_name,
                scalar_summaries,
                raw_alias_summaries,
                types,
            ));
        }
        dedupe_i32_scalar_path_states(&mut next);
        states = next;
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        i32_scalar_op_timing_finish(function_name, index, op, states.len(), op_start);
    }
    states
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_op_timing_start(
    function_name: &str,
    index: usize,
    op: &ResourceOp,
    incoming_paths: usize,
) -> Option<std::time::Instant> {
    if !resource_i32_op_timing_enabled() || !resource_timing_function_matches(function_name) {
        return None;
    }
    std::eprintln!(
        "[resource-i32-op-timing] start function={} op={} kind={} incoming_paths={}",
        function_name,
        index,
        resource_op_kind_for_i32_timing(op),
        incoming_paths
    );
    Some(std::time::Instant::now())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_op_timing_finish(
    function_name: &str,
    index: usize,
    op: &ResourceOp,
    outgoing_paths: usize,
    start: Option<std::time::Instant>,
) {
    if let Some(start) = start {
        std::eprintln!(
            "[resource-i32-op-timing] end function={} op={} kind={} outgoing_paths={} elapsed_ms={}",
            function_name,
            index,
            resource_op_kind_for_i32_timing(op),
            outgoing_paths,
            start.elapsed().as_millis()
        );
    }
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn resource_op_kind_for_i32_timing(op: &ResourceOp) -> &'static str {
    match op {
        ResourceOp::DeclareLocal { .. } => "declare_local",
        ResourceOp::Read { .. } => "read",
        ResourceOp::Move { .. } => "move",
        ResourceOp::Assign { .. } => "assign",
        ResourceOp::Drop { .. } => "drop",
        ResourceOp::EndScope { .. } => "end_scope",
        ResourceOp::CallEffect { .. } => "call_effect",
        ResourceOp::Borrow { .. } => "borrow",
        ResourceOp::Call { .. } => "call",
        ResourceOp::IndirectCall { .. } => "indirect_call",
        ResourceOp::FunctionValue { .. } => "function_value",
        ResourceOp::RawMemory { .. } => "raw_memory",
        ResourceOp::RawAddressAlias { .. } => "raw_address_alias",
        ResourceOp::RawAddressView { .. } => "raw_address_view",
        ResourceOp::StorageOrigin { .. } => "storage_origin",
        ResourceOp::CollectionSlotLifecycle { .. } => "collection_slot_lifecycle",
        ResourceOp::CollectionStorageRelocate { .. } => "collection_storage_relocate",
        ResourceOp::CollectionSlotDropTraversal { .. } => "collection_slot_drop_traversal",
        ResourceOp::CollectionSlotTransformRange { .. } => "collection_slot_transform_range",
        ResourceOp::Construct { .. } => "construct",
        ResourceOp::Expr { .. } => "expr",
        ResourceOp::Branch { .. } => "branch",
        ResourceOp::Loop { .. } => "loop",
        ResourceOp::Match { .. } => "match",
    }
}

fn dedupe_i32_scalar_path_states(states: &mut Vec<I32ScalarPathState>) {
    let mut unique = Vec::new();
    for state in states.drain(..) {
        if !unique.iter().any(|existing| existing == &state) {
            unique.push(state);
        }
    }
    *states = unique;
}

fn merge_or_preserve_i32_scalar_control_flow_paths(
    paths: Vec<I32ScalarPathState>,
    output: &Place,
) -> Vec<I32ScalarPathState> {
    if i32_scalar_paths_have_distinct_return_variants(&paths, output) {
        return preserve_i32_scalar_control_flow_paths(paths);
    }
    merge_i32_scalar_path_states(paths)
}

fn preserve_i32_scalar_control_flow_paths(
    mut paths: Vec<I32ScalarPathState>,
) -> Vec<I32ScalarPathState> {
    // return fact の merge は sibling variant を理解しているため、Branch/Match の
    // 直後に alias graph を合流させると、arm 内でだけ成立する return fact を
    // 収集前に失う。ここでは exact duplicate だけを取り除き、variant 別の事実は
    // return value の projection と一緒に後段へ渡す。
    dedupe_i32_scalar_path_states(&mut paths);
    paths
}

fn merge_i32_scalar_path_states(paths: Vec<I32ScalarPathState>) -> Vec<I32ScalarPathState> {
    if paths.is_empty() {
        return paths;
    }
    vec![I32ScalarPathState {
        raw_aliases: RawCellAddressAliases::merge_paths(
            &paths
                .iter()
                .map(|path| path.raw_aliases.clone())
                .collect::<Vec<_>>(),
        ),
        function_aliases: FunctionAliasTable::merge_paths(
            &paths
                .iter()
                .map(|path| path.function_aliases.clone())
                .collect::<Vec<_>>(),
        ),
        concrete_variants: merge_i32_scalar_concrete_variants(
            &paths
                .iter()
                .map(|path| path.concrete_variants.clone())
                .collect::<Vec<_>>(),
        ),
    }]
}

fn i32_scalar_paths_have_distinct_return_variants(
    paths: &[I32ScalarPathState],
    output: &Place,
) -> bool {
    let Some(first) = paths.first() else {
        return false;
    };
    let first_facts = i32_scalar_return_variant_facts(first, output);
    paths.iter().skip(1).any(|path| {
        !i32_scalar_return_variant_fact_sets_equal(
            &first_facts,
            &i32_scalar_return_variant_facts(path, output),
        )
    })
}

fn i32_scalar_return_variant_facts(
    path: &I32ScalarPathState,
    output: &Place,
) -> Vec<I32ScalarReturnVariantFact> {
    let mut out = Vec::new();
    for entry in &path.concrete_variants.entries {
        let Some(suffix) = place_suffix_after_prefix(&entry.place, output) else {
            continue;
        };
        let fact = I32ScalarReturnVariantFact {
            suffix,
            variant: entry.variant.clone(),
        };
        if !out.iter().any(|existing| existing == &fact) {
            out.push(fact);
        }
    }
    out
}

fn i32_scalar_return_variant_fact_sets_equal(
    left: &[I32ScalarReturnVariantFact],
    right: &[I32ScalarReturnVariantFact],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .all(|fact| right.iter().any(|other| other == fact))
}

fn i32_scalar_path_states_after_op(
    mut state: I32ScalarPathState,
    op: &ResourceOp,
    function_name: &str,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> Vec<I32ScalarPathState> {
    match op {
        ResourceOp::Branch {
            output,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            let mut then_state = state.clone();
            if let Some(condition_fact) = condition_fact {
                record_condition_fact_value_constraints(
                    &mut then_state.raw_aliases,
                    condition_fact,
                    true,
                );
            }
            let mut then_paths = i32_scalar_path_states_after_ops(
                vec![then_state],
                then_ops,
                function_name,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            for then_path in &mut then_paths {
                then_path
                    .raw_aliases
                    .copy_alias_if_tracked(then_value, output);
                then_path.function_aliases.copy_alias(then_value, output);
                then_path.concrete_variants.copy(then_value, output);
            }
            if let Some(condition_fact) = condition_fact {
                record_condition_fact_value_constraints(
                    &mut state.raw_aliases,
                    condition_fact,
                    false,
                );
            }
            let mut else_paths = i32_scalar_path_states_after_ops(
                vec![state],
                else_ops,
                function_name,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            for else_path in &mut else_paths {
                else_path
                    .raw_aliases
                    .copy_alias_if_tracked(else_value, output);
                else_path.function_aliases.copy_alias(else_value, output);
                else_path.concrete_variants.copy(else_value, output);
            }
            then_paths.extend(else_paths);
            merge_or_preserve_i32_scalar_control_flow_paths(then_paths, output)
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            let mut paths = Vec::new();
            for arm in arms {
                let mut arm_state = state.clone();
                if let Some(bind_local) = &arm.bind_local {
                    if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                        arm_state
                            .raw_aliases
                            .copy_alias_if_tracked(&source, bind_local);
                        arm_state.function_aliases.copy_alias(&source, bind_local);
                        arm_state.concrete_variants.copy(&source, bind_local);
                    } else {
                        arm_state.raw_aliases.clear(bind_local);
                        arm_state.concrete_variants.clear(bind_local);
                    }
                }
                for mut path in i32_scalar_path_states_after_ops(
                    vec![arm_state],
                    &arm.ops,
                    function_name,
                    scalar_summaries,
                    raw_alias_summaries,
                    types,
                ) {
                    path.raw_aliases.copy_alias_if_tracked(&arm.value, output);
                    path.function_aliases.copy_alias(&arm.value, output);
                    path.concrete_variants.copy(&arm.value, output);
                    paths.push(path);
                }
            }
            if paths.is_empty() {
                vec![state]
            } else {
                merge_or_preserve_i32_scalar_control_flow_paths(paths, output)
            }
        }
        ResourceOp::Loop { .. } => {
            propagate_i32_scalar_concrete_variant_op(&mut state.concrete_variants, op);
            propagate_i32_scalar_op(
                &mut state.raw_aliases,
                &mut state.function_aliases,
                op,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            vec![state]
        }
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. } => {
            propagate_i32_scalar_concrete_variant_op(&mut state.concrete_variants, op);
            propagate_i32_scalar_op(
                &mut state.raw_aliases,
                &mut state.function_aliases,
                op,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            vec![state]
        }
    }
}

fn apply_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &I32ScalarReturnSummary,
    types: &TypeCtx,
) -> bool {
    apply_i32_scalar_return_facts(raw_aliases, output, args, &summary.facts, types)
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        crate::types::TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

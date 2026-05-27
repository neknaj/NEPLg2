extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::condition_fact::record_condition_fact_value_constraints;
use super::function_alias::FunctionAliasTable;
use super::i32_scalar_return_facts::{
    apply_i32_scalar_return_facts, collect_i32_scalar_return_facts_for_value_suffix_cached,
    I32ScalarParameterCondition, I32ScalarReturnAlias, I32ScalarReturnCondition,
    I32ScalarReturnConstant, I32ScalarReturnFacts, I32ScalarReturnOffset, I32ScalarReturnRelation,
};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_op;
use super::initialized_str_layout::seed_str_storage_layout;
use super::model::{
    AggregateKind, Place, PlaceProjection, ResourceCallTarget, ResourceFunction, ResourceModule,
    ResourceOp, ResourceTerminator,
};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::place_utils::{
    construct_aggregate_field_place, match_bind_payload_place, place_suffix_after_prefix,
    projection_result_type, reference_target_place, replace_place_prefix,
    type_can_seed_raw_address_alias,
};
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;
use super::variant_name::normalize_variant_name;

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
struct I32ScalarConcreteVariants {
    entries: Vec<I32ScalarConcreteVariant>,
}

#[derive(Clone, PartialEq, Eq)]
struct I32ScalarConcreteVariant {
    place: Place,
    variant: String,
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
) -> Vec<I32ScalarReturnSummary> {
    let relevant = module
        .functions
        .iter()
        .map(|function| function_i32_scalar_summary_relevant(types, function))
        .collect();
    let mut worklist = SummaryWorklist::new_filtered(module, relevant);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let scalar_summary_index = I32ScalarReturnSummaryIndex::new(&summaries);
        let summary = function_i32_scalar_return_summary(
            function,
            &scalar_summary_index,
            raw_alias_summaries,
            types,
        );
        log_i32_scalar_summary_fact_counts(function, &summary);
        function_start.log("i32_scalar_summary", function);
        if update_i32_scalar_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_i32_scalar_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
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

fn function_i32_scalar_summary_relevant(types: &TypeCtx, function: &ResourceFunction) -> bool {
    // i32 scalar summary は call 境界を越えて i32 leaf の alias/offset/condition を
    // 伝播するための要約である。引数にも戻り値にも i32 leaf が存在しない関数は、
    // summary を適用できる場所がないため、再計算 worklist から外す。
    type_has_i32_scalar_leaf(types, function.result)
        || function
            .params
            .iter()
            .any(|param| type_has_i32_scalar_leaf(types, param.place.ty))
}

fn type_has_i32_scalar_leaf(types: &TypeCtx, ty: TypeId) -> bool {
    !I32LeafProjectionCache::default()
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
    summary: I32ScalarReturnSummary,
) -> bool {
    let has_facts = !summary.facts.is_empty();
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
    for block in &function.blocks {
        let states = i32_scalar_path_states_after_ops(
            vec![initial_i32_scalar_path_state(function, types)],
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
                        collect_i32_scalar_return_facts_for_value_suffix_cached(
                            &function.params,
                            types,
                            &state.raw_aliases,
                            value,
                            &[],
                            &mut i32_leaf_cache,
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
    if std::env::var_os("NEPL_RESOURCE_I32_RETURN_TIMING").is_none() {
        return None;
    }
    if let Some(filter) = std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION")
        .ok()
        .filter(|filter| !function.name.contains(filter))
    {
        let _ = filter;
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
    if std::env::var_os("NEPL_RESOURCE_I32_OP_TIMING").is_none() {
        return None;
    }
    if let Some(filter) = std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION")
        .ok()
        .filter(|filter| !function_name.contains(filter))
    {
        let _ = filter;
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

trait I32ScalarReturnProjectedFact: Clone + Eq {
    fn return_projection(&self) -> &[PlaceProjection];
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnAlias {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnOffset {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnConstant {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnCondition {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

fn i32_scalar_return_fact_projections(
    types: &TypeCtx,
    value: &Place,
    facts: &I32ScalarReturnFacts,
    concrete_variants: &I32ScalarConcreteVariants,
    leaf_cache: &mut I32LeafProjectionCache,
) -> Vec<Vec<PlaceProjection>> {
    let mut projections = Vec::new();
    for leaf in leaf_cache.leaf_places_for_conditions(types, value) {
        if !concrete_variants.projection_is_possible(types, value, &leaf.suffix) {
            continue;
        }
        push_unique_i32_scalar_return_projection(&mut projections, &leaf.suffix);
    }
    concrete_variants.push_variant_projection_paths(value, &mut projections);
    for alias in &facts.aliases {
        push_unique_i32_scalar_return_projection(&mut projections, &alias.return_projection);
    }
    for offset in &facts.offsets {
        push_unique_i32_scalar_return_projection(&mut projections, &offset.return_projection);
    }
    for relation in &facts.relations {
        push_unique_i32_scalar_return_projection(
            &mut projections,
            &relation.left_return_projection,
        );
        push_unique_i32_scalar_return_projection(
            &mut projections,
            &relation.right_return_projection,
        );
    }
    for constant in &facts.constants {
        push_unique_i32_scalar_return_projection(&mut projections, &constant.return_projection);
    }
    for condition in &facts.return_conditions {
        push_unique_i32_scalar_return_projection(&mut projections, &condition.return_projection);
    }
    projections
}

fn push_unique_i32_scalar_return_projection(
    projections: &mut Vec<Vec<PlaceProjection>>,
    projection: &[PlaceProjection],
) {
    if !projections
        .iter()
        .any(|existing| existing.as_slice() == projection)
    {
        projections.push(projection.to_vec());
    }
}

fn merge_i32_scalar_return_fact_paths<T>(
    paths: Vec<Vec<T>>,
    projection_paths: &[Vec<Vec<PlaceProjection>>],
) -> Vec<T>
where
    T: I32ScalarReturnProjectedFact,
{
    let mut candidates = Vec::new();
    for path in &paths {
        for fact in path {
            push_unique_i32_scalar_return_fact(&mut candidates, fact.clone());
        }
    }
    candidates
        .into_iter()
        .filter(|fact| {
            paths
                .iter()
                .zip(projection_paths)
                .all(|(path, projections)| {
                    path.iter().any(|path_fact| path_fact == fact)
                        || projections.iter().any(|projection| {
                            return_projections_target_sibling_variant(
                                fact.return_projection(),
                                projection,
                            )
                        })
                })
        })
        .collect()
}

fn merge_i32_scalar_return_relation_paths(
    paths: Vec<Vec<I32ScalarReturnRelation>>,
    projection_paths: &[Vec<Vec<PlaceProjection>>],
) -> Vec<I32ScalarReturnRelation> {
    let mut candidates = Vec::new();
    for path in &paths {
        for relation in path {
            push_unique_i32_scalar_return_relation(&mut candidates, relation.clone());
        }
    }
    candidates
        .into_iter()
        .filter(|relation| {
            paths
                .iter()
                .zip(projection_paths)
                .all(|(path, projections)| {
                    path.iter().any(|path_relation| path_relation == relation)
                        || relation_projection_targets_sibling_variant(relation, projections)
                })
        })
        .collect()
}

fn relation_projection_targets_sibling_variant(
    relation: &I32ScalarReturnRelation,
    projections: &[Vec<PlaceProjection>],
) -> bool {
    projections.iter().any(|projection| {
        return_projections_target_sibling_variant(&relation.left_return_projection, projection)
            || return_projections_target_sibling_variant(
                &relation.right_return_projection,
                projection,
            )
    })
}

fn push_unique_i32_scalar_return_relation(
    relations: &mut Vec<I32ScalarReturnRelation>,
    relation: I32ScalarReturnRelation,
) {
    if !relations.iter().any(|existing| existing == &relation) {
        relations.push(relation);
    }
}

fn push_unique_i32_scalar_return_fact<T>(facts: &mut Vec<T>, fact: T)
where
    T: I32ScalarReturnProjectedFact,
{
    if !facts.iter().any(|existing| existing == &fact) {
        facts.push(fact);
    }
}

fn merge_i32_scalar_parameter_condition_paths(
    paths: Vec<Vec<I32ScalarParameterCondition>>,
) -> Vec<I32ScalarParameterCondition> {
    let mut out = Vec::new();
    if let Some(first) = paths.first() {
        for fact in first {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.iter().any(|existing| existing == fact))
            {
                push_unique_i32_scalar_parameter_condition(&mut out, fact.clone());
            }
        }
    }
    out
}

fn push_unique_i32_scalar_parameter_condition(
    facts: &mut Vec<I32ScalarParameterCondition>,
    fact: I32ScalarParameterCondition,
) {
    if !facts.iter().any(|existing| existing == &fact) {
        facts.push(fact);
    }
}

fn return_projections_target_sibling_variant(
    left_projection: &[PlaceProjection],
    right_projection: &[PlaceProjection],
) -> bool {
    left_projection
        .iter()
        .zip(right_projection)
        .enumerate()
        .any(|(index, (left, right))| {
            matches!(
                (left, right),
                (
                    PlaceProjection::EnumPayload { variant: left_variant },
                    PlaceProjection::EnumPayload {
                        variant: right_variant
                    },
                ) if left_variant != right_variant
                    && place_projection_prefixes_match(
                        &left_projection[..index],
                        &right_projection[..index],
                    )
            )
        })
}

fn place_projection_prefixes_match(left: &[PlaceProjection], right: &[PlaceProjection]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(left, right)| left == right)
}

impl I32ScalarConcreteVariants {
    fn clear(&mut self, place: &Place) {
        self.entries.retain(|entry| {
            super::place_utils::place_suffix_after_prefix(&entry.place, place).is_none()
        });
    }

    fn set(&mut self, place: &Place, variant: &str) {
        self.clear(place);
        self.entries.push(I32ScalarConcreteVariant {
            place: place.clone(),
            variant: normalize_variant_name(variant),
        });
    }

    fn copy(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copied = self
            .entries
            .iter()
            .filter_map(|entry| {
                replace_place_prefix(&entry.place, source, target).map(|place| {
                    I32ScalarConcreteVariant {
                        place,
                        variant: entry.variant.clone(),
                    }
                })
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for entry in copied {
            self.push_unique(entry);
        }
    }

    fn variant_for(&self, place: &Place) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.variant.as_str())
    }

    fn push_variant_projection_paths(
        &self,
        value: &Place,
        projections: &mut Vec<Vec<PlaceProjection>>,
    ) {
        for entry in &self.entries {
            let Some(prefix) = super::place_utils::place_suffix_after_prefix(&entry.place, value)
            else {
                continue;
            };
            let mut projection = prefix;
            projection.push(PlaceProjection::EnumPayload {
                variant: entry.variant.clone(),
            });
            push_unique_i32_scalar_return_projection(projections, &projection);
        }
    }

    fn projection_is_possible(
        &self,
        types: &TypeCtx,
        value: &Place,
        projection: &[PlaceProjection],
    ) -> bool {
        let mut prefix = Vec::new();
        let mut current_ty = value.ty;
        for item in projection {
            if let PlaceProjection::EnumPayload { variant } = item {
                let enum_place = super::place_utils::place_with_suffix(value, &prefix, current_ty);
                if let Some(known) = self.variant_for(&enum_place) {
                    if known != normalize_variant_name(variant) {
                        return false;
                    }
                }
            }
            current_ty = projection_result_type(types, current_ty, item).unwrap_or(current_ty);
            prefix.push(item.clone());
        }
        true
    }

    fn push_unique(&mut self, entry: I32ScalarConcreteVariant) {
        if self.entries.iter().any(|existing| existing == &entry) {
            return;
        }
        self.entries.push(entry);
    }
}

fn propagate_i32_scalar_concrete_variant_op(
    variants: &mut I32ScalarConcreteVariants,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                variants.copy(initializer, place);
            } else {
                variants.clear(place);
            }
        }
        ResourceOp::Read { source, output, .. }
        | ResourceOp::Move { source, output, .. }
        | ResourceOp::Assign {
            target: output,
            value: source,
            ..
        } => variants.copy(source, output),
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            variants.clear(output);
            for (index, input) in inputs.iter().enumerate() {
                let field = construct_aggregate_field_place(output, kind, index, input);
                variants.copy(input, &field);
            }
            if let AggregateKind::Enum { variant, .. } = kind {
                variants.set(output, variant);
            }
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_variants = variants.clone();
            propagate_i32_scalar_concrete_variant_ops(&mut condition_variants, condition_ops);
            let mut body_variants = condition_variants.clone();
            propagate_i32_scalar_concrete_variant_ops(&mut body_variants, body_ops);
            *variants = merge_i32_scalar_concrete_variants(&[condition_variants, body_variants]);
        }
        ResourceOp::Expr { output, .. }
        | ResourceOp::Call { output, .. }
        | ResourceOp::IndirectCall { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::RawMemory { output, .. }
        | ResourceOp::Borrow { output, .. } => variants.clear(output),
        ResourceOp::Drop { place, .. } => variants.clear(place),
        ResourceOp::Branch { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => {}
    }
}

fn propagate_i32_scalar_concrete_variant_ops(
    variants: &mut I32ScalarConcreteVariants,
    ops: &[ResourceOp],
) {
    for op in ops {
        propagate_i32_scalar_concrete_variant_op(variants, op);
    }
}

fn merge_i32_scalar_concrete_variants(
    paths: &[I32ScalarConcreteVariants],
) -> I32ScalarConcreteVariants {
    let mut out = I32ScalarConcreteVariants::default();
    let Some(first) = paths.first() else {
        return out;
    };
    for entry in &first.entries {
        if paths
            .iter()
            .skip(1)
            .all(|path| path.entries.iter().any(|path_entry| path_entry == entry))
        {
            out.push_unique(entry.clone());
        }
    }
    out
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        crate::types::TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

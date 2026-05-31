use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::{FileId, Span};
use crate::types::{TypeCtx, TypeId};

use super::super::i32_scalar_return_facts::{
    I32ScalarParameterCondition, I32ScalarReturnAlias, I32ScalarReturnCondition,
    I32ScalarReturnConstant, I32ScalarReturnFacts, I32ScalarReturnOffset, I32ScalarReturnRelation,
};
use super::super::model::{
    EffectOp, I32ValueCondition, Place, ResourceBlock, ResourceBlockId, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceI32RelationOp, ResourceId, ResourceLocal,
    ResourceModule, ResourceOp, ResourceTerminator,
};
use super::super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::super::summary_dependency::build_function_summary_dependencies;
use super::*;

fn test_span() -> Span {
    Span::new(FileId(0), 1, 2)
}

fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
    let mut context = ResourceSummaryValueCacheContext::new(7);
    context.insert_source_policy_hash(FileId(0), policy_hash);
    context
}

fn i32_scalar_function(
    _types: &TypeCtx,
    name: &str,
    param_ty: TypeId,
    result_ty: TypeId,
    body_variant: bool,
) -> ResourceFunction {
    let span = test_span();
    let param = Place::local(String::from("value"), param_ty);
    let result = Place::temporary(ResourceId(0), result_ty);
    let literal = if body_variant { 2 } else { 1 };
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("value"),
            ty: param_ty,
            mutable: false,
            place: param,
        }],
        result: result_ty,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(literal),
                output: result.clone(),
                ty: result_ty,
                span,
            }],
            terminator: ResourceTerminator::Return {
                value: Some(result),
                span,
            },
            span,
        }],
        span,
    }
}

fn i32_scalar_dependent_function(
    types: &TypeCtx,
    name: &str,
    dependency: &str,
) -> ResourceFunction {
    let span = test_span();
    let param = Place::local(String::from("value"), types.i32());
    let output = Place::temporary(ResourceId(10), types.i32());
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("value"),
            ty: types.i32(),
            mutable: false,
            place: param.clone(),
        }],
        result: types.i32(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::Call {
                output: output.clone(),
                target: ResourceCallTarget::User {
                    name: String::from(dependency),
                    type_args: Vec::new(),
                },
                args: vec![param],
                effect: EffectOp::Pure,
                span,
            }],
            terminator: ResourceTerminator::Return {
                value: Some(output),
                span,
            },
            span,
        }],
        span,
    }
}

fn i32_scalar_summary_for(name: &str, ty: TypeId) -> I32ScalarReturnSummary {
    I32ScalarReturnSummary {
        function: String::from(name),
        parameters: vec![Place::local(String::from("value"), ty)],
        facts: I32ScalarReturnFacts {
            aliases: vec![I32ScalarReturnAlias {
                return_projection: Vec::new(),
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: ty,
            }],
            offsets: vec![I32ScalarReturnOffset {
                return_projection: Vec::new(),
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: ty,
                offset: 1,
            }],
            relations: vec![I32ScalarReturnRelation {
                left_return_projection: Vec::new(),
                op: ResourceI32RelationOp::Eq,
                right_return_projection: Vec::new(),
                scalar_ty: ty,
            }],
            constants: vec![I32ScalarReturnConstant {
                return_projection: Vec::new(),
                scalar_ty: ty,
                value: 7,
            }],
            return_conditions: vec![I32ScalarReturnCondition {
                return_projection: Vec::new(),
                scalar_ty: ty,
                condition: I32ValueCondition::Positive,
            }],
            parameter_conditions: vec![I32ScalarParameterCondition {
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: ty,
                condition: I32ValueCondition::NonNegative,
            }],
        },
    }
}

fn single_function_module(function: ResourceFunction) -> ResourceModule {
    ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: Vec::new(),
    }
}

fn record_i32_summary(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    summaries: &[I32ScalarReturnSummary],
) {
    let dependencies = build_function_summary_dependencies(module);
    let relevant_functions = vec![true; module.functions.len()];
    let preseeded_functions = vec![false; module.functions.len()];
    record_i32_scalar_return_summary_value_cache_candidates(
        cache,
        context,
        types,
        module,
        &dependencies,
        &relevant_functions,
        &preseeded_functions,
        summaries,
    );
}

fn preseed_i32_summaries(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
) -> (Vec<bool>, Vec<bool>, Vec<I32ScalarReturnSummary>) {
    let dependencies = build_function_summary_dependencies(module);
    let relevant_functions = vec![true; module.functions.len()];
    let mut worklist_relevant_functions = relevant_functions.clone();
    let mut preseeded_functions = vec![false; module.functions.len()];
    let mut summaries = Vec::new();
    preseed_i32_scalar_return_summaries_from_value_cache(
        cache,
        context,
        types,
        module,
        &relevant_functions,
        &dependencies,
        &mut worklist_relevant_functions,
        &mut preseeded_functions,
        &mut summaries,
    );
    (worklist_relevant_functions, preseeded_functions, summaries)
}

/// i32 scalar summary cache は、aliases / offsets / relations / constants /
/// conditions を同じ complete entry として保存し、worklist 前に完全な summary
/// surface として戻せる場合だけ対象関数を skip する。skip 済み関数は同じ compile
/// の末尾で candidate hit として再記録せず、replay counter だけで再利用を示す。
#[test]
fn i32_scalar_return_facts_preseed_replays_same_summary_surface() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let module = single_function_module(i32_scalar_function(
        &types,
        "i32_leaf",
        types.i32(),
        types.i32(),
        false,
    ));
    let summary = i32_scalar_summary_for("i32_leaf", types.i32());
    let context = test_context(11);
    record_i32_summary(
        &mut cache,
        &context,
        &types,
        &module,
        core::slice::from_ref(&summary),
    );

    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_i32_summaries(&mut cache, &context, &types, &module);

    assert_eq!(worklist_relevant_functions, vec![false]);
    assert_eq!(preseeded_functions, vec![true]);
    assert_eq!(summaries, vec![summary]);
    let stats = cache.stats();
    assert_eq!(
        stats.resource_summary_value_i32_scalar_return_facts_stores,
        6
    );
    assert_eq!(stats.resource_summary_value_replay_hits, 6);
    assert_eq!(stats.resource_summary_value_replayed_ops, 6);

    let dependencies = build_function_summary_dependencies(&module);
    let relevant_functions = vec![true; module.functions.len()];
    record_i32_scalar_return_summary_value_cache_candidates(
        &mut cache,
        &context,
        &types,
        &module,
        &dependencies,
        &relevant_functions,
        &preseeded_functions,
        &summaries,
    );

    let stats = cache.stats();
    assert_eq!(stats.resource_summary_value_i32_scalar_return_facts_hits, 0);
    assert_eq!(stats.resource_summary_value_recomputed_ops, 6);
}

/// facts が空の relevant function も「空の summary が固定点結果である」と cache できる。
/// 空 entry を保存しないと、RPN のような大きい stdlib closure で no-fact function が
/// 微小編集ごとに worklist へ戻り、i32 scalar stage の固定費が残る。
#[test]
fn i32_scalar_empty_return_facts_preseed_skips_no_fact_function() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let module = single_function_module(i32_scalar_function(
        &types,
        "i32_empty",
        types.i32(),
        types.i32(),
        false,
    ));
    let context = test_context(11);
    record_i32_summary(&mut cache, &context, &types, &module, &[]);

    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_i32_summaries(&mut cache, &context, &types, &module);

    assert_eq!(worklist_relevant_functions, vec![false]);
    assert_eq!(preseeded_functions, vec![true]);
    assert!(summaries.is_empty());
    let stats = cache.stats();
    assert_eq!(
        stats.resource_summary_value_i32_scalar_return_facts_stores,
        0
    );
    assert_eq!(stats.resource_summary_value_replay_hits, 0);
}

/// i32 scalar cache key は function body、source policy、signature type boundary を
/// stale hit 防止入力として含む。いずれかが変わる場合、保存済み facts は replay
/// せず通常の fixed-point worklist に戻る。
#[test]
fn i32_scalar_return_facts_preseed_misses_on_body_source_or_signature_change() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let base_module = single_function_module(i32_scalar_function(
        &types,
        "i32_leaf",
        types.i32(),
        types.i32(),
        false,
    ));
    let summary = i32_scalar_summary_for("i32_leaf", types.i32());
    let context = test_context(11);
    record_i32_summary(
        &mut cache,
        &context,
        &types,
        &base_module,
        core::slice::from_ref(&summary),
    );

    let body_changed = single_function_module(i32_scalar_function(
        &types,
        "i32_leaf",
        types.i32(),
        types.i32(),
        true,
    ));
    let source_changed = test_context(12);
    let signature_changed = single_function_module(i32_scalar_function(
        &types,
        "i32_leaf",
        types.u8(),
        types.i32(),
        false,
    ));

    for (module, context) in [
        (&body_changed, &context),
        (&base_module, &source_changed),
        (&signature_changed, &context),
    ] {
        let (worklist_relevant_functions, preseeded_functions, summaries) =
            preseed_i32_summaries(&mut cache, context, &types, module);
        assert_eq!(worklist_relevant_functions, vec![true]);
        assert_eq!(preseeded_functions, vec![false]);
        assert!(summaries.is_empty());
    }

    let stats = cache.stats();
    assert_eq!(stats.resource_summary_value_replay_hits, 0);
    assert_eq!(stats.resource_summary_value_replayed_ops, 0);
}

/// i32 scalar summary は callee の i32/raw-alias summary を取り込むため、caller body
/// だけを key にすると callee edit 後に stale replay し得る。dependency closure hash
/// により callee body が変わると caller summary も miss する。
#[test]
fn i32_scalar_return_facts_dependency_closure_invalidates_on_callee_body_change() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let callee = i32_scalar_function(&types, "callee", types.i32(), types.i32(), false);
    let caller = i32_scalar_dependent_function(&types, "caller", "callee");
    let module = ResourceModule {
        functions: vec![callee, caller],
        entry: None,
        string_literals: Vec::new(),
    };
    let context = test_context(11);
    let summary = i32_scalar_summary_for("caller", types.i32());
    record_i32_summary(
        &mut cache,
        &context,
        &types,
        &module,
        core::slice::from_ref(&summary),
    );

    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_i32_summaries(&mut cache, &context, &types, &module);
    assert_eq!(worklist_relevant_functions, vec![false, false]);
    assert_eq!(preseeded_functions, vec![true, true]);
    assert_eq!(summaries, vec![summary]);

    let changed_callee = i32_scalar_function(&types, "callee", types.i32(), types.i32(), true);
    let changed_module = ResourceModule {
        functions: vec![
            changed_callee,
            i32_scalar_dependent_function(&types, "caller", "callee"),
        ],
        entry: None,
        string_literals: Vec::new(),
    };
    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_i32_summaries(&mut cache, &context, &types, &changed_module);

    assert_eq!(worklist_relevant_functions, vec![true, true]);
    assert_eq!(preseeded_functions, vec![false, false]);
    assert!(summaries.is_empty());
}

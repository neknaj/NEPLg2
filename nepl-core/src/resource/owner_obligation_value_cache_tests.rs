use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::{FileId, Span};
use crate::types::TypeCtx;

use super::super::model::{
    Place, RawAddressAliasKind, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal,
    ResourceModule, ResourceOp, ResourceTerminator,
};
use super::super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::*;

fn test_span() -> Span {
    Span::new(FileId(0), 1, 2)
}

fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
    let mut context = ResourceSummaryValueCacheContext::new(7);
    context.insert_source_policy_hash(FileId(0), policy_hash);
    context
}

fn local_identity_function(types: &TypeCtx, name: &str, unreachable: bool) -> ResourceFunction {
    let span = test_span();
    let param_place = Place::local(String::from("value"), types.i32());
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("value"),
            ty: types.i32(),
            mutable: false,
            place: param_place.clone(),
        }],
        result: types.i32(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: Vec::new(),
            terminator: if unreachable {
                ResourceTerminator::Unreachable { span }
            } else {
                ResourceTerminator::Return {
                    value: Some(param_place),
                    span,
                }
            },
            span,
        }],
        span,
    }
}

fn raw_alias_function(types: &TypeCtx, name: &str, unreachable: bool) -> ResourceFunction {
    let span = test_span();
    let raw_place = Place::local(String::from("raw"), types.i32());
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("raw"),
            ty: types.i32(),
            mutable: false,
            place: raw_place.clone(),
        }],
        result: types.unit(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::RawAddressAlias {
                source: raw_place,
                target: Place::local(String::from("raw_alias"), types.i32()),
                kind: RawAddressAliasKind::Transparent,
                span,
            }],
            terminator: if unreachable {
                ResourceTerminator::Unreachable { span }
            } else {
                ResourceTerminator::Return { value: None, span }
            },
            span,
        }],
        span,
    }
}

fn single_function_module(function: ResourceFunction) -> ResourceModule {
    ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: Vec::new(),
    }
}

/// owner obligation cache は、診断なしの関数だけを pass entry として保存する。
/// 二回目の同一 compile では owner checker 本体を起動せず、後続 gate が見る
/// diagnostics-free surface と deferred counter だけを戻す。
#[test]
fn owner_obligation_check_replays_without_rerunning_checker() {
    let types = TypeCtx::new();
    let module = single_function_module(raw_alias_function(&types, "raw_alias", false));
    let context = test_context(11);
    let mut cache = ResourceSummaryValueCache::new();

    let first =
        check_resource_owner_obligations_with_summary_cache(&module, &types, &mut cache, &context);
    let second =
        check_resource_owner_obligations_with_summary_cache(&module, &types, &mut cache, &context);

    assert_eq!(first.diagnostics, second.diagnostics);
    assert_eq!(first.deferred, second.deferred);
    assert_eq!(first.functions.len(), second.functions.len());
    assert_eq!(first.functions[0].name, second.functions[0].name);
    assert!(second.functions[0].final_owners.is_empty());
    let stats = cache.stats();
    assert_eq!(stats.resource_owner_obligation_function_checks, 1);
    assert_eq!(
        stats.resource_summary_value_owner_obligation_check_stores,
        1
    );
    assert_eq!(stats.resource_summary_value_owner_obligation_check_hits, 1);
    assert_eq!(
        stats.resource_summary_value_owner_obligation_check_plan_skip_functions,
        1
    );
    assert_eq!(stats.resource_owner_return_summary_recomputations, 1);
    assert_eq!(
        stats.resource_owner_return_summary_pass_cache_skip_functions,
        1
    );
}

/// scalar-only の pure 関数は owner obligation が観測する資源を持たない。
/// cache が有効な compile でも stable key probe や checker 起動へ進めず、no-cache 経路と
/// 同じ空の検査結果を返す。
#[test]
fn owner_obligation_check_skips_scalar_function_before_cache_probe() {
    let types = TypeCtx::new();
    let module = single_function_module(local_identity_function(&types, "identity", false));
    let context = test_context(11);
    let mut cache = ResourceSummaryValueCache::new();

    let without_cache = check_resource_owner_obligations(&module, &types);
    let with_cache =
        check_resource_owner_obligations_with_summary_cache(&module, &types, &mut cache, &context);

    assert_eq!(without_cache, with_cache);
    let stats = cache.stats();
    assert_eq!(stats.resource_owner_obligation_function_checks, 0);
    assert_eq!(
        stats.resource_summary_value_owner_obligation_check_stores,
        0
    );
    assert_eq!(stats.resource_summary_value_owner_obligation_check_hits, 0);
    assert_eq!(stats.resource_owner_return_summary_recomputations, 0);
    assert_eq!(
        stats.resource_owner_return_summary_pass_cache_skip_functions,
        0
    );
}

/// cache key は function body hash を含むため、同じ名前と同じ signature でも本文が
/// 変わった関数は replay できない。本文編集後は owner checker を再実行し、新しい
/// diagnostic-free entry として保存する。
#[test]
fn owner_obligation_check_misses_after_body_change() {
    let types = TypeCtx::new();
    let original = single_function_module(raw_alias_function(&types, "raw_alias", false));
    let edited = single_function_module(raw_alias_function(&types, "raw_alias", true));
    let context = test_context(11);
    let mut cache = ResourceSummaryValueCache::new();

    check_resource_owner_obligations_with_summary_cache(&original, &types, &mut cache, &context);
    check_resource_owner_obligations_with_summary_cache(&edited, &types, &mut cache, &context);

    let stats = cache.stats();
    assert_eq!(stats.resource_owner_obligation_function_checks, 2);
    assert_eq!(
        stats.resource_summary_value_owner_obligation_check_stores,
        2
    );
    assert_eq!(stats.resource_summary_value_owner_obligation_check_hits, 0);
    assert_eq!(stats.resource_owner_return_summary_recomputations, 2);
    assert_eq!(
        stats.resource_owner_return_summary_pass_cache_skip_functions,
        0
    );
}

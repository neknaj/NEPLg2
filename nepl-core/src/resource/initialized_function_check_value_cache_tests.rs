use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::{FileId, Span};
use crate::types::TypeCtx;

use super::super::model::{
    Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceModule,
    ResourceTerminator,
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

fn single_function_module(function: ResourceFunction) -> ResourceModule {
    ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: Vec::new(),
    }
}

/// final initialized function check cache は、診断も auto drop plan も持たない関数だけを
/// stable entry として保存する。二回目の同一 compile では `ResourceCheckEngine` を
/// 起動せず、後続 stage が使う checked-pass surface だけを戻すことで skip を観測できる。
#[test]
fn final_initialized_function_check_replays_without_rerunning_checker() {
    let types = TypeCtx::new();
    let module = single_function_module(local_identity_function(&types, "identity", false));
    let context = test_context(11);
    let mut cache = ResourceSummaryValueCache::new();

    let first =
        check_resource_initialized_moves_with_summary_cache(&module, &types, &mut cache, &context);
    let second =
        check_resource_initialized_moves_with_summary_cache(&module, &types, &mut cache, &context);

    assert_eq!(first.diagnostics, second.diagnostics);
    assert_eq!(first.deferred, second.deferred);
    assert_eq!(first.functions.len(), second.functions.len());
    assert_eq!(first.functions[0].name, second.functions[0].name);
    assert_eq!(
        first.functions[0].auto_drop_points,
        second.functions[0].auto_drop_points
    );
    assert!(second.functions[0].final_cells.is_empty());
    assert!(second.functions[0].final_collection_slots.is_empty());
    let stats = cache.stats();
    assert_eq!(stats.resource_initialized_function_checks, 1);
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_stores,
        1
    );
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_hits,
        1
    );
    assert_eq!(stats.resource_summary_value_lazy_pass_hits, 1);
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_plan_skip_functions,
        1
    );
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_replay_probe_functions,
        1
    );
}

/// cache key は function body hash を含むため、同じ名前と同じ signature でも本文が
/// 変わった関数は replay できない。本文編集後は現行 checker を再実行し、その結果を
/// 新しい entry として保存する。
#[test]
fn final_initialized_function_check_misses_after_body_change() {
    let types = TypeCtx::new();
    let original = single_function_module(local_identity_function(&types, "identity", false));
    let edited = single_function_module(local_identity_function(&types, "identity", true));
    let context = test_context(11);
    let mut cache = ResourceSummaryValueCache::new();

    check_resource_initialized_moves_with_summary_cache(&original, &types, &mut cache, &context);
    check_resource_initialized_moves_with_summary_cache(&edited, &types, &mut cache, &context);

    let stats = cache.stats();
    assert_eq!(stats.resource_initialized_function_checks, 2);
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_stores,
        2
    );
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_hits,
        0
    );
    assert_eq!(
        stats.resource_summary_value_initialized_function_check_plan_skip_functions,
        0
    );
}

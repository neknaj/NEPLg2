use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::{FileId, Span};
use crate::types::{TypeCtx, TypeId};

use super::super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
};
use super::super::model::{
    EffectOp, Place, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceId, ResourceLocal, ResourceModule, ResourceOp, ResourceTerminator,
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

fn raw_init_leaf_function(
    types: &TypeCtx,
    name: &str,
    param_ty: TypeId,
    body_variant: bool,
) -> ResourceFunction {
    let span = test_span();
    let param = Place::local(String::from("raw"), param_ty);
    let ops = if body_variant {
        vec![ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(1),
            output: Place::temporary(ResourceId(0), types.i32()),
            ty: types.i32(),
            span,
        }]
    } else {
        Vec::new()
    };
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("raw"),
            ty: param_ty,
            mutable: false,
            place: param,
        }],
        result: types.unit(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops,
            terminator: ResourceTerminator::Return { value: None, span },
            span,
        }],
        span,
    }
}

fn raw_init_leaf_summary(ty: TypeId) -> RawCellInitializationFunctionSummary {
    raw_init_leaf_summary_for("raw_init_leaf", ty)
}

fn raw_init_leaf_summary_for(name: &str, ty: TypeId) -> RawCellInitializationFunctionSummary {
    RawCellInitializationFunctionSummary {
        function: String::from(name),
        type_params: Vec::new(),
        return_cells: Vec::new(),
        return_byte_ranges: Vec::new(),
        param_cells: vec![RawCellInitializationParamCell {
            param_index: 0,
            suffix: Vec::new(),
            ty,
            holds_raw_address: false,
        }],
        param_byte_ranges: Vec::new(),
        param_release_requirements: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_param_byte_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    }
}

fn raw_init_dependent_function(
    types: &TypeCtx,
    name: &str,
    dependency: &str,
    param_ty: TypeId,
) -> ResourceFunction {
    let span = test_span();
    let param = Place::local(String::from("raw"), param_ty);
    ResourceFunction {
        name: String::from(name),
        origin_name: String::from(name),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("raw"),
            ty: param_ty,
            mutable: false,
            place: param.clone(),
        }],
        result: types.unit(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::Call {
                output: Place::temporary(ResourceId(10), types.unit()),
                target: ResourceCallTarget::User {
                    name: String::from(dependency),
                    type_args: Vec::new(),
                },
                args: vec![param],
                effect: EffectOp::Pure,
                span,
            }],
            terminator: ResourceTerminator::Return { value: None, span },
            span,
        }],
        span,
    }
}

fn raw_init_leaf_module(function: ResourceFunction) -> ResourceModule {
    ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: Vec::new(),
    }
}

fn record_leaf_summary(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    summary: &RawCellInitializationFunctionSummary,
) {
    let dependencies = build_function_summary_dependencies(module);
    let preseeded_functions = vec![false; module.functions.len()];
    record_raw_cell_initialization_summary_value_cache_candidates(
        cache,
        context,
        types,
        module,
        &dependencies,
        &preseeded_functions,
        core::slice::from_ref(summary),
    );
}

fn preseed_leaf_summaries(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
) -> (
    Vec<bool>,
    Vec<bool>,
    Vec<RawCellInitializationFunctionSummary>,
) {
    let dependencies = build_function_summary_dependencies(module);
    let relevant_functions = vec![true; module.functions.len()];
    let mut worklist_relevant_functions = relevant_functions.clone();
    let mut preseeded_functions = vec![false; module.functions.len()];
    let mut summaries = Vec::new();
    preseed_raw_cell_initialization_summaries_from_value_cache(
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

/// raw-init param facts cache は、保存済み leaf entry を fixed-point worklist の前へ
/// 同じ summary surface として戻せる場合だけ対象関数を skip する。
#[test]
fn raw_init_param_facts_preseed_replays_same_summary_surface() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let module = raw_init_leaf_module(raw_init_leaf_function(
        &types,
        "raw_init_leaf",
        types.i32(),
        false,
    ));
    let summary = raw_init_leaf_summary(types.i32());
    let context = test_context(11);
    record_leaf_summary(&mut cache, &context, &types, &module, &summary);

    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_leaf_summaries(&mut cache, &context, &types, &module);

    assert_eq!(worklist_relevant_functions, vec![false]);
    assert_eq!(preseeded_functions, vec![true]);
    assert_eq!(summaries, vec![summary]);
    let stats = cache.stats();
    assert_eq!(stats.resource_summary_value_raw_init_param_facts_stores, 1);
    assert_eq!(stats.resource_summary_value_replay_hits, 1);
    assert_eq!(stats.resource_summary_value_replayed_ops, 1);
}

/// raw-init preseed の key は function body、source policy、signature type boundary を
/// すべて含む。いずれかが変わる場合は保存済み summary を使わず、通常 worklist に戻す。
#[test]
fn raw_init_param_facts_preseed_misses_on_body_source_or_signature_change() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let base_module = raw_init_leaf_module(raw_init_leaf_function(
        &types,
        "raw_init_leaf",
        types.i32(),
        false,
    ));
    let summary = raw_init_leaf_summary(types.i32());
    let context = test_context(11);
    record_leaf_summary(&mut cache, &context, &types, &base_module, &summary);

    let body_changed = raw_init_leaf_module(raw_init_leaf_function(
        &types,
        "raw_init_leaf",
        types.i32(),
        true,
    ));
    let source_changed = test_context(12);
    let signature_changed = raw_init_leaf_module(raw_init_leaf_function(
        &types,
        "raw_init_leaf",
        types.u8(),
        false,
    ));

    for (module, context) in [
        (&body_changed, &context),
        (&base_module, &source_changed),
        (&signature_changed, &context),
    ] {
        let (worklist_relevant_functions, preseeded_functions, summaries) =
            preseed_leaf_summaries(&mut cache, context, &types, module);
        assert_eq!(worklist_relevant_functions, vec![true]);
        assert_eq!(preseeded_functions, vec![false]);
        assert!(summaries.is_empty());
    }

    let stats = cache.stats();
    assert_eq!(stats.resource_summary_value_replay_hits, 0);
    assert_eq!(stats.resource_summary_value_replayed_ops, 0);
}

/// raw-init param facts cache は依存先 summary を取り込む関数も保存できるが、key には
/// dependency closure の body / source policy / type boundary hash を入れる。これにより
/// caller body が同じでも callee implementation edit 後は stale replay せず通常 worklist
/// に戻る。
#[test]
fn raw_init_param_facts_dependency_closure_invalidates_on_callee_body_change() {
    let mut cache = ResourceSummaryValueCache::new();
    let types = TypeCtx::new();
    let callee = raw_init_leaf_function(&types, "callee", types.i32(), false);
    let caller = raw_init_dependent_function(&types, "caller", "callee", types.i32());
    let module = ResourceModule {
        functions: vec![callee, caller],
        entry: None,
        string_literals: Vec::new(),
    };
    let context = test_context(11);
    let summary = raw_init_leaf_summary_for("caller", types.i32());
    record_leaf_summary(&mut cache, &context, &types, &module, &summary);

    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_leaf_summaries(&mut cache, &context, &types, &module);
    assert_eq!(worklist_relevant_functions, vec![true, false]);
    assert_eq!(preseeded_functions, vec![false, true]);
    assert_eq!(summaries, vec![summary]);

    let changed_callee = raw_init_leaf_function(&types, "callee", types.i32(), true);
    let changed_module = ResourceModule {
        functions: vec![
            changed_callee,
            raw_init_dependent_function(&types, "caller", "callee", types.i32()),
        ],
        entry: None,
        string_literals: Vec::new(),
    };
    let (worklist_relevant_functions, preseeded_functions, summaries) =
        preseed_leaf_summaries(&mut cache, &context, &types, &changed_module);

    assert_eq!(worklist_relevant_functions, vec![true, true]);
    assert_eq!(preseeded_functions, vec![false, false]);
    assert!(summaries.is_empty());
}

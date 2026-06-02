use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::report::ResourceOwnerFunctionCheck;
use super::resource_summary_value_cache::{
    owner_obligation_check_dependency_closure_hash, ResourceSummaryDependencyClosureHash,
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};

pub(super) struct OwnerObligationCheckCacheInput {
    pub(super) type_params: Vec<TypeId>,
    pub(super) dependency_closure_hash: ResourceSummaryDependencyClosureHash,
}

/// owner obligation check replay 用の stable key 入力を作る。
///
/// owner obligation は callee の owner return summary を参照するため、caller 本文だけでなく
/// dependency closure を key に含める。closure hash が作れない場合は古い pass を使わず、
/// 通常 checker へ戻して no-store 理由を counter に残す。
pub(super) fn owner_obligation_check_cache_input(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: Option<&[Vec<usize>]>,
    function_index: usize,
    function: &ResourceFunction,
    function_op_count: usize,
) -> Option<OwnerObligationCheckCacheInput> {
    let (cache, context, dependencies) = (cache?, context?, dependencies?);
    match owner_obligation_check_dependency_closure_hash(
        context,
        types,
        module,
        dependencies,
        function_index,
    ) {
        Ok(dependency_closure_hash) => Some(OwnerObligationCheckCacheInput {
            type_params: owner_summary_type_params(types, function),
            dependency_closure_hash,
        }),
        Err(reason) => {
            cache
                .record_owner_obligation_check_dependency_closure_bypass(reason, function_op_count);
            None
        }
    }
}

/// cache hit した owner obligation check を診断なし pass として戻す。
///
/// hit した場合は `ResourceOwnerCheckEngine::check_function` を起動せず、後続 gate が使う
/// diagnostic-free surface だけを返す。owner state は session-local な検査内部状態なので、
/// pass-only replay では materialize しない。
pub(super) fn replay_owner_obligation_check_from_value_cache(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    function: &ResourceFunction,
    input: Option<&OwnerObligationCheckCacheInput>,
    function_op_count: usize,
) -> Option<ResourceOwnerFunctionCheck> {
    let (cache, context, input) = (cache?, context?, input?);
    cache.replay_owner_obligation_check_entry_pass(
        context,
        types,
        function,
        &input.type_params,
        input.dependency_closure_hash,
        function_op_count,
    )
}

/// 現行 owner checker が作った diagnostic-free result を stable cache 候補として保存する。
///
/// diagnostics がある関数は replay で現在 source に対する診断生成を隠してしまうため保存しない。
/// diagnostic-free 関数だけを pass entry として保存し、微小編集時の owner obligation 固定費を
/// 変更された関数とその依存 closure に閉じ込める。
pub(super) fn record_owner_obligation_check_value_cache_candidate(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    function: &ResourceFunction,
    input: Option<&OwnerObligationCheckCacheInput>,
    function_check: &ResourceOwnerFunctionCheck,
    function_has_diagnostics: bool,
    function_op_count: usize,
) {
    let (cache, context, input) = match (cache, context, input) {
        (Some(cache), Some(context), Some(input)) => (cache, context, input),
        _ => return,
    };
    if !cache.stable_entry_collection_enabled() {
        return;
    }
    if function_has_diagnostics {
        cache.record_owner_obligation_check_diagnostic_bypass(function_op_count);
        return;
    }
    match cache.owner_obligation_check_entry_candidate(
        context,
        types,
        function,
        &input.type_params,
        input.dependency_closure_hash,
        function_check,
        function_op_count,
    ) {
        Ok(candidate) => {
            cache.record_owner_obligation_check_entry_candidates(vec![candidate]);
        }
        Err(reason) => {
            cache.record_owner_obligation_check_candidate_bypass(reason, function_op_count);
        }
    }
}

use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::report::ResourceFunctionCheck;
use super::resource_summary_value_cache::{
    initialized_function_check_dependency_closure_hash, ResourceSummaryDependencyClosureHash,
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};

pub(super) struct InitializedFunctionCheckCacheInput {
    pub(super) type_params: Vec<TypeId>,
    pub(super) dependency_closure_hash: ResourceSummaryDependencyClosureHash,
}

/// final initialized function check replay 用の stable key 入力を作る。
///
/// dependency closure hash が作れない場合は、古い check result を使わず通常 checker へ
/// 戻る。bypass 理由は cache counter に残し、次の性能分解で dependency graph / source
/// policy / type boundary のどこが不安定かを追えるようにする。
pub(super) fn initialized_function_check_cache_input(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: Option<&[Vec<usize>]>,
    function_index: usize,
    function: &ResourceFunction,
    function_op_count: usize,
) -> Option<InitializedFunctionCheckCacheInput> {
    let (cache, context, dependencies) = (cache?, context?, dependencies?);
    match initialized_function_check_dependency_closure_hash(
        context,
        types,
        module,
        dependencies,
        function_index,
    ) {
        Ok(dependency_closure_hash) => Some(InitializedFunctionCheckCacheInput {
            type_params: owner_summary_type_params(types, function),
            dependency_closure_hash,
        }),
        Err(reason) => {
            cache.record_initialized_function_check_dependency_closure_bypass(
                reason,
                function_op_count,
            );
            None
        }
    }
}

/// cache hit した final function check を現在 compile の型と place id へ戻す。
///
/// hit した場合は caller が `ResourceCheckEngine::check_function` を起動せず、
/// diagnostic-free / auto-drop-free な checked pass として report へ追加する。後続の
/// cell gate は diagnostics、drop elaboration は auto-drop point だけを消費するため、
/// cache hit 時に final cell / collection slot state を materialize しない。key が作れない
/// 場合や entry がない場合は `None` を返し、既存 checker に authority を戻す。
pub(super) fn replay_initialized_function_check_from_value_cache(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    function: &ResourceFunction,
    input: Option<&InitializedFunctionCheckCacheInput>,
    function_op_count: usize,
) -> Option<ResourceFunctionCheck> {
    let (cache, context, input) = (cache?, context?, input?);
    cache.replay_initialized_function_check_entry_pass(
        context,
        types,
        function,
        &input.type_params,
        input.dependency_closure_hash,
        function_op_count,
    )
}

/// 現行 checker が作った function check を stable cache 候補として保存する。
///
/// diagnostics と auto drop point は span を含むため、この境界では保存しない。
/// diagnostics がある関数は replay で新しい source 位置を隠さないよう no-store にし、
/// auto drop point がある関数は stable mirror 側の `AutoDropPoints` reject として
/// drop elaboration 用 span の再束縛設計が入るまで no-store にする。
pub(super) fn record_initialized_function_check_value_cache_candidate(
    cache: Option<&mut ResourceSummaryValueCache>,
    context: Option<&ResourceSummaryValueCacheContext>,
    types: &TypeCtx,
    function: &ResourceFunction,
    input: Option<&InitializedFunctionCheckCacheInput>,
    function_check: &ResourceFunctionCheck,
    function_has_diagnostics: bool,
    function_op_count: usize,
) {
    let (cache, context, input) = match (cache, context, input) {
        (Some(cache), Some(context), Some(input)) => (cache, context, input),
        _ => return,
    };
    if function_has_diagnostics {
        cache.record_initialized_function_check_diagnostic_bypass(function_op_count);
        return;
    }
    match cache.initialized_function_check_entry_candidate(
        context,
        types,
        function,
        &input.type_params,
        input.dependency_closure_hash,
        function_check,
        function_op_count,
    ) {
        Ok(candidate) => {
            cache.record_initialized_function_check_entry_candidates(vec![candidate]);
        }
        Err(reason) => {
            cache.record_initialized_function_check_candidate_bypass(reason, function_op_count);
        }
    }
}

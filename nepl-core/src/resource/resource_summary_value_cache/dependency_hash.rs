extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::super::model::ResourceModule;
use super::super::owner_summary_type_params::owner_summary_type_params;
use super::body_hash::resource_function_body_hash;
use super::candidate_key::ResourceSummaryDependencyClosureHash;
use super::context::ResourceSummaryValueCacheContext;
use super::key::ResourceSummaryFunctionIdentity;
use super::stable_hash::ResourceSummaryStableHasher;
use super::type_boundary::resource_summary_type_parameter_boundary_hash;

/// raw-init summary value key に含める dependency closure hash を作る。
///
/// raw initialization summary は user call の callee summary を本文解析中に取り込むため、
/// caller body だけを key にすると callee implementation edit 後に stale hit し得る。
/// この hash は direct dependency だけでなく reachable dependency closure の body hash、
/// source capability policy、function-local type boundary をまとめ、依存先の証明入力が
/// 変わったとき caller の cached raw-init facts も miss するようにする。
pub(in crate::resource) fn raw_init_dependency_closure_hash(
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Option<ResourceSummaryDependencyClosureHash> {
    let mut reachable = BTreeSet::new();
    collect_dependency_closure(dependencies, function_index, &mut reachable)?;

    let mut hash =
        ResourceSummaryStableHasher::new("neplg2-resource-summary-raw-init-dependency-closure-v1");
    hash.write_usize(reachable.len());
    for dependency_index in reachable {
        let dependency = module.functions.get(dependency_index)?;
        let identity = ResourceSummaryFunctionIdentity::from_resource_function(dependency)?;
        identity.write_stable(&mut hash);
        hash.write_u64(resource_function_body_hash(types, dependency)?);
        hash.write_u64(
            context
                .source_capability_policy_hash_for_function(dependency)?
                .as_u64(),
        );
        let type_params = owner_summary_type_params(types, dependency);
        hash.write_u64(resource_summary_type_parameter_boundary_hash(
            types,
            &type_params,
        )?);
    }

    Some(ResourceSummaryDependencyClosureHash::from_stable_hash(
        hash.finish(),
    ))
}

fn collect_dependency_closure(
    dependencies: &[Vec<usize>],
    function_index: usize,
    out: &mut BTreeSet<usize>,
) -> Option<()> {
    for dependency in dependencies.get(function_index)? {
        if out.insert(*dependency) {
            collect_dependency_closure(dependencies, *dependency, out)?;
        }
    }
    Some(())
}

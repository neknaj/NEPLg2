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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryDependencyClosureHashReject {
    DependencyGraph,
    DependencyFunctionIdentity,
    DependencyFunctionBody,
    DependencySourcePolicy,
    DependencyTypeBoundary,
}

pub(in crate::resource) type RawInitDependencyClosureHashReject =
    ResourceSummaryDependencyClosureHashReject;

pub(in crate::resource) type RawAliasDependencyClosureHashReject =
    ResourceSummaryDependencyClosureHashReject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryDependencyClosureKind {
    RawAlias,
    RawInit,
    I32Scalar,
    InitializedFunctionCheck,
}

/// summary value key に含める dependency closure hash を作る。
///
/// raw initialization summary や i32 scalar summary は user call の callee summary を
/// 本文解析中に取り込むため、caller body だけを key にすると callee implementation
/// edit 後に stale hit し得る。この hash は direct dependency だけでなく reachable
/// dependency closure の body hash、source capability policy、function-local type
/// boundary をまとめ、依存先の証明入力が変わったとき caller の cached summary facts も
/// miss するようにする。
///
/// 失敗時は、広い `unstable key` ではなく dependency graph、function identity、body
/// hash、source policy、type boundary のどれが欠けたかを返す。性能改善では残った探索
/// 空間を定量的に潰す必要があるため、no-store の理由も静的に列挙する。
pub(in crate::resource) fn resource_summary_dependency_closure_hash(
    kind: ResourceSummaryDependencyClosureKind,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Result<ResourceSummaryDependencyClosureHash, ResourceSummaryDependencyClosureHashReject> {
    let mut reachable = BTreeSet::new();
    collect_dependency_closure(dependencies, function_index, &mut reachable)?;

    let mut hash =
        ResourceSummaryStableHasher::new("neplg2-resource-summary-dependency-closure-v1");
    hash.write_str(kind.tag());
    hash.write_usize(reachable.len());
    for dependency_index in reachable {
        let dependency = module
            .functions
            .get(dependency_index)
            .ok_or(ResourceSummaryDependencyClosureHashReject::DependencyGraph)?;
        let identity = ResourceSummaryFunctionIdentity::from_resource_function(dependency)
            .ok_or(ResourceSummaryDependencyClosureHashReject::DependencyFunctionIdentity)?;
        identity.write_stable(&mut hash);
        hash.write_u64(
            resource_function_body_hash(types, dependency)
                .ok_or(ResourceSummaryDependencyClosureHashReject::DependencyFunctionBody)?,
        );
        hash.write_u64(
            context
                .source_capability_policy_hash_for_function(dependency)
                .ok_or(ResourceSummaryDependencyClosureHashReject::DependencySourcePolicy)?
                .as_u64(),
        );
        let type_params = owner_summary_type_params(types, dependency);
        hash.write_u64(
            resource_summary_type_parameter_boundary_hash(types, &type_params)
                .ok_or(ResourceSummaryDependencyClosureHashReject::DependencyTypeBoundary)?,
        );
    }

    Ok(ResourceSummaryDependencyClosureHash::from_stable_hash(
        hash.finish(),
    ))
}

pub(in crate::resource) fn raw_init_dependency_closure_hash(
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Result<ResourceSummaryDependencyClosureHash, RawInitDependencyClosureHashReject> {
    resource_summary_dependency_closure_hash(
        ResourceSummaryDependencyClosureKind::RawInit,
        context,
        types,
        module,
        dependencies,
        function_index,
    )
}

pub(in crate::resource) fn raw_alias_dependency_closure_hash(
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Result<ResourceSummaryDependencyClosureHash, RawAliasDependencyClosureHashReject> {
    resource_summary_dependency_closure_hash(
        ResourceSummaryDependencyClosureKind::RawAlias,
        context,
        types,
        module,
        dependencies,
        function_index,
    )
}

pub(in crate::resource) fn i32_scalar_dependency_closure_hash(
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Result<ResourceSummaryDependencyClosureHash, ResourceSummaryDependencyClosureHashReject> {
    resource_summary_dependency_closure_hash(
        ResourceSummaryDependencyClosureKind::I32Scalar,
        context,
        types,
        module,
        dependencies,
        function_index,
    )
}

pub(in crate::resource) fn initialized_function_check_dependency_closure_hash(
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    function_index: usize,
) -> Result<ResourceSummaryDependencyClosureHash, ResourceSummaryDependencyClosureHashReject> {
    resource_summary_dependency_closure_hash(
        ResourceSummaryDependencyClosureKind::InitializedFunctionCheck,
        context,
        types,
        module,
        dependencies,
        function_index,
    )
}

fn collect_dependency_closure(
    dependencies: &[Vec<usize>],
    function_index: usize,
    out: &mut BTreeSet<usize>,
) -> Result<(), ResourceSummaryDependencyClosureHashReject> {
    for dependency in dependencies
        .get(function_index)
        .ok_or(ResourceSummaryDependencyClosureHashReject::DependencyGraph)?
    {
        if out.insert(*dependency) {
            collect_dependency_closure(dependencies, *dependency, out)?;
        }
    }
    Ok(())
}

impl ResourceSummaryDependencyClosureKind {
    fn tag(self) -> &'static str {
        match self {
            ResourceSummaryDependencyClosureKind::RawAlias => "raw-alias",
            ResourceSummaryDependencyClosureKind::RawInit => "raw-init",
            ResourceSummaryDependencyClosureKind::I32Scalar => "i32-scalar",
            ResourceSummaryDependencyClosureKind::InitializedFunctionCheck => {
                "initialized-function-check"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::{FileId, Span};
    use crate::types::TypeId;

    use super::super::super::model::{
        RawBodyKind, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceTerminator,
    };
    use super::*;

    fn raw_body_dependency(types: &TypeCtx, kind: RawBodyKind, file: FileId) -> ResourceFunction {
        ResourceFunction {
            name: "raw_dep".to_string(),
            origin_name: "raw_dep".to_string(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.i32(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::RawBody {
                    kind,
                    span: Span::new(file, 10, 20),
                },
                span: Span::new(file, 0, 20),
            }],
            span: Span::new(file, 0, 20),
        }
    }

    fn caller(types: &TypeCtx, type_params: Vec<TypeId>) -> ResourceFunction {
        ResourceFunction {
            name: "caller".to_string(),
            origin_name: "caller".to_string(),
            type_params,
            params: Vec::new(),
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::new(FileId(0), 1, 2),
                },
                span: Span::new(FileId(0), 0, 2),
            }],
            span: Span::new(FileId(0), 0, 2),
        }
    }

    fn context(file: FileId, policy_hash: u64) -> ResourceSummaryValueCacheContext {
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(file, policy_hash);
        context
    }

    #[test]
    fn raw_init_dependency_closure_hash_accepts_raw_body_dependency_with_source_policy() {
        let types = TypeCtx::new();
        let module = ResourceModule {
            functions: vec![
                caller(&types, Vec::new()),
                raw_body_dependency(&types, RawBodyKind::Wasm, FileId(1)),
            ],
            entry: None,
            string_literals: Vec::new(),
        };
        let dependencies = vec![vec![1], Vec::new()];

        assert!(raw_init_dependency_closure_hash(
            &context(FileId(1), 100),
            &types,
            &module,
            &dependencies,
            0,
        )
        .is_ok());
    }

    #[test]
    fn raw_init_dependency_closure_hash_tracks_dependency_source_policy() {
        let types = TypeCtx::new();
        let module = ResourceModule {
            functions: vec![
                caller(&types, Vec::new()),
                raw_body_dependency(&types, RawBodyKind::Wasm, FileId(1)),
            ],
            entry: None,
            string_literals: Vec::new(),
        };
        let dependencies = vec![vec![1], Vec::new()];

        let first = raw_init_dependency_closure_hash(
            &context(FileId(1), 100),
            &types,
            &module,
            &dependencies,
            0,
        )
        .expect("source policy should make raw body dependency keyable");
        let second = raw_init_dependency_closure_hash(
            &context(FileId(1), 101),
            &types,
            &module,
            &dependencies,
            0,
        )
        .expect("edited source policy should still be keyable");

        assert_ne!(first, second);
    }

    #[test]
    fn raw_init_dependency_closure_hash_reports_missing_dependency_source_policy() {
        let types = TypeCtx::new();
        let module = ResourceModule {
            functions: vec![
                caller(&types, Vec::new()),
                raw_body_dependency(&types, RawBodyKind::Wasm, FileId(1)),
            ],
            entry: None,
            string_literals: Vec::new(),
        };
        let dependencies = vec![vec![1], Vec::new()];

        assert_eq!(
            raw_init_dependency_closure_hash(
                &context(FileId(2), 100),
                &types,
                &module,
                &dependencies,
                0,
            ),
            Err(RawInitDependencyClosureHashReject::DependencySourcePolicy)
        );
    }

    #[test]
    fn raw_init_dependency_closure_hash_reports_dependency_type_boundary_failure() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        types
            .unify(generic, types.i32())
            .expect("test setup should bind the dependency type parameter");
        let module = ResourceModule {
            functions: vec![caller(&types, Vec::new()), caller(&types, vec![generic])],
            entry: None,
            string_literals: Vec::new(),
        };
        let dependencies = vec![vec![1], Vec::new()];

        assert_eq!(
            raw_init_dependency_closure_hash(
                &context(FileId(0), 100),
                &types,
                &module,
                &dependencies,
                0,
            ),
            Err(RawInitDependencyClosureHashReject::DependencyTypeBoundary)
        );
    }
}

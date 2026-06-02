use crate::types::{TypeCtx, TypeId};

use super::super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::super::model::ResourceFunction;
use super::body_hash::resource_function_body_hash;
use super::key::{ResourceSummaryFunctionIdentity, ResourceSummaryValueCacheKey};
use super::stable_mirror::{
    stable_drop_traversal_forall_leaf_entry, ResourceSummaryStableDropTraversalForallLeafEntry,
};
use super::type_boundary::{
    resource_summary_generic_type_argument_hash, resource_summary_type_parameter_boundary_hash,
};

// この module は Resource summary value cache に保存してよい候補だけを作る境界である。
// namespace や source capability policy に仮値を入れず、key と stable value の両方を
// 作れる場合だけ store/hit の対象へ渡す。

/// `ResourceSummaryCacheNamespaceKey::stable_hash` として作られた hash。
///
/// candidate builder は複数種類の `u64` hash を同時に扱うため、裸の `u64` を直接渡さず
/// 入力の意味を型名で分ける。値の正当性は caller が namespace key を構築した時点で
/// 保証するため、この wrapper は追加の sentinel 判定を行わない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ResourceSummaryCacheNamespaceHash(u64);

/// `SourceMap::source_capability_policy_hash_for_file` から作られた source policy hash。
///
/// source capability proof がある file では、現在の source path、source hash、proof set に
/// 結び付いた policy hash だけを key に入れる。proof が空の file では、Resource IR body
/// hash が function semantics を固定するため、path と空 proof set の hash を key に入れる。
/// 未計算の状態を `0` などで表さず、caller がこの wrapper を作れない場合は candidate
/// builder を呼ばない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ResourceSummarySourceCapabilityPolicyHash(u64);

/// raw-init summary が依存する user function closure の stable invalidation hash。
///
/// raw-init param facts は call 境界で callee summary を取り込むことがある。依存関係を
/// 持つ関数を cache する場合、この hash を key に含め、依存先の body / source policy /
/// signature boundary が変わったら caller summary も必ず miss させる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) struct ResourceSummaryDependencyClosureHash(u64);

/// generic type argument を key に入れる方法。
///
/// 現在の collection slot summary は、monomorphize 後の function body と
/// function-local type parameter boundary は保持するが、元の call-site generic
/// argument list は保持しない。そのため空 slice をそのまま受け取ると、「非 generic」
/// 「generic template」「実引数を取り忘れた generic instantiation」が区別できない。
/// この enum で caller が意味を明示し、未知の実引数を empty hash として扱わない。
pub(super) enum ResourceSummaryGenericTypeArgumentKeyInput<'a> {
    NonGeneric,
    TemplateBoundaryOnly,
    /// 将来、call-site の concrete generic argument list を保存できるようになった段階で
    /// instantiated summary key に使う。現 MVP は summary 側に実引数 list がないため、
    /// production path ではまだ構築しない。
    #[allow(dead_code)]
    KnownInstantiation(&'a [TypeId]),
}

impl ResourceSummaryCacheNamespaceHash {
    pub(super) fn from_stable_hash(value: u64) -> Self {
        Self(value)
    }

    pub(super) fn as_u64(self) -> u64 {
        self.0
    }
}

impl ResourceSummarySourceCapabilityPolicyHash {
    pub(super) fn from_stable_hash(value: u64) -> Self {
        Self(value)
    }

    pub(super) fn as_u64(self) -> u64 {
        self.0
    }
}

impl ResourceSummaryDependencyClosureHash {
    pub(in crate::resource) fn from_stable_hash(value: u64) -> Self {
        Self(value)
    }

    pub(in crate::resource) fn as_u64(self) -> u64 {
        self.0
    }
}

/// `DropTraversal + ForallInitializedRange` の per-value cache key を作る。
///
/// この builder は stable mirror value、function identity、function body hash、
/// function-local type parameter boundary、generic type argument、source capability policy、
/// namespace をすべて揃えられる場合だけ `Some` を返す。generic argument が空でよいのは
/// caller が `NonGeneric` または `TemplateBoundaryOnly` を明示した場合だけである。generic
/// instantiation の実引数を取得できない場合は、空 slice ではなく no-store 候補へ倒す。
#[cfg(test)]
pub(super) fn drop_traversal_forall_candidate_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
    op: &CollectionSlotLifecycleSummaryOp,
) -> Option<ResourceSummaryValueCacheKey> {
    drop_traversal_forall_leaf_entry_candidate_key_and_entry(
        types,
        namespace_hash,
        source_capability_policy_hash,
        function,
        type_params,
        generic_type_args,
        core::slice::from_ref(op),
    )
    .map(|(key, _)| key)
}

pub(super) fn drop_traversal_forall_leaf_entry_candidate_key_and_entry(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
    ops: &[CollectionSlotLifecycleSummaryOp],
) -> Option<(
    ResourceSummaryValueCacheKey,
    ResourceSummaryStableDropTraversalForallLeafEntry,
)> {
    let key = drop_traversal_forall_leaf_entry_key(
        types,
        namespace_hash,
        source_capability_policy_hash,
        function,
        type_params,
        generic_type_args,
    )?;
    let stable_entry = stable_drop_traversal_forall_leaf_entry(types, ops)?;

    Some((key, stable_entry))
}

pub(super) fn drop_traversal_forall_leaf_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(
        ResourceSummaryValueCacheKey::new_drop_traversal_forall_leaf_entry(
            namespace_hash.as_u64(),
            function_identity,
            function_body_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash.as_u64(),
        ),
    )
}

pub(super) fn raw_init_complete_leaf_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(
        ResourceSummaryValueCacheKey::new_raw_init_complete_leaf_entry(
            namespace_hash.as_u64(),
            function_identity,
            function_body_hash,
            dependency_closure_hash.as_u64(),
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash.as_u64(),
        ),
    )
}

pub(super) fn raw_alias_return_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(ResourceSummaryValueCacheKey::new_raw_alias_return_entry(
        namespace_hash.as_u64(),
        function_identity,
        function_body_hash,
        dependency_closure_hash.as_u64(),
        type_parameter_boundary_hash,
        generic_type_argument_hash,
        source_capability_policy_hash.as_u64(),
    ))
}

pub(super) fn i32_scalar_return_facts_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(
        ResourceSummaryValueCacheKey::new_i32_scalar_return_facts_entry(
            namespace_hash.as_u64(),
            function_identity,
            function_body_hash,
            dependency_closure_hash.as_u64(),
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash.as_u64(),
        ),
    )
}

pub(super) fn initialized_function_check_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(
        ResourceSummaryValueCacheKey::new_initialized_function_check_entry(
            namespace_hash.as_u64(),
            function_identity,
            function_body_hash,
            dependency_closure_hash.as_u64(),
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash.as_u64(),
        ),
    )
}

pub(super) fn owner_obligation_check_entry_key(
    types: &TypeCtx,
    namespace_hash: ResourceSummaryCacheNamespaceHash,
    source_capability_policy_hash: ResourceSummarySourceCapabilityPolicyHash,
    dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    function: &ResourceFunction,
    type_params: &[TypeId],
    generic_type_args: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<ResourceSummaryValueCacheKey> {
    let function_identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
    let function_body_hash = resource_function_body_hash(types, function)?;
    let type_parameter_boundary_hash =
        resource_summary_type_parameter_boundary_hash(types, type_params)?;
    let generic_type_argument_hash =
        generic_type_argument_key_hash(types, function, type_params, generic_type_args)?;

    Some(
        ResourceSummaryValueCacheKey::new_owner_obligation_check_entry(
            namespace_hash.as_u64(),
            function_identity,
            function_body_hash,
            dependency_closure_hash.as_u64(),
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash.as_u64(),
        ),
    )
}

fn generic_type_argument_key_hash(
    types: &TypeCtx,
    function: &ResourceFunction,
    type_params: &[TypeId],
    input: ResourceSummaryGenericTypeArgumentKeyInput<'_>,
) -> Option<u64> {
    match input {
        ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric => {
            if !function.type_params.is_empty() || !type_params.is_empty() {
                return None;
            }
            resource_summary_generic_type_argument_hash(types, &[])
        }
        ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly => {
            if function.type_params.is_empty() && type_params.is_empty() {
                return None;
            }
            resource_summary_generic_type_argument_hash(types, &[])
        }
        ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(args) => {
            if function.type_params.is_empty() || args.len() != function.type_params.len() {
                return None;
            }
            resource_summary_generic_type_argument_hash(types, args)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::{NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeKind};

    use super::super::super::collection_slot_summary_model::{
        CollectionSlotInitializedRangeDropTraversalCertificate,
        CollectionSlotInitializedRangeDropTraversalProof,
        CollectionSlotLifecycleSummaryDropTraversalCoverage,
        CollectionSlotLifecycleSummaryI32Operand, CollectionSlotLifecycleSummaryOp,
    };
    use super::super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceExprKind, ResourceFunction, ResourceId,
        ResourceOp, ResourceTerminator,
    };
    use super::super::super::summary_projection::SummaryPlace;
    use super::*;

    fn namespace(value: u64) -> ResourceSummaryCacheNamespaceHash {
        ResourceSummaryCacheNamespaceHash::from_stable_hash(value)
    }

    fn source_policy(value: u64) -> ResourceSummarySourceCapabilityPolicyHash {
        ResourceSummarySourceCapabilityPolicyHash::from_stable_hash(value)
    }

    fn simple_function(types: &TypeCtx, literal: i32) -> ResourceFunction {
        function_with_type_params(types, Vec::new(), literal, "example")
    }

    fn generic_function(types: &TypeCtx, type_param: TypeId) -> ResourceFunction {
        function_with_type_params(types, vec![type_param], 1, "generic")
    }

    fn function_with_type_params(
        types: &TypeCtx,
        type_params: Vec<TypeId>,
        literal: i32,
        name: &str,
    ) -> ResourceFunction {
        let ty = types.i32();
        let output = Place::temporary(ResourceId(0), ty);
        ResourceFunction {
            name: name.to_string(),
            origin_name: name.to_string(),
            type_params,
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(literal),
                    output: output.clone(),
                    ty,
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn forall_drop_traversal_op(types: &TypeCtx) -> CollectionSlotLifecycleSummaryOp {
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        }
    }

    fn certified_drop_traversal_op(types: &TypeCtx) -> CollectionSlotLifecycleSummaryOp {
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                Vec::new(),
            ),
        }
    }

    #[test]
    fn candidate_key_builds_for_non_generic_forall_drop_traversal() {
        let types = TypeCtx::new();
        let function = simple_function(&types, 1);
        let op = forall_drop_traversal_op(&types);

        let key = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        );

        assert!(key.is_some());
    }

    #[test]
    fn candidate_key_tracks_invalidation_inputs() {
        let types = TypeCtx::new();
        let first_function = simple_function(&types, 1);
        let second_function = simple_function(&types, 2);
        let op = forall_drop_traversal_op(&types);
        let base = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &first_function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .expect("stable non-generic candidate should build");

        let namespace_edit = drop_traversal_forall_candidate_key(
            &types,
            namespace(11),
            source_policy(20),
            &first_function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .expect("namespace edit should still be keyable");
        let source_policy_edit = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(21),
            &first_function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .expect("source policy edit should still be keyable");
        let body_edit = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &second_function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .expect("body edit should still be keyable");

        assert_ne!(base, namespace_edit);
        assert_ne!(base, source_policy_edit);
        assert_ne!(base, body_edit);
    }

    #[test]
    fn candidate_key_tracks_generic_arguments_and_type_boundary() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let generic_with_cap = types.fresh_var(Some("T".to_string()));
        types.set_var_capabilities(generic_with_cap, true, false, true);
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        let i32_key = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[types.i32()]),
            &op,
        )
        .expect("generic candidate with known i32 argument should build");
        let bool_key = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[types.bool()]),
            &op,
        )
        .expect("generic candidate with known bool argument should build");
        let type_boundary_key = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic_with_cap],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[types.i32()]),
            &op,
        )
        .expect("generic candidate with different parameter capability should build");

        assert_ne!(i32_key, bool_key);
        assert_ne!(i32_key, type_boundary_key);
    }

    #[test]
    fn candidate_key_rejects_missing_generic_arguments() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[]),
            &op,
        )
        .is_none());
    }

    #[test]
    fn candidate_key_builds_for_template_boundary_without_concrete_arguments() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        let key = drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly,
            &op,
        );

        assert!(key.is_some());
    }

    #[test]
    fn candidate_key_rejects_non_generic_input_with_generic_boundary() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .is_none());
    }

    #[test]
    fn candidate_key_rejects_unstable_generic_arguments() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let nominal = types.register_named(
            "Nominal".to_string(),
            TypeKind::Named("Nominal".to_string()),
        );
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[nominal]),
            &op,
        )
        .is_none());
    }

    #[test]
    fn candidate_key_accepts_resolved_nominal_generic_arguments() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let payload = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Nominal".to_string(),
            TypeKind::Struct {
                name: "Nominal".to_string(),
                type_params: Vec::new(),
                fields: vec![payload],
                field_names: vec!["payload".to_string()],
            },
            NominalStableTypeIdentity::new(
                NominalStableTypeKind::Struct,
                "/user/types.nepl".to_string(),
                "Nominal".to_string(),
                0,
                1,
            ),
        );
        let function = generic_function(&types, generic);
        let op = forall_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[generic],
            ResourceSummaryGenericTypeArgumentKeyInput::KnownInstantiation(&[nominal]),
            &op,
        )
        .is_some());
    }

    #[test]
    fn candidate_key_rejects_non_forall_drop_traversal_value() {
        let types = TypeCtx::new();
        let function = simple_function(&types, 1);
        let op = certified_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .is_none());
    }

    #[test]
    fn candidate_key_rejects_empty_function_identity() {
        let types = TypeCtx::new();
        let mut function = simple_function(&types, 1);
        function.name.clear();
        let op = forall_drop_traversal_op(&types);

        assert!(drop_traversal_forall_candidate_key(
            &types,
            namespace(10),
            source_policy(20),
            &function,
            &[],
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric,
            &op,
        )
        .is_none());
    }
}

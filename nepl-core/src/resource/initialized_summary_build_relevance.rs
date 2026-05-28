extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_summary_seed::summary_input_type_may_seed_raw_address_alias;
use super::model::{ResourceFunction, ResourceModule};
use super::summary_dependency::build_function_summary_dependencies;

pub(super) fn raw_cell_initialization_summary_relevance(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
) -> Vec<bool> {
    let signature_relevant = module
        .functions
        .iter()
        .map(|function| function_raw_cell_initialization_signature_relevant(types, function))
        .collect::<Vec<_>>();
    let mut relevant = module
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            signature_relevant[index]
                && (raw_alias_summaries.get(&function.name).is_some()
                    || function_has_direct_raw_initialization_summary_op(function))
        })
        .collect::<Vec<_>>();
    let dependencies = build_function_summary_dependencies(module);
    let mut changed = true;
    while changed {
        changed = false;
        for (index, function_dependencies) in dependencies.iter().enumerate() {
            if relevant[index] || !signature_relevant[index] {
                continue;
            }
            if function_dependencies
                .iter()
                .any(|dependency| relevant[*dependency])
            {
                relevant[index] = true;
                changed = true;
            }
        }
    }
    relevant
}

fn function_raw_cell_initialization_signature_relevant(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    // raw initialization summary は raw address / byte-range / release requirement を
    // call 境界で受け渡すための要約である。`i32` は raw address の表現にも使われるが、
    // すべての整数関数を summary 対象にすると、普通の算術処理まで raw memory 解析へ
    // 巻き込んでしまう。まず公開可能な型を持つ関数だけを候補にし、実際の raw alias
    // summary・raw memory 系 op・関連 callee から到達できるものだけを計算対象にする。
    summary_input_type_may_seed_raw_address_alias(types, function.result)
        || function
            .params
            .iter()
            .any(|param| place_may_seed_raw_initialization_summary(types, &param.place))
}

fn place_may_seed_raw_initialization_summary(types: &TypeCtx, place: &super::model::Place) -> bool {
    summary_input_type_may_seed_raw_address_alias(types, place.ty)
        || reference_target_type(types, place.ty).is_some_and(|target_ty| {
            summary_input_type_may_seed_raw_address_alias(types, target_ty)
        })
}

pub(super) fn function_has_direct_raw_initialization_summary_op(
    function: &ResourceFunction,
) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_have_direct_raw_initialization_summary_op(&block.ops))
}

fn ops_have_direct_raw_initialization_summary_op(ops: &[super::model::ResourceOp]) -> bool {
    ops.iter().any(op_has_direct_raw_initialization_summary_op)
}

fn op_has_direct_raw_initialization_summary_op(op: &super::model::ResourceOp) -> bool {
    match op {
        // Collection slot ops are not raw-memory instructions themselves, but the
        // initialized-cell facts they create are consumed through the same raw init
        // summary boundary at call sites. Treat them as direct triggers so helper
        // functions without explicit RawMemory ops are not pruned out.
        super::model::ResourceOp::RawMemory { .. }
        | super::model::ResourceOp::RawAddressAlias { .. }
        | super::model::ResourceOp::RawAddressView { .. }
        | super::model::ResourceOp::StorageOrigin { .. }
        | super::model::ResourceOp::IndirectCall { .. }
        | super::model::ResourceOp::CollectionSlotLifecycle { .. }
        | super::model::ResourceOp::CollectionStorageRelocate { .. }
        | super::model::ResourceOp::CollectionSlotDropTraversal { .. }
        | super::model::ResourceOp::CollectionSlotTransformRange { .. } => true,
        super::model::ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_have_direct_raw_initialization_summary_op(then_ops)
                || ops_have_direct_raw_initialization_summary_op(else_ops)
        }
        super::model::ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_have_direct_raw_initialization_summary_op(condition_ops)
                || ops_have_direct_raw_initialization_summary_op(body_ops)
        }
        super::model::ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_have_direct_raw_initialization_summary_op(&arm.ops)),
        super::model::ResourceOp::Call { .. }
        | super::model::ResourceOp::Expr { .. }
        | super::model::ResourceOp::DeclareLocal { .. }
        | super::model::ResourceOp::Read { .. }
        | super::model::ResourceOp::Assign { .. }
        | super::model::ResourceOp::Borrow { .. }
        | super::model::ResourceOp::Move { .. }
        | super::model::ResourceOp::Drop { .. }
        | super::model::ResourceOp::EndScope { .. }
        | super::model::ResourceOp::CallEffect { .. }
        | super::model::ResourceOp::FunctionValue { .. }
        | super::model::ResourceOp::Construct { .. } => false,
    }
}

pub(super) fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

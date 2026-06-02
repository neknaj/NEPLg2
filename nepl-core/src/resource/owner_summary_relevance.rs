extern crate alloc;

use alloc::vec::Vec;

use super::model::{EffectOp, Place, ResourceFunction, ResourceModule, ResourceOp};
use super::owner_summary_leaf::owner_leaf_places;
use crate::types::TypeCtx;

/// Owner return summary の固定点計算に入れる関数を保守的に絞る。
///
/// summary は caller が callee の owner return / consume / host-memory contract を
/// 適用するための情報であり、単なる scalar-only helper には facts が発生しない。
/// ここでは owner leaf を運ぶ public signature と、raw memory / raw view / external
/// memory contract を直接作る operation だけを relevant に残す。判定に含めた operation
/// は通常の summary engine で再検査されるため、この関数は「不要と断定できるもの」を
/// worklist から外すだけで、summary facts 自体を合成しない。
pub(super) fn owner_summary_relevant_functions(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<bool> {
    module
        .functions
        .iter()
        .map(|function| owner_summary_relevant_function(types, function))
        .collect()
}

fn owner_summary_relevant_function(types: &TypeCtx, function: &ResourceFunction) -> bool {
    function_signature_carries_owner_summary_facts(types, function)
        || function
            .blocks
            .iter()
            .any(|block| ops_directly_affect_owner_summary(&block.ops))
}

fn function_signature_carries_owner_summary_facts(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    function
        .params
        .iter()
        .any(|param| place_type_has_owner_leaf(types, &param.place))
        || place_type_has_owner_leaf(types, &Place::local("__return".into(), function.result))
}

fn place_type_has_owner_leaf(types: &TypeCtx, place: &Place) -> bool {
    !owner_leaf_places(types, place).is_empty()
}

fn ops_directly_affect_owner_summary(ops: &[ResourceOp]) -> bool {
    ops.iter().any(op_directly_affects_owner_summary)
}

fn op_directly_affects_owner_summary(op: &ResourceOp) -> bool {
    match op {
        ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::IndirectCall { .. } => true,
        ResourceOp::Call { effect, .. } => !matches!(effect, EffectOp::Pure),
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_directly_affect_owner_summary(then_ops)
                || ops_directly_affect_owner_summary(else_ops)
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_directly_affect_owner_summary(condition_ops)
                || ops_directly_affect_owner_summary(body_ops)
        }
        ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_directly_affect_owner_summary(&arm.ops)),
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. } => false,
    }
}

#[cfg(test)]
#[path = "owner_summary_relevance_tests.rs"]
mod tests;

use super::model::{ResourceFunction, ResourceOp};

pub(super) fn function_allows_complete_leaf_entry_replay(function: &ResourceFunction) -> bool {
    !function_has_indirect_call(function)
}

fn function_has_indirect_call(function: &ResourceFunction) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_have_indirect_call(&block.ops))
}

fn ops_have_indirect_call(ops: &[ResourceOp]) -> bool {
    for op in ops {
        match op {
            ResourceOp::IndirectCall { .. } => return true,
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                if ops_have_indirect_call(then_ops) || ops_have_indirect_call(else_ops) {
                    return true;
                }
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                if ops_have_indirect_call(condition_ops) || ops_have_indirect_call(body_ops) {
                    return true;
                }
            }
            ResourceOp::Match { arms, .. } => {
                if arms.iter().any(|arm| ops_have_indirect_call(&arm.ops)) {
                    return true;
                }
            }
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
            | ResourceOp::Call { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
    false
}

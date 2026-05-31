extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_payload_tracking::{
    collection_slot_lifecycle_event_needs_tracking, collection_slot_payload_type_needs_tracking,
};
use super::collection_slot_storage_carrier::type_can_carry_collection_slot_storage;
use super::model::{ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp};

struct CollectionSlotSummaryCallEdge {
    caller: usize,
    callee: usize,
    exchanges_collection_storage: bool,
}

pub(super) fn collection_slot_summary_relevant_functions(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<bool> {
    let function_indices = module
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| (function.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let signature_carries_collection_storage = module
        .functions
        .iter()
        .map(|function| function_signature_carries_collection_slots(types, function))
        .collect::<Vec<_>>();
    let call_edges = collection_slot_summary_call_edges(module, types, &function_indices);
    let mut relevant = module
        .functions
        .iter()
        .map(|function| {
            function.blocks.iter().any(|block| {
                block
                    .ops
                    .iter()
                    .any(|op| op_directly_affects_collection_slots(types, op))
            })
        })
        .collect::<Vec<_>>();

    loop {
        let mut changed = false;
        for edge in &call_edges {
            if relevant[edge.callee] && !relevant[edge.caller] {
                relevant[edge.caller] = true;
                changed = true;
            }
            if relevant[edge.caller]
                && !relevant[edge.callee]
                && (edge.exchanges_collection_storage
                    || signature_carries_collection_storage[edge.callee])
            {
                relevant[edge.callee] = true;
                changed = true;
            }
        }
        if !changed {
            return relevant;
        }
    }
}

fn function_signature_carries_collection_slots(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    function
        .params
        .iter()
        .any(|param| type_can_carry_collection_slot_storage(types, param.place.ty))
        || type_can_carry_collection_slot_storage(types, function.result)
}

fn collection_slot_summary_call_edges(
    module: &ResourceModule,
    types: &TypeCtx,
    function_indices: &BTreeMap<&str, usize>,
) -> Vec<CollectionSlotSummaryCallEdge> {
    let mut out = Vec::new();
    for (caller, function) in module.functions.iter().enumerate() {
        for block in &function.blocks {
            collect_ops_collection_slot_summary_call_edges(
                &mut out,
                types,
                function_indices,
                caller,
                &block.ops,
            );
        }
    }
    out
}

fn collect_ops_collection_slot_summary_call_edges(
    out: &mut Vec<CollectionSlotSummaryCallEdge>,
    types: &TypeCtx,
    function_indices: &BTreeMap<&str, usize>,
    caller: usize,
    ops: &[ResourceOp],
) {
    for op in ops {
        collect_op_collection_slot_summary_call_edges(out, types, function_indices, caller, op);
    }
}

fn collect_op_collection_slot_summary_call_edges(
    out: &mut Vec<CollectionSlotSummaryCallEdge>,
    types: &TypeCtx,
    function_indices: &BTreeMap<&str, usize>,
    caller: usize,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            output,
            args,
            ..
        } => {
            if let Some(callee) = function_indices.get(name.as_str()) {
                push_unique_collection_slot_call_edge(
                    out,
                    CollectionSlotSummaryCallEdge {
                        caller,
                        callee: *callee,
                        exchanges_collection_storage: place_list_carries_collection_storage(
                            types, output, args,
                        ),
                    },
                );
            }
        }
        ResourceOp::FunctionValue { identity, .. } => {
            if let Some(callee) = function_indices.get(identity.symbol()) {
                push_unique_collection_slot_call_edge(
                    out,
                    CollectionSlotSummaryCallEdge {
                        caller,
                        callee: *callee,
                        exchanges_collection_storage: false,
                    },
                );
            }
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_collection_slot_summary_call_edges(
                out,
                types,
                function_indices,
                caller,
                then_ops,
            );
            collect_ops_collection_slot_summary_call_edges(
                out,
                types,
                function_indices,
                caller,
                else_ops,
            );
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_collection_slot_summary_call_edges(
                out,
                types,
                function_indices,
                caller,
                condition_ops,
            );
            collect_ops_collection_slot_summary_call_edges(
                out,
                types,
                function_indices,
                caller,
                body_ops,
            );
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_collection_slot_summary_call_edges(
                    out,
                    types,
                    function_indices,
                    caller,
                    &arm.ops,
                );
            }
        }
        ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
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

fn place_list_carries_collection_storage(
    types: &TypeCtx,
    output: &super::model::Place,
    args: &[super::model::Place],
) -> bool {
    type_can_carry_collection_slot_storage(types, output.ty)
        || args
            .iter()
            .any(|arg| type_can_carry_collection_slot_storage(types, arg.ty))
}

fn push_unique_collection_slot_call_edge(
    out: &mut Vec<CollectionSlotSummaryCallEdge>,
    edge: CollectionSlotSummaryCallEdge,
) {
    if !out.iter().any(|existing| {
        existing.caller == edge.caller
            && existing.callee == edge.callee
            && existing.exchanges_collection_storage == edge.exchanges_collection_storage
    }) {
        out.push(edge);
    }
}

fn op_directly_affects_collection_slots(types: &TypeCtx, op: &ResourceOp) -> bool {
    match op {
        ResourceOp::CollectionSlotLifecycle { event, .. } => {
            collection_slot_lifecycle_event_needs_tracking(types, *event)
        }
        ResourceOp::CollectionSlotDropTraversal { expected_ty, .. }
        | ResourceOp::CollectionSlotTransformRange { expected_ty, .. } => {
            collection_slot_payload_type_needs_tracking(types, *expected_ty)
        }
        ResourceOp::IndirectCall { .. } => true,
        // StorageRelocate は raw storage の移動だけを表すため、それ単独では
        // slot lifecycle summary の起点にしない。slot state を運ぶ必要が
        // ある呼び出しは、caller/callee の storage-carrying edge から伝播する。
        ResourceOp::CollectionStorageRelocate { .. } => false,
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            then_ops
                .iter()
                .any(|op| op_directly_affects_collection_slots(types, op))
                || else_ops
                    .iter()
                    .any(|op| op_directly_affects_collection_slots(types, op))
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            condition_ops
                .iter()
                .any(|op| op_directly_affects_collection_slots(types, op))
                || body_ops
                    .iter()
                    .any(|op| op_directly_affects_collection_slots(types, op))
        }
        ResourceOp::Match { arms, .. } => arms.iter().any(|arm| {
            arm.ops
                .iter()
                .any(|op| op_directly_affects_collection_slots(types, op))
        }),
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
        | ResourceOp::Construct { .. } => false,
    }
}

#[cfg(test)]
#[path = "collection_slot_summary_relevance_tests.rs"]
mod tests;

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
        ResourceOp::FunctionValue { name, .. } => {
            if let Some(callee) = function_indices.get(name.as_str()) {
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
mod tests {
    use alloc::string::{String, ToString};
    use alloc::vec;

    use crate::source_map::CompilerMemoryType;
    use crate::span::Span;
    use crate::types::{TypeId, TypeKind};

    use super::super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
    use super::super::model::Place;
    use super::*;

    fn empty_function(
        name: &str,
        params: Vec<(String, TypeId)>,
        result: TypeId,
    ) -> ResourceFunction {
        ResourceFunction {
            name: name.to_string(),
            origin_name: name.to_string(),
            type_params: Vec::new(),
            params: params
                .into_iter()
                .map(|(name, ty)| super::super::model::ResourceLocal {
                    name: name.clone(),
                    ty,
                    mutable: false,
                    place: Place::local(name, ty),
                })
                .collect(),
            result,
            effect: crate::ast::Effect::Pure,
            entry_block: super::super::model::ResourceBlockId(0),
            blocks: vec![super::super::model::ResourceBlock {
                id: super::super::model::ResourceBlockId(0),
                ops: Vec::new(),
                terminator: super::super::model::ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn collection_storage_marker_calling_function(
        name: &str,
        callee: &str,
        storage_ty: TypeId,
        value_ty: TypeId,
    ) -> ResourceFunction {
        let mut function = empty_function(name, Vec::new(), value_ty);
        let storage = Place::local("storage".to_string(), storage_ty);
        function.blocks[0]
            .ops
            .push(ResourceOp::CollectionSlotLifecycle {
                target: storage.clone(),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
                span: Span::dummy(),
            });
        function.blocks[0].ops.push(ResourceOp::Call {
            output: Place::local("storage_out".to_string(), storage_ty),
            target: ResourceCallTarget::User {
                name: callee.to_string(),
                type_args: Vec::new(),
            },
            args: vec![storage],
            effect: super::super::model::EffectOp::Pure,
            span: Span::dummy(),
        });
        function
    }

    fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
        types.register_named(
            name.to_string(),
            TypeKind::Struct {
                name: name.to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        )
    }

    fn register_region_token(types: &mut TypeCtx) -> TypeId {
        let raw_ty = types.i32();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let region_token_ty = types.register_named(
            "RegionToken".to_string(),
            TypeKind::Struct {
                name: "RegionToken".to_string(),
                type_params: vec![value_ty],
                fields: vec![raw_ty, raw_ty],
                field_names: vec!["raw".to_string(), "size".to_string()],
            },
        );
        types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
        region_token_ty
    }

    #[test]
    fn owner_token_with_non_copy_payload_keeps_summary_for_slot_storage_transfer() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let payload_ty = register_empty_struct(&mut types, "OwnedPayload");
        let region_token = register_region_token(&mut types);
        let storage_ty = types.apply(region_token, vec![payload_ty]);
        let module = ResourceModule {
            functions: vec![
                collection_storage_marker_calling_function(
                    "mark_collection_storage",
                    "identity_storage",
                    storage_ty,
                    payload_ty,
                ),
                empty_function(
                    "identity_storage",
                    vec![("storage".to_string(), storage_ty)],
                    storage_ty,
                ),
            ],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![true, true]
        );
    }

    #[test]
    fn owner_token_with_copy_payload_does_not_force_collection_slot_callee() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let u8_ty = types.u8();
        types.register_copy_impl_target(u8_ty);
        let region_token = register_region_token(&mut types);
        let storage_ty = types.apply(region_token, vec![u8_ty]);
        let module = ResourceModule {
            functions: vec![
                collection_storage_marker_calling_function(
                    "mark_collection_storage",
                    "identity_storage",
                    storage_ty,
                    u8_ty,
                ),
                empty_function(
                    "identity_storage",
                    vec![("storage".to_string(), storage_ty)],
                    storage_ty,
                ),
            ],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![false, false]
        );
    }

    #[test]
    fn copy_scalar_signature_does_not_force_collection_slot_summary() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let module = ResourceModule {
            functions: vec![empty_function(
                "identity_i32",
                vec![("value".to_string(), types.i32())],
                types.i32(),
            )],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![false]
        );
    }

    #[test]
    fn direct_slot_payload_i32_does_not_make_i32_helper_relevant() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let module = ResourceModule {
            functions: vec![
                collection_storage_marker_calling_function(
                    "mark_collection_storage",
                    "identity_i32",
                    types.i32(),
                    types.i32(),
                ),
                empty_function(
                    "identity_i32",
                    vec![("value".to_string(), types.i32())],
                    types.i32(),
                ),
            ],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![false, false]
        );
    }

    #[test]
    fn string_owner_signature_does_not_force_collection_slot_summary() {
        let types = TypeCtx::new();
        let module = ResourceModule {
            functions: vec![empty_function(
                "identity_str",
                vec![("value".to_string(), types.str())],
                types.str(),
            )],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![false]
        );
    }
}

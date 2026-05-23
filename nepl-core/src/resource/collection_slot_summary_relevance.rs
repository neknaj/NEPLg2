extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_storage_carrier::type_can_carry_collection_slot_storage;
use super::model::{
    Place, PlaceRoot, ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp,
};
use super::owner_summary_leaf::owner_leaf_places;

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
    let mut relevant = module
        .functions
        .iter()
        .map(|function| {
            function_signature_carries_collection_slots(types, function)
                || function
                    .blocks
                    .iter()
                    .any(|block| block.ops.iter().any(op_directly_affects_collection_slots))
        })
        .collect::<Vec<_>>();

    loop {
        let mut changed = false;
        for (index, function) in module.functions.iter().enumerate() {
            if relevant[index] {
                continue;
            }
            let mut dependencies = BTreeSet::new();
            collect_collection_slot_summary_dependencies(function, &mut dependencies);
            if dependencies.into_iter().any(|name| {
                function_indices
                    .get(name.as_str())
                    .is_some_and(|dependency| relevant[*dependency])
            }) {
                relevant[index] = true;
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
        .any(|param| type_carries_collection_slots(types, param.place.ty))
        || type_carries_collection_slots(types, function.result)
}

fn type_carries_collection_slots(types: &TypeCtx, ty: crate::types::TypeId) -> bool {
    !owner_leaf_places(
        types,
        &Place {
            root: PlaceRoot::Unknown,
            projections: Vec::new(),
            ty,
        },
    )
    .is_empty()
        || type_can_carry_collection_slot_storage(types, ty)
}

fn collect_collection_slot_summary_dependencies(
    function: &ResourceFunction,
    out: &mut BTreeSet<String>,
) {
    for block in &function.blocks {
        collect_ops_collection_slot_summary_dependencies(&block.ops, out);
    }
}

fn collect_ops_collection_slot_summary_dependencies(
    ops: &[ResourceOp],
    out: &mut BTreeSet<String>,
) {
    for op in ops {
        collect_op_collection_slot_summary_dependencies(op, out);
    }
}

fn collect_op_collection_slot_summary_dependencies(op: &ResourceOp, out: &mut BTreeSet<String>) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            ..
        }
        | ResourceOp::FunctionValue { name, .. } => {
            out.insert(name.clone());
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_collection_slot_summary_dependencies(then_ops, out);
            collect_ops_collection_slot_summary_dependencies(else_ops, out);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_collection_slot_summary_dependencies(condition_ops, out);
            collect_ops_collection_slot_summary_dependencies(body_ops, out);
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_collection_slot_summary_dependencies(&arm.ops, out);
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
        | ResourceOp::Construct { .. } => {}
    }
}

fn op_directly_affects_collection_slots(op: &ResourceOp) -> bool {
    match op {
        ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::IndirectCall { .. } => true,
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            then_ops.iter().any(op_directly_affects_collection_slots)
                || else_ops.iter().any(op_directly_affects_collection_slots)
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            condition_ops
                .iter()
                .any(op_directly_affects_collection_slots)
                || body_ops.iter().any(op_directly_affects_collection_slots)
        }
        ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| arm.ops.iter().any(op_directly_affects_collection_slots)),
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

    use crate::span::Span;
    use crate::types::{TypeId, TypeKind};

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

    #[test]
    fn non_copy_struct_signature_keeps_summary_for_slot_storage_transfer() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let storage_ty = types.register_named(
            "CollectionStorage".to_string(),
            TypeKind::Struct {
                name: "CollectionStorage".to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        );
        let module = ResourceModule {
            functions: vec![empty_function(
                "identity_storage",
                vec![("storage".to_string(), storage_ty)],
                storage_ty,
            )],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![true]
        );
    }

    #[test]
    fn aggregate_storage_signature_is_relevant_even_with_same_shaped_copy_payload() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        let copied_payload_ty = types.register_named(
            "Owned".to_string(),
            TypeKind::Struct {
                name: "Owned".to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        );
        types.register_copy_impl_target(copied_payload_ty);
        let storage_ty = types.register_named(
            "CollectionStorage".to_string(),
            TypeKind::Struct {
                name: "CollectionStorage".to_string(),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        );
        let module = ResourceModule {
            functions: vec![empty_function(
                "identity_storage",
                vec![("storage".to_string(), storage_ty)],
                storage_ty,
            )],
            entry: None,
            string_literals: vec![],
        };

        assert_eq!(
            collection_slot_summary_relevant_functions(&module, &types),
            vec![true]
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
}

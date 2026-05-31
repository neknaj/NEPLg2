use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::model::{RawMemoryOp, ResourceCallTarget, ResourceFunction, ResourceOp};

pub(super) fn owner_summary_type_params(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> Vec<TypeId> {
    let mut out = function.type_params.clone();
    for param in &function.params {
        collect_type_vars(types, param.ty, &mut out, 0);
    }
    collect_type_vars(types, function.result, &mut out, 0);
    collect_owner_summary_body_type_vars(types, function, &mut out);
    out
}

fn collect_owner_summary_body_type_vars(
    types: &TypeCtx,
    function: &ResourceFunction,
    out: &mut Vec<TypeId>,
) {
    // raw-init summary の param cell value type は、callee summary や raw address view の
    // 先にある型として function signature の外側に現れることがある。ここで summary
    // replay の意味に関わる body 内の型だけを owner summary boundary へ昇格し、既存の
    // type-parameter boundary hash と replay 側の strict 重複検査に通す。単なる local の
    // 型は authority にしない。
    for block in &function.blocks {
        for op in &block.ops {
            collect_owner_summary_body_type_vars_from_op(types, op, out);
        }
    }
}

fn collect_owner_summary_body_type_vars_from_op(
    types: &TypeCtx,
    op: &ResourceOp,
    out: &mut Vec<TypeId>,
) {
    match op {
        ResourceOp::RawMemory {
            operation,
            output,
            args,
            ..
        } => match operation {
            RawMemoryOp::Load => collect_type_vars(types, output.ty, out, 0),
            RawMemoryOp::Store => {
                if let Some(value) = args.get(1) {
                    collect_type_vars(types, value.ty, out, 0);
                }
            }
            RawMemoryOp::Fill => {
                if let Some(value) = args.get(2) {
                    collect_type_vars(types, value.ty, out, 0);
                }
            }
            RawMemoryOp::Alloc
            | RawMemoryOp::Dealloc
            | RawMemoryOp::Realloc
            | RawMemoryOp::LoadU8
            | RawMemoryOp::StoreU8
            | RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::FillBytes => {}
        },
        ResourceOp::Call { target, .. } => collect_call_target_type_vars(types, target, out),
        ResourceOp::IndirectCall { params, result, .. } => {
            for param in params {
                collect_type_vars(types, *param, out, 0);
            }
            collect_type_vars(types, *result, out, 0);
        }
        ResourceOp::CollectionSlotLifecycle { event, .. } => {
            collect_collection_slot_event_type_vars(types, *event, out);
        }
        ResourceOp::CollectionSlotDropTraversal { expected_ty, .. }
        | ResourceOp::CollectionSlotTransformRange { expected_ty, .. } => {
            collect_type_vars(types, *expected_ty, out, 0);
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            for op in then_ops {
                collect_owner_summary_body_type_vars_from_op(types, op, out);
            }
            for op in else_ops {
                collect_owner_summary_body_type_vars_from_op(types, op, out);
            }
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            for op in condition_ops {
                collect_owner_summary_body_type_vars_from_op(types, op, out);
            }
            for op in body_ops {
                collect_owner_summary_body_type_vars_from_op(types, op, out);
            }
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                for op in &arm.ops {
                    collect_owner_summary_body_type_vars_from_op(types, op, out);
                }
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
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::Construct { .. } => {}
    }
}

fn collect_call_target_type_vars(
    types: &TypeCtx,
    target: &ResourceCallTarget,
    out: &mut Vec<TypeId>,
) {
    match target {
        ResourceCallTarget::Builtin { .. } => {}
        ResourceCallTarget::User { type_args, .. } => {
            for ty in type_args {
                collect_type_vars(types, *ty, out, 0);
            }
        }
        ResourceCallTarget::Trait { self_ty, .. } => collect_type_vars(types, *self_ty, out, 0),
    }
}

fn collect_collection_slot_event_type_vars(
    types: &TypeCtx,
    event: CollectionSlotLifecycleEvent,
    out: &mut Vec<TypeId>,
) {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty }
        | CollectionSlotLifecycleEvent::BorrowRead {
            expected_ty: value_ty,
        }
        | CollectionSlotLifecycleEvent::MoveOut {
            expected_ty: value_ty,
        }
        | CollectionSlotLifecycleEvent::DropInitialized {
            expected_ty: value_ty,
        }
        | CollectionSlotLifecycleEvent::StorageDealloc { value_ty } => {
            collect_type_vars(types, value_ty, out, 0);
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized { old_ty, new_ty, .. } => {
            collect_type_vars(types, old_ty, out, 0);
            collect_type_vars(types, new_ty, out, 0);
        }
    }
}

fn collect_type_vars(types: &TypeCtx, ty: TypeId, out: &mut Vec<TypeId>, depth: usize) {
    if depth > 32 {
        return;
    }
    match types.get_ref(ty) {
        TypeKind::Var(var) => {
            if let Some(binding) = var.binding {
                collect_type_vars(types, binding, out, depth + 1);
            } else if !out.contains(&ty) {
                out.push(ty);
            }
        }
        TypeKind::Apply { args, .. } => {
            for arg in args {
                collect_type_vars(types, *arg, out, depth + 1);
            }
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for variant in variants {
                if let Some(payload) = variant.payload {
                    collect_type_vars(types, payload, out, depth + 1);
                }
            }
        }
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for field in fields {
                collect_type_vars(types, *field, out, depth + 1);
            }
        }
        TypeKind::Tuple { items } => {
            for item in items {
                collect_type_vars(types, *item, out, depth + 1);
            }
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            for ty in type_params {
                collect_type_vars(types, *ty, out, depth + 1);
            }
            for param in params {
                collect_type_vars(types, *param, out, depth + 1);
            }
            collect_type_vars(types, *result, out, depth + 1);
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            collect_type_vars(types, *inner, out, depth + 1);
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::Span;

    use super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceTerminator,
    };
    use super::*;

    fn function_with_ops(types: &TypeCtx, ops: Vec<ResourceOp>) -> ResourceFunction {
        ResourceFunction {
            name: "owner_summary_type_params_subject".to_string(),
            origin_name: "owner_summary_type_params_subject".to_string(),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: "address".to_string(),
                ty: types.i32(),
                mutable: false,
                place: Place::local("address".to_string(), types.i32()),
            }],
            result: types.unit(),
            effect: Effect::Impure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    #[test]
    fn owner_summary_type_params_include_raw_store_value_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_ops(
            &types,
            vec![ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: Place::local("out".to_string(), types.unit()),
                args: vec![
                    Place::local("address".to_string(), types.i32()),
                    Place::local("value".to_string(), value_ty),
                ],
                span: Span::dummy(),
            }],
        );

        let boundary = owner_summary_type_params(&types, &function);

        assert!(boundary.contains(&value_ty));
    }

    #[test]
    fn owner_summary_type_params_include_raw_load_output_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_ops(
            &types,
            vec![ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: Place::local("loaded".to_string(), value_ty),
                args: vec![Place::local("address".to_string(), types.i32())],
                span: Span::dummy(),
            }],
        );

        let boundary = owner_summary_type_params(&types, &function);

        assert!(boundary.contains(&value_ty));
    }

    #[test]
    fn owner_summary_type_params_include_user_call_type_arguments() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_ops(
            &types,
            vec![ResourceOp::Call {
                output: Place::local("out".to_string(), types.unit()),
                target: ResourceCallTarget::User {
                    name: "callee".to_string(),
                    type_args: vec![value_ty],
                },
                args: vec![Place::local("address".to_string(), types.i32())],
                effect: super::super::model::EffectOp::Pure,
                span: Span::dummy(),
            }],
        );

        let boundary = owner_summary_type_params(&types, &function);

        assert!(boundary.contains(&value_ty));
    }

    #[test]
    fn owner_summary_type_params_include_collection_slot_value_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_ops(
            &types,
            vec![ResourceOp::CollectionSlotLifecycle {
                target: Place::local("storage".to_string(), types.i32()),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
                span: Span::dummy(),
            }],
        );

        let boundary = owner_summary_type_params(&types, &function);

        assert!(boundary.contains(&value_ty));
    }

    #[test]
    fn owner_summary_type_params_keep_ambiguous_raw_generics_visible_to_boundary_hash() {
        let mut types = TypeCtx::new();
        let first = types.fresh_var(Some("T".to_string()));
        let second = types.fresh_var(Some("T".to_string()));
        let function = function_with_ops(
            &types,
            vec![
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Store,
                    output: Place::local("first_out".to_string(), types.unit()),
                    args: vec![
                        Place::local("address".to_string(), types.i32()),
                        Place::local("first_value".to_string(), first),
                    ],
                    span: Span::dummy(),
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Store,
                    output: Place::local("second_out".to_string(), types.unit()),
                    args: vec![
                        Place::local("address".to_string(), types.i32()),
                        Place::local("second_value".to_string(), second),
                    ],
                    span: Span::dummy(),
                },
            ],
        );

        let boundary = owner_summary_type_params(&types, &function);

        assert!(boundary.contains(&first));
        assert!(boundary.contains(&second));
    }
}

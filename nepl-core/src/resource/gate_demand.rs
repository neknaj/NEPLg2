extern crate alloc;

use alloc::collections::BTreeSet;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{
    EffectOp, Place, PlaceRoot, ResourceCallTarget, ResourceExprKind, ResourceFunction,
    ResourceModule, ResourceOp, ResourceTerminator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceSafetyGateDemand {
    ResourceNeutral,
    RequiresResourceSafetyGates,
}

pub(crate) fn resource_safety_gate_demand(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceSafetyGateDemand {
    for function in &module.functions {
        if resource_function_requires_safety_gates(function, types) {
            return ResourceSafetyGateDemand::RequiresResourceSafetyGates;
        }
    }
    ResourceSafetyGateDemand::ResourceNeutral
}

fn resource_function_requires_safety_gates(function: &ResourceFunction, types: &TypeCtx) -> bool {
    if !matches!(function.effect, Effect::Pure)
        || resource_type_requires_safety_gate(types, function.result)
        || function
            .params
            .iter()
            .any(|param| resource_place_requires_safety_gate(&param.place, types))
    {
        return true;
    }

    function.blocks.iter().any(|block| {
        block
            .ops
            .iter()
            .any(|op| resource_op_requires_safety_gate(op, types))
            || resource_terminator_requires_safety_gate(&block.terminator, types)
    })
}

fn resource_terminator_requires_safety_gate(
    terminator: &ResourceTerminator,
    types: &TypeCtx,
) -> bool {
    match terminator {
        ResourceTerminator::Return { value, .. } => value
            .as_ref()
            .is_some_and(|place| resource_place_requires_safety_gate(place, types)),
        ResourceTerminator::Unreachable { .. } => false,
        ResourceTerminator::RawBody { .. } => true,
    }
}

fn resource_op_requires_safety_gate(op: &ResourceOp, types: &TypeCtx) -> bool {
    match op {
        ResourceOp::Expr { kind, output, .. } => {
            resource_expr_kind_requires_safety_gate(kind)
                || resource_place_requires_safety_gate(output, types)
        }
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            resource_place_requires_safety_gate(place, types)
                || initializer
                    .as_ref()
                    .is_some_and(|place| resource_place_requires_safety_gate(place, types))
        }
        ResourceOp::Read { source, output, .. } => {
            resource_place_requires_safety_gate(source, types)
                || resource_place_requires_safety_gate(output, types)
        }
        ResourceOp::Assign { target, value, .. } => {
            resource_place_requires_safety_gate(target, types)
                || resource_place_requires_safety_gate(value, types)
        }
        ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::Construct { .. } => true,
        ResourceOp::RawAddressAlias { source, target, .. }
        | ResourceOp::RawAddressView { source, target, .. } => {
            resource_place_requires_safety_gate(source, types)
                || resource_place_requires_safety_gate(target, types)
        }
        ResourceOp::CallEffect { effect, .. } => resource_effect_op_requires_safety_gate(effect),
        ResourceOp::FunctionValue { output, effect, .. } => {
            resource_effect_op_requires_safety_gate(effect)
                || resource_place_requires_safety_gate(output, types)
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            ..
        } => {
            resource_effect_op_requires_safety_gate(effect)
                || resource_call_target_requires_safety_gate(target, types)
                || resource_place_requires_safety_gate(output, types)
                || args
                    .iter()
                    .any(|arg| resource_place_requires_safety_gate(arg, types))
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            params,
            result,
            args,
            effect,
            ..
        } => {
            resource_effect_op_requires_safety_gate(effect)
                || resource_place_requires_safety_gate(output, types)
                || resource_place_requires_safety_gate(callee, types)
                || resource_type_requires_safety_gate(types, *result)
                || params
                    .iter()
                    .any(|param| resource_type_requires_safety_gate(types, *param))
                || args
                    .iter()
                    .any(|arg| resource_place_requires_safety_gate(arg, types))
        }
        ResourceOp::Branch {
            output,
            condition,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            resource_place_requires_safety_gate(output, types)
                || resource_place_requires_safety_gate(condition, types)
                || then_ops
                    .iter()
                    .any(|op| resource_op_requires_safety_gate(op, types))
                || resource_place_requires_safety_gate(then_value, types)
                || else_ops
                    .iter()
                    .any(|op| resource_op_requires_safety_gate(op, types))
                || resource_place_requires_safety_gate(else_value, types)
        }
        ResourceOp::Loop {
            condition_ops,
            condition,
            body_ops,
            ..
        } => {
            resource_place_requires_safety_gate(condition, types)
                || condition_ops
                    .iter()
                    .any(|op| resource_op_requires_safety_gate(op, types))
                || body_ops
                    .iter()
                    .any(|op| resource_op_requires_safety_gate(op, types))
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            resource_place_requires_safety_gate(output, types)
                || resource_place_requires_safety_gate(scrutinee, types)
                || arms.iter().any(|arm| {
                    arm.bind_local
                        .as_ref()
                        .is_some_and(|place| resource_place_requires_safety_gate(place, types))
                        || arm
                            .ops
                            .iter()
                            .any(|op| resource_op_requires_safety_gate(op, types))
                        || resource_place_requires_safety_gate(&arm.value, types)
                })
        }
    }
}

fn resource_expr_kind_requires_safety_gate(kind: &ResourceExprKind) -> bool {
    match kind {
        ResourceExprKind::Literal
        | ResourceExprKind::LiteralI32(_)
        | ResourceExprKind::LocalRead
        | ResourceExprKind::FunctionValue
        | ResourceExprKind::Call
        | ResourceExprKind::Block
        | ResourceExprKind::Let
        | ResourceExprKind::Set
        | ResourceExprKind::Intrinsic => false,
        ResourceExprKind::IndirectCall
        | ResourceExprKind::Branch
        | ResourceExprKind::Loop
        | ResourceExprKind::Match
        | ResourceExprKind::Construct
        | ResourceExprKind::Borrow
        | ResourceExprKind::Deref
        | ResourceExprKind::Drop => true,
    }
}

fn resource_call_target_requires_safety_gate(target: &ResourceCallTarget, types: &TypeCtx) -> bool {
    match target {
        ResourceCallTarget::Builtin { .. } => false,
        ResourceCallTarget::User { type_args, .. } => type_args
            .iter()
            .any(|arg| resource_type_requires_safety_gate(types, *arg)),
        ResourceCallTarget::Trait {
            trait_args,
            self_ty,
            ..
        } => {
            resource_type_requires_safety_gate(types, *self_ty)
                || trait_args
                    .iter()
                    .any(|arg| resource_type_requires_safety_gate(types, *arg))
        }
    }
}

fn resource_effect_op_requires_safety_gate(effect: &EffectOp) -> bool {
    match effect {
        EffectOp::Pure => false,
        EffectOp::UserCall { effect, .. } | EffectOp::IndirectCall { effect } => {
            matches!(effect, Effect::Impure)
        }
        EffectOp::InternalAlloc
        | EffectOp::UnsafeMemory { .. }
        | EffectOp::ExternalIo { .. }
        | EffectOp::Nondet { .. }
        | EffectOp::Unknown { .. } => true,
    }
}

fn resource_place_requires_safety_gate(place: &Place, types: &TypeCtx) -> bool {
    if matches!(place.root, PlaceRoot::Storage(_) | PlaceRoot::Unknown)
        || !place.projections.is_empty()
    {
        return true;
    }
    resource_type_requires_safety_gate(types, place.ty)
}

fn resource_type_requires_safety_gate(types: &TypeCtx, ty: TypeId) -> bool {
    let mut visiting = BTreeSet::new();
    resource_type_requires_safety_gate_inner(types, ty, &mut visiting)
}

fn resource_type_requires_safety_gate_inner(
    types: &TypeCtx,
    ty: TypeId,
    visiting: &mut BTreeSet<TypeId>,
) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    if !visiting.insert(resolved) {
        return false;
    }
    let requires = match types.get_ref(resolved) {
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never => false,
        TypeKind::Named(name) => {
            !matches!(name.as_str(), "i64" | "i128" | "u64" | "u128" | "f64")
                || matches!(name.as_str(), "MemPtr" | "RegionToken")
        }
        TypeKind::Enum { .. } => true,
        TypeKind::Struct { .. } => true,
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| resource_type_requires_safety_gate_inner(types, *item, visiting)),
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => {
            matches!(effect, Effect::Impure)
                || resource_type_requires_safety_gate_inner(types, *result, visiting)
                || params
                    .iter()
                    .any(|param| resource_type_requires_safety_gate_inner(types, *param, visiting))
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| resource_type_requires_safety_gate_inner(types, binding, visiting))
            .unwrap_or(true),
        TypeKind::Apply { base, args } => {
            resource_nominal_type_is_raw_resource(types, *base)
                || resource_apply_base_requires_safety_gate(types, *base)
                || args
                    .iter()
                    .any(|arg| resource_type_requires_safety_gate_inner(types, *arg, visiting))
        }
        TypeKind::Box(_) | TypeKind::Reference(_, _) => true,
    };
    visiting.remove(&resolved);
    requires
}

fn resource_nominal_type_is_raw_resource(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Named(name) | TypeKind::Struct { name, .. } => {
            matches!(name.as_str(), "MemPtr" | "RegionToken")
        }
        TypeKind::Apply { base, .. } => resource_nominal_type_is_raw_resource(types, *base),
        _ => false,
    }
}

fn resource_apply_base_requires_safety_gate(types: &TypeCtx, base: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(base));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } | TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => resource_apply_base_requires_safety_gate(types, *base),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::span::Span;

    use super::super::model::{
        RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceExprKind, ResourceId, ResourceLocal,
    };
    use super::*;

    #[test]
    fn gate_demand_keeps_primitive_identity_calls_neutral() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let span = Span::dummy();
        let input = Place::local(String::from("x"), i32_ty);
        let read = Place::temporary(ResourceId(0), i32_ty);
        let output = Place::temporary(ResourceId(1), i32_ty);
        let module = ResourceModule {
            functions: vec![ResourceFunction {
                name: String::from("main"),
                params: vec![ResourceLocal {
                    name: String::from("x"),
                    ty: i32_ty,
                    mutable: false,
                    place: input.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Block,
                            output: Place::temporary(ResourceId(2), i32_ty),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Read {
                            source: input,
                            output: read.clone(),
                            span,
                        },
                        ResourceOp::CallEffect {
                            effect: EffectOp::UserCall {
                                name: String::from("id"),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Call {
                            output: output.clone(),
                            target: ResourceCallTarget::User {
                                name: String::from("id"),
                                type_args: vec![],
                            },
                            args: vec![read.clone()],
                            effect: EffectOp::UserCall {
                                name: String::from("id"),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::RawAddressAlias {
                            source: read,
                            target: output.clone(),
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Call,
                            output: output.clone(),
                            ty: i32_ty,
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(output),
                        span,
                    },
                    span,
                }],
                span,
            }],
            entry: Some(String::from("main")),
            string_literals: vec![],
        };

        assert_eq!(
            resource_safety_gate_demand(&module, &types),
            ResourceSafetyGateDemand::ResourceNeutral
        );
    }

    #[test]
    fn gate_demand_requires_raw_memory_gates() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let span = Span::dummy();
        let size = Place::temporary(ResourceId(0), i32_ty);
        let raw = Place::temporary(ResourceId(1), i32_ty);
        let module = ResourceModule {
            functions: vec![ResourceFunction {
                name: String::from("main"),
                params: vec![],
                result: i32_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::LiteralI32(4),
                            output: size.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: raw.clone(),
                            args: vec![size],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(raw),
                        span,
                    },
                    span,
                }],
                span,
            }],
            entry: Some(String::from("main")),
            string_literals: vec![],
        };

        assert_eq!(
            resource_safety_gate_demand(&module, &types),
            ResourceSafetyGateDemand::RequiresResourceSafetyGates
        );
    }
}

extern crate alloc;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_alias_type::type_can_seed_raw_address_alias;
use super::model::{EffectOp, Place, ResourceFunction, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn should_seed_raw_address_parameter(
    function: &ResourceFunction,
    parameter: &Place,
    types: &TypeCtx,
) -> bool {
    if !type_can_seed_raw_address_alias(types, parameter.ty) {
        return false;
    }
    if !type_is_plain_i32(types, parameter.ty) {
        return true;
    }
    function_uses_parameter_as_raw_address(function, parameter)
}

fn type_is_plain_i32(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    matches!(types.get_ref(resolved), TypeKind::I32)
}

fn function_uses_parameter_as_raw_address(function: &ResourceFunction, parameter: &Place) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_use_parameter_as_raw_address(&block.ops, parameter))
}

fn ops_use_parameter_as_raw_address(ops: &[ResourceOp], parameter: &Place) -> bool {
    ops.iter()
        .any(|op| op_uses_parameter_as_raw_address(op, parameter))
}

fn op_uses_parameter_as_raw_address(op: &ResourceOp, parameter: &Place) -> bool {
    match op {
        ResourceOp::RawAddressAlias { source, .. } | ResourceOp::RawAddressView { source, .. } => {
            place_has_prefix(source, parameter)
        }
        ResourceOp::RawMemory { args, .. } => {
            args.iter().any(|arg| place_has_prefix(arg, parameter))
        }
        ResourceOp::Call { args, effect, .. } | ResourceOp::IndirectCall { args, effect, .. } => {
            matches!(
                effect,
                EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
            ) && args.iter().any(|arg| place_has_prefix(arg, parameter))
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_use_parameter_as_raw_address(then_ops, parameter)
                || ops_use_parameter_as_raw_address(else_ops, parameter)
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_use_parameter_as_raw_address(condition_ops, parameter)
                || ops_use_parameter_as_raw_address(body_ops, parameter)
        }
        ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_use_parameter_as_raw_address(&arm.ops, parameter)),
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::Construct { .. } => false,
    }
}

fn place_has_prefix(place: &Place, prefix: &Place) -> bool {
    place == prefix || place_suffix_after_prefix(place, prefix).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeCtx;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::super::model::{
        RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceId, ResourceLocal, ResourceTerminator,
    };

    #[test]
    fn i32_parameter_without_raw_address_use_is_not_seeded() {
        let types = TypeCtx::new();
        let param = Place::local("addr".to_string(), types.i32());
        let function = resource_function(
            param.clone(),
            Vec::new(),
            ResourceTerminator::Return {
                value: Some(param.clone()),
                span: Span::dummy(),
            },
        );

        assert!(!should_seed_raw_address_parameter(
            &function, &param, &types
        ));
    }

    #[test]
    fn i32_parameter_with_raw_memory_use_is_seeded() {
        let types = TypeCtx::new();
        let param = Place::local("addr".to_string(), types.i32());
        let output = Place::temporary(ResourceId(0), types.i32());
        let function = resource_function(
            param.clone(),
            vec![ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output,
                args: vec![param.clone()],
                span: Span::dummy(),
            }],
            ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
        );

        assert!(should_seed_raw_address_parameter(&function, &param, &types));
    }

    fn resource_function(
        param: Place,
        ops: Vec<ResourceOp>,
        terminator: ResourceTerminator,
    ) -> ResourceFunction {
        ResourceFunction {
            name: "f".to_string(),
            params: vec![ResourceLocal {
                name: "addr".to_string(),
                ty: param.ty,
                mutable: false,
                place: param,
            }],
            result: TypeId(0),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator,
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }
}

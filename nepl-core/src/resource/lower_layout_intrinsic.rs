use crate::hir::{FuncRef, HirBody, HirExprKind};
use crate::layout::{storage_align_bytes, storage_size_bytes};
use crate::runtime_helpers::helper_base_name;
use crate::types::TypeId;

use super::lower::LoweringEnvironment;

pub(super) fn layout_intrinsic_i32_value_from_callee(
    callee: &FuncRef,
    env: &LoweringEnvironment,
) -> Option<i32> {
    match callee {
        FuncRef::User(name, type_args, _) => layout_intrinsic_i32_value(name, type_args, env),
        FuncRef::Builtin(name) => layout_intrinsic_i32_value(name, &[], env),
        FuncRef::Trait { .. } => None,
    }
}

pub(super) fn layout_intrinsic_i32_value(
    name: &str,
    type_args: &[TypeId],
    env: &LoweringEnvironment,
) -> Option<i32> {
    if type_args.len() == 1 {
        return match helper_base_name(name) {
            "size_of" => i32::try_from(storage_size_bytes(env.types, type_args[0])).ok(),
            "align_of" => i32::try_from(storage_align_bytes(env.types, type_args[0])).ok(),
            _ => None,
        };
    }
    let function = env.function(name)?;
    let HirBody::Block(block) = &function.body else {
        return None;
    };
    if block.lines.len() != 1 {
        return None;
    }
    let HirExprKind::Intrinsic {
        name, type_args, ..
    } = &block.lines[0].expr.kind
    else {
        return None;
    };
    layout_intrinsic_i32_value(name, type_args, env)
}

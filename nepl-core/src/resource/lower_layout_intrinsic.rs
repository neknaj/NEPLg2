use crate::hir::{FuncRef, HirBody, HirExprKind};
use crate::intrinsic_kinds::CoreIntrinsicKind;
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

pub(super) fn layout_intrinsic_i64_value_from_callee(
    callee: &FuncRef,
    env: &LoweringEnvironment,
) -> Option<i64> {
    layout_intrinsic_i32_value_from_callee(callee, env).map(i64::from)
}

pub(super) fn layout_intrinsic_i32_value(
    name: &str,
    type_args: &[TypeId],
    env: &LoweringEnvironment,
) -> Option<i32> {
    if let Some(value) = CoreIntrinsicKind::from_intrinsic_name(helper_base_name(name))
        .and_then(|kind| kind.layout_i32_value(env.types, type_args))
    {
        return Some(value);
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

pub(super) fn layout_intrinsic_i64_value(
    name: &str,
    type_args: &[TypeId],
    env: &LoweringEnvironment,
) -> Option<i64> {
    layout_intrinsic_i32_value(name, type_args, env).map(i64::from)
}

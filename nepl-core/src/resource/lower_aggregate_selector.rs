extern crate alloc;

use alloc::string::String;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::runtime_helpers::helper_base_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AggregateFieldSelector<'a> {
    Index(usize),
    Name(&'a str),
    Unsupported,
}

pub(super) fn non_negative_i32_literal(expr: &HirExpr) -> Option<usize> {
    match expr.kind {
        HirExprKind::LiteralI32(value) if value >= 0 => Some(value as usize),
        _ => None,
    }
}

pub(super) fn aggregate_field_selector<'a>(
    selector: &HirExpr,
    string_literals: &'a [String],
) -> AggregateFieldSelector<'a> {
    if let Some(index) = non_negative_i32_literal(selector) {
        return AggregateFieldSelector::Index(index);
    }
    match &selector.kind {
        HirExprKind::LiteralStr(index) => string_literals
            .get(*index as usize)
            .map(String::as_str)
            .map(AggregateFieldSelector::Name)
            .unwrap_or(AggregateFieldSelector::Unsupported),
        _ => AggregateFieldSelector::Unsupported,
    }
}

pub(super) fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() == 2 => {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if matches!(callee_base_name(callee), Some("add")) && args.len() == 2 =>
        {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

fn callee_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::runtime_helpers::helper_base_name;

use super::model::ResourceOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddressProjectionPrimitive {
    Add,
}

impl AddressProjectionPrimitive {
    fn from_symbol(name: &str) -> Option<Self> {
        match helper_base_name(name) {
            "add" => Some(Self::Add),
            _ => None,
        }
    }

    fn from_callee(callee: &FuncRef) -> Option<Self> {
        match callee {
            FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Self::from_symbol(name),
            FuncRef::Trait { .. } => None,
        }
    }
}

pub(super) fn non_negative_i32_literal(expr: &HirExpr) -> Option<usize> {
    match expr.kind {
        HirExprKind::LiteralI32(value) if value >= 0 => Some(value as usize),
        _ => None,
    }
}

pub(super) fn intrinsic_is_address_projection(name: &str) -> bool {
    AddressProjectionPrimitive::from_symbol(name) == Some(AddressProjectionPrimitive::Add)
}

pub(super) fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. }
            if AddressProjectionPrimitive::from_symbol(name)
                == Some(AddressProjectionPrimitive::Add)
                && args.len() == 2 =>
        {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if AddressProjectionPrimitive::from_callee(callee)
                == Some(AddressProjectionPrimitive::Add)
                && args.len() == 2 =>
        {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

pub(super) fn storage_offset_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, ResourceOffset)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. }
            if intrinsic_is_address_projection(name) && !args.is_empty() =>
        {
            let offset = args
                .get(1)
                .and_then(non_negative_i32_literal)
                .map(ResourceOffset::Known)
                .unwrap_or(ResourceOffset::Unknown);
            Some((&args[0], offset))
        }
        _ => None,
    }
}

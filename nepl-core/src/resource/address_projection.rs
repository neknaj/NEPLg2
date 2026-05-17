use crate::hir::{FuncRef, HirExpr, HirExprKind};

use super::{model::ResourceOffset, scalar_primitive::I32ArithmeticPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AddressProjectionPrimitive {
    Add,
    Sub,
}

impl AddressProjectionPrimitive {
    pub(super) fn from_symbol(name: &str) -> Option<Self> {
        Self::from_arithmetic(I32ArithmeticPrimitive::from_symbol(name)?)
    }

    pub(super) fn from_base_name(base: &str) -> Option<Self> {
        Self::from_arithmetic(I32ArithmeticPrimitive::from_base_name(base)?)
    }

    fn from_arithmetic(arithmetic: I32ArithmeticPrimitive) -> Option<Self> {
        match arithmetic {
            I32ArithmeticPrimitive::Add => Some(Self::Add),
            I32ArithmeticPrimitive::Sub => Some(Self::Sub),
            I32ArithmeticPrimitive::Mul => None,
        }
    }

    fn from_callee(callee: &FuncRef) -> Option<Self> {
        Self::from_arithmetic(I32ArithmeticPrimitive::from_func_ref(callee)?)
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
            if intrinsic_is_address_projection(name) && args.len() == 2 =>
        {
            let offset = non_negative_i32_literal(&args[1])?;
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if args.len() == 2
                && AddressProjectionPrimitive::from_callee(callee)
                    == Some(AddressProjectionPrimitive::Add) =>
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

use crate::hir::FuncRef;
use crate::runtime_helpers::helper_base_name;

use super::model::{ResourceCallTarget, ResourceI32RelationOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum I32ArithmeticPrimitive {
    Add,
    Sub,
    Mul,
}

impl I32ArithmeticPrimitive {
    pub(super) fn from_symbol(name: &str) -> Option<Self> {
        Self::from_base_name(helper_base_name(name))
    }

    pub(super) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "mul" => Some(Self::Mul),
            _ => None,
        }
    }

    pub(super) fn from_func_ref(callee: &FuncRef) -> Option<Self> {
        match callee {
            FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Self::from_symbol(name),
            FuncRef::Trait { .. } => None,
        }
    }

    pub(super) fn from_resource_call_target(target: &ResourceCallTarget) -> Option<Self> {
        match target {
            ResourceCallTarget::Builtin { name } | ResourceCallTarget::User { name, .. } => {
                Self::from_symbol(name)
            }
            ResourceCallTarget::Trait { method, .. } => Self::from_symbol(method.as_str()),
        }
    }

    pub(super) fn checked_i64(self, left: i64, right: i64) -> Option<i64> {
        match self {
            Self::Add => left.checked_add(right),
            Self::Sub => left.checked_sub(right),
            Self::Mul => left.checked_mul(right),
        }
    }

    pub(super) fn wrapping_i32(self, left: i32, right: i32) -> i32 {
        match self {
            Self::Add => left.wrapping_add(right),
            Self::Sub => left.wrapping_sub(right),
            Self::Mul => left.wrapping_mul(right),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum I32ComparisonPrimitive {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl I32ComparisonPrimitive {
    pub(super) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "eq" => Some(Self::Eq),
            "ne" => Some(Self::Ne),
            "lt" => Some(Self::Lt),
            "le" => Some(Self::Le),
            "gt" => Some(Self::Gt),
            "ge" => Some(Self::Ge),
            _ => None,
        }
    }

    pub(super) const fn relation_op(self) -> ResourceI32RelationOp {
        match self {
            Self::Eq => ResourceI32RelationOp::Eq,
            Self::Ne => ResourceI32RelationOp::Ne,
            Self::Lt => ResourceI32RelationOp::Lt,
            Self::Le => ResourceI32RelationOp::Le,
            Self::Gt => ResourceI32RelationOp::Gt,
            Self::Ge => ResourceI32RelationOp::Ge,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BooleanPrimitive {
    And,
    Or,
}

impl BooleanPrimitive {
    pub(super) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            _ => None,
        }
    }
}

use super::model::{ResourceCallTarget, ResourceI32RelationOp};
pub(super) use crate::scalar_primitives::{
    BooleanPrimitive, I32ArithmeticPrimitive, I32ComparisonPrimitive,
};

impl I32ArithmeticPrimitive {
    pub(super) fn from_resource_call_target(target: &ResourceCallTarget) -> Option<Self> {
        match target {
            ResourceCallTarget::Builtin { name } | ResourceCallTarget::User { name, .. } => {
                Self::from_symbol(name)
            }
            ResourceCallTarget::Trait { method, .. } => Self::from_symbol(method.as_str()),
        }
    }
}

impl I32ComparisonPrimitive {
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

use crate::hir::FuncRef;
use crate::runtime_helpers::helper_base_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum I32ArithmeticPrimitive {
    Add,
    Sub,
    Mul,
}

impl I32ArithmeticPrimitive {
    pub(crate) fn from_symbol(name: &str) -> Option<Self> {
        Self::from_base_name(helper_base_name(name))
    }

    pub(crate) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "mul" => Some(Self::Mul),
            _ => None,
        }
    }

    pub(crate) const fn base_name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
        }
    }

    pub(crate) fn from_codegen_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "add" => Some(Self::Add),
            _ => None,
        }
    }

    pub(crate) const fn codegen_intrinsic_name(self) -> Option<&'static str> {
        match self {
            Self::Add => Some("add"),
            Self::Sub | Self::Mul => None,
        }
    }

    pub(crate) const fn codegen_argument_count(self) -> Option<usize> {
        match self {
            Self::Add => Some(2),
            Self::Sub | Self::Mul => None,
        }
    }

    pub(crate) fn from_func_ref(callee: &FuncRef) -> Option<Self> {
        match callee {
            FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Self::from_symbol(name),
            FuncRef::Trait { .. } => None,
        }
    }

    pub(crate) fn checked_i64(self, left: i64, right: i64) -> Option<i64> {
        match self {
            Self::Add => left.checked_add(right),
            Self::Sub => left.checked_sub(right),
            Self::Mul => left.checked_mul(right),
        }
    }

    pub(crate) fn wrapping_i32(self, left: i32, right: i32) -> i32 {
        match self {
            Self::Add => left.wrapping_add(right),
            Self::Sub => left.wrapping_sub(right),
            Self::Mul => left.wrapping_mul(right),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum I32ComparisonPrimitive {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl I32ComparisonPrimitive {
    pub(crate) fn from_base_name(base: &str) -> Option<Self> {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BooleanPrimitive {
    And,
    Or,
}

impl BooleanPrimitive {
    pub(crate) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BooleanPrimitive, I32ArithmeticPrimitive, I32ComparisonPrimitive};

    #[test]
    fn i32_arithmetic_source_spellings_round_trip() {
        for (name, primitive) in [
            ("add", I32ArithmeticPrimitive::Add),
            ("sub", I32ArithmeticPrimitive::Sub),
            ("mul", I32ArithmeticPrimitive::Mul),
        ] {
            assert_eq!(
                I32ArithmeticPrimitive::from_base_name(name),
                Some(primitive)
            );
            assert_eq!(I32ArithmeticPrimitive::from_symbol(name), Some(primitive));
            assert_eq!(primitive.base_name(), name);
        }
        assert_eq!(I32ArithmeticPrimitive::from_base_name("div"), None);
    }

    #[test]
    fn backend_i32_arithmetic_intrinsics_are_explicit_subset() {
        assert_eq!(
            I32ArithmeticPrimitive::from_codegen_intrinsic_name("add"),
            Some(I32ArithmeticPrimitive::Add)
        );
        assert_eq!(
            I32ArithmeticPrimitive::Add.codegen_intrinsic_name(),
            Some("add")
        );
        assert_eq!(
            I32ArithmeticPrimitive::Add.codegen_argument_count(),
            Some(2)
        );
        assert_eq!(
            I32ArithmeticPrimitive::from_codegen_intrinsic_name("sub"),
            None
        );
        assert_eq!(I32ArithmeticPrimitive::Sub.codegen_intrinsic_name(), None);
        assert_eq!(I32ArithmeticPrimitive::Mul.codegen_argument_count(), None);
    }

    #[test]
    fn i32_comparison_source_spellings_round_trip() {
        for (name, primitive) in [
            ("eq", I32ComparisonPrimitive::Eq),
            ("ne", I32ComparisonPrimitive::Ne),
            ("lt", I32ComparisonPrimitive::Lt),
            ("le", I32ComparisonPrimitive::Le),
            ("gt", I32ComparisonPrimitive::Gt),
            ("ge", I32ComparisonPrimitive::Ge),
        ] {
            assert_eq!(
                I32ComparisonPrimitive::from_base_name(name),
                Some(primitive)
            );
        }
        assert_eq!(I32ComparisonPrimitive::from_base_name("cmp"), None);
    }

    #[test]
    fn boolean_source_spellings_round_trip() {
        assert_eq!(
            BooleanPrimitive::from_base_name("and"),
            Some(BooleanPrimitive::And)
        );
        assert_eq!(
            BooleanPrimitive::from_base_name("or"),
            Some(BooleanPrimitive::Or)
        );
        assert_eq!(BooleanPrimitive::from_base_name("xor"), None);
    }
}

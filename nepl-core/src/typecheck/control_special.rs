#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlSpecialFunction {
    If,
    While,
}

impl ControlSpecialFunction {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "if" => Some(Self::If),
            "while" => Some(Self::While),
            _ => None,
        }
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::If => "if",
            Self::While => "while",
        }
    }
}

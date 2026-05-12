use crate::ast::Directive;
use crate::compiler::{BuildProfile, CompileTarget};

pub(super) fn parse_variant_name(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.splitn(2, "::");
    let a = parts.next()?;
    let b = parts.next()?;
    Some((a, b))
}

pub(super) fn variant_member_tail(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

pub(super) fn parse_i32_literal(text: &str) -> Option<i32> {
    let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else {
        (false, text)
    };
    let (radix, digits) = if let Some(rest) = digits.strip_prefix("0x") {
        (16, rest)
    } else if let Some(rest) = digits.strip_prefix("0X") {
        (16, rest)
    } else {
        (10, digits)
    };
    if digits.is_empty() {
        return None;
    }
    let unsigned = i128::from_str_radix(digits, radix).ok()?;
    let signed = if neg { -unsigned } else { unsigned };
    i32::try_from(signed).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_member_tail_uses_last_separator() {
        assert_eq!(variant_member_tail("Result::Ok"), "Ok");
        assert_eq!(variant_member_tail("module::Result::Ok"), "Ok");
        assert_eq!(variant_member_tail("Ok"), "Ok");
    }
}

pub(super) fn gate_allows(
    d: &Directive,
    target: CompileTarget,
    active_profile: BuildProfile,
) -> Option<bool> {
    crate::target_gate::directive_gate_allows(d, target, active_profile)
}

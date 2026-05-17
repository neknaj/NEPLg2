extern crate alloc;

use alloc::string::String;

use crate::hir::{HirExpr, HirExprKind};

use super::address_projection::non_negative_i32_literal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AggregateFieldSelector<'a> {
    Index(usize),
    Name(&'a str),
    Unsupported,
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

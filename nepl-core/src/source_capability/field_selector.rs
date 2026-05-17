use crate::ast::{Literal, PrefixItem, Symbol};

pub(super) fn field_selector_after_call_head(index: usize, items: &[PrefixItem]) -> Option<&str> {
    let selector_index = match items.get(index + 1) {
        Some(PrefixItem::Symbol(Symbol::AddrOf { .. } | Symbol::Deref(_))) => index + 3,
        Some(_) => index + 2,
        None => return None,
    };
    prefix_item_string_literal(items.get(selector_index)?)
}

fn prefix_item_string_literal(item: &PrefixItem) -> Option<&str> {
    match item {
        PrefixItem::Literal(Literal::Str(value), _) => Some(value.as_str()),
        _ => None,
    }
}

use crate::ast::{PrefixItem, Symbol};

pub(super) fn explicit_constructor_symbol(
    item: &PrefixItem,
    has_following_payload: bool,
) -> Option<&str> {
    if !has_following_payload {
        return None;
    }

    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, type_args, _)) => {
            if type_args.is_empty() {
                None
            } else {
                Some(ident.name.as_str())
            }
        }
        PrefixItem::Symbol(
            Symbol::Let { .. }
            | Symbol::Set { .. }
            | Symbol::If(_)
            | Symbol::While(_)
            | Symbol::AddrOf { .. }
            | Symbol::Deref(_),
        )
        | PrefixItem::Literal(_, _)
        | PrefixItem::Block(_, _)
        | PrefixItem::Match(_, _)
        | PrefixItem::Tuple(_, _)
        | PrefixItem::Group(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Intrinsic(_, _) => None,
    }
}

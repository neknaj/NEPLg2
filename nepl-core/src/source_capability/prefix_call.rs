use crate::ast::{PrefixItem, Symbol};

/// Tracks where a prefix item can begin a nested call.
///
/// Source capabilities use this only as syntactic evidence. Typed ownership,
/// effect, and memory-safety proof still belongs to typecheck and Resource IR.
#[derive(Debug, Clone, Copy)]
pub(super) struct PrefixCallHead {
    current_item_can_start_call: bool,
}

impl PrefixCallHead {
    pub(super) fn new() -> Self {
        Self {
            current_item_can_start_call: true,
        }
    }

    pub(super) fn current_item_can_start_call(&self) -> bool {
        self.current_item_can_start_call
    }

    pub(super) fn observe_item(&mut self, item: &PrefixItem) {
        self.current_item_can_start_call = prefix_item_allows_following_call_head(item);
    }
}

fn prefix_item_allows_following_call_head(item: &PrefixItem) -> bool {
    match item {
        PrefixItem::TypeAnnotation(_, _) | PrefixItem::Pipe(_) => true,
        PrefixItem::Symbol(symbol) => symbol_allows_following_call_head(symbol),
        PrefixItem::Literal(_, _)
        | PrefixItem::Block(_, _)
        | PrefixItem::Match(_, _)
        | PrefixItem::Tuple(_, _)
        | PrefixItem::Group(_, _)
        | PrefixItem::Intrinsic(_, _) => false,
    }
}

fn symbol_allows_following_call_head(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Let { .. }
        | Symbol::Set { .. }
        | Symbol::If(_)
        | Symbol::While(_)
        | Symbol::AddrOf { .. }
        | Symbol::Deref(_) => true,
        Symbol::Ident(_, _, _) => false,
    }
}

use alloc::vec::Vec;

use crate::ast::Ident;
use crate::span::Span;
use crate::types::TypeKind;

use super::env::Binding;
use super::BlockChecker;

pub(super) struct CallableApplyLookup {
    pub(super) bindings: Vec<Binding>,
    pub(super) has_function_value_binding: bool,
}

impl<'a> BlockChecker<'a> {
    pub(super) fn lookup_callable_apply_bindings(
        &self,
        name: &str,
        symbol_resolved: bool,
        span: Span,
    ) -> CallableApplyLookup {
        let qualified_call = if symbol_resolved {
            None
        } else {
            self.lookup_qualified_bindings(&Ident {
                name: alloc::string::String::from(name),
                span,
            })
        };
        let bindings = if symbol_resolved {
            self.env
                .lookup_all_callables_by_symbol(name)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        } else if let Some((_, qualified)) = &qualified_call {
            qualified.clone()
        } else {
            self.env
                .lookup_all_callables(name)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        };
        let has_function_value_binding = if symbol_resolved || qualified_call.is_some() {
            false
        } else {
            self.env
                .lookup_value(name)
                .map(|b| {
                    let rty = self.ctx.resolve_id(b.ty);
                    matches!(self.ctx.get(rty), TypeKind::Function { .. })
                })
                .unwrap_or(false)
        };
        CallableApplyLookup {
            bindings,
            has_function_value_binding,
        }
    }
}

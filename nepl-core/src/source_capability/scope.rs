use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::ast::{Ident, MatchPattern, Module, PrefixItem, Stmt, Symbol};
use crate::qualified_name::split_leading_qualifier;
use crate::source_capability::binding::{bind_symbol_kind, SourceCapabilityBindingKind};

#[derive(Debug, Clone, Default)]
pub(super) struct SourceCapabilityScope {
    shadowed_symbols: BTreeMap<String, SourceCapabilityBindingKind>,
}

impl SourceCapabilityScope {
    pub(super) fn from_module(module: &Module) -> Self {
        let mut scope = Self::default();
        for stmt in &module.root.items {
            scope.bind_top_level_stmt(stmt);
        }
        scope
    }

    pub(super) fn with_params(&self, params: &[Ident]) -> Self {
        let mut scope = self.clone();
        for param in params {
            scope.bind(&param.name);
        }
        scope
    }

    pub(super) fn bind_match_pattern(&mut self, pattern: &MatchPattern) {
        if let MatchPattern::Variant {
            bind: Some(bind), ..
        } = pattern
        {
            self.bind(&bind.name);
        }
    }

    pub(super) fn bind_stmt_locals(&mut self, stmt: &Stmt) {
        if let Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) = stmt {
            for item in &expr.items {
                if let PrefixItem::Symbol(Symbol::Let { name, .. }) = item {
                    self.bind(&name.name);
                }
            }
        }
    }

    pub(super) fn shadow_kind_symbol_or_qualifier(
        &self,
        symbol: &str,
    ) -> Option<SourceCapabilityBindingKind> {
        split_leading_qualifier(symbol)
            .map(|(qualifier, _)| self.shadow_kind(qualifier))
            .unwrap_or_else(|| self.shadow_kind(symbol))
    }

    fn shadow_kind(&self, name: &str) -> Option<SourceCapabilityBindingKind> {
        self.shadowed_symbols.get(name).copied()
    }
    fn bind_top_level_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FnDef(def) => self.bind_kind(
                &def.name.name,
                SourceCapabilityBindingKind::TopLevelCallable,
            ),
            Stmt::FnAlias(alias) => self.bind_kind(
                &alias.name.name,
                SourceCapabilityBindingKind::TopLevelCallable,
            ),
            Stmt::Impl(def) => {
                for method in &def.methods {
                    self.bind_kind(&method.name.name, SourceCapabilityBindingKind::ImplMethod);
                }
            }
            Stmt::Directive(_)
            | Stmt::StructDef(_)
            | Stmt::EnumDef(_)
            | Stmt::Trait(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_)
            | Stmt::Expr(_)
            | Stmt::ExprSemi(_, _) => {}
        }
    }

    fn bind(&mut self, name: &str) {
        self.bind_kind(name, SourceCapabilityBindingKind::LocalValue);
    }

    fn bind_kind(&mut self, name: &str, kind: SourceCapabilityBindingKind) {
        bind_symbol_kind(&mut self.shadowed_symbols, name, kind);
    }
}

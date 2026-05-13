use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::ast::{Ident, MatchPattern, Module, PrefixItem, Stmt, Symbol};

#[derive(Debug, Clone, Default)]
pub(super) struct RawMemoryBoundaryScope {
    shadowed_symbols: BTreeSet<String>,
}

impl RawMemoryBoundaryScope {
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

    pub(super) fn shadows(&self, name: &str) -> bool {
        self.shadowed_symbols.contains(name)
    }

    fn bind_top_level_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FnDef(def) => self.bind(&def.name.name),
            Stmt::FnAlias(alias) => self.bind(&alias.name.name),
            Stmt::StructDef(def) => self.bind(&def.name.name),
            Stmt::EnumDef(def) => self.bind(&def.name.name),
            Stmt::Trait(def) => self.bind(&def.name.name),
            Stmt::Directive(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_)
            | Stmt::Impl(_)
            | Stmt::Expr(_)
            | Stmt::ExprSemi(_, _) => {}
        }
    }

    fn bind(&mut self, name: &str) {
        self.shadowed_symbols.insert(String::from(name));
    }
}

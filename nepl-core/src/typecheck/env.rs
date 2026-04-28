use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::builtins::BuiltinKind;
use crate::resolve::DefId;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::{same_function_signature, FieldAccessorKind, TraitBoundRef};
// ---------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(super) struct Binding {
    pub(super) name: String,
    pub(super) ty: TypeId,
    pub(super) mutable: bool,
    pub(super) no_shadow: bool,
    pub(super) defined: bool,
    pub(super) span: Span,
    pub(super) kind: BindingKind,
}

#[derive(Debug, Clone)]
pub(super) enum BindingKind {
    Var,
    Func {
        def_id: Option<DefId>,
        symbol: String,
        effect: Effect,
        arity: usize,
        builtin: Option<BuiltinKind>,
        field_accessor: Option<FieldAccessorKind>,
        type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>,
        captures: Vec<(String, TypeId)>,
    },
}

impl BindingKind {
    pub(super) fn is_var(&self) -> bool {
        matches!(self, BindingKind::Var)
    }

    pub(super) fn is_callable(&self) -> bool {
        matches!(self, BindingKind::Func { .. })
    }
}

#[derive(Debug, Default)]
pub(super) struct Scope {
    pub(super) values: Vec<Binding>,
    pub(super) callables: Vec<Binding>,
}

#[derive(Debug)]
pub(super) struct Env {
    pub(super) scopes: Vec<Scope>,
}

impl Env {
    pub(super) fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub(super) fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(super) fn insert_into_scope(scope: &mut Scope, binding: Binding) {
        if binding.kind.is_var() {
            scope.values.push(binding);
        } else {
            scope.callables.push(binding);
        }
    }

    pub(super) fn insert_global(&mut self, binding: Binding) {
        if let Some(scope) = self.scopes.first_mut() {
            Self::insert_into_scope(scope, binding);
        }
    }

    pub(super) fn remove_duplicate_func(&mut self, name: &str, ty: TypeId, ctx: &TypeCtx) {
        if let Some(scope) = self.scopes.first_mut() {
            scope.callables.retain(|b| {
                if b.name != name || !b.kind.is_callable() {
                    return true;
                }
                !same_function_signature(ctx, b.ty, ty)
            });
        }
    }

    pub(super) fn insert_local(&mut self, binding: Binding) -> Result<(), ()> {
        if let Some(scope) = self.scopes.last_mut() {
            let has_value = scope.values.iter().any(|b| b.name == binding.name);
            if binding.kind.is_var() {
                if has_value {
                    return Err(());
                }
                scope.values.push(binding);
            } else {
                if has_value {
                    return Err(());
                }
                scope.callables.push(binding);
            }
        }
        Ok(())
    }

    pub(super) fn lookup_current_value(&self, name: &str) -> Option<&Binding> {
        self.scopes
            .last()
            .and_then(|scope| scope.values.iter().rev().find(|b| b.name == name))
    }

    pub(super) fn lookup_any_defined(&self, name: &str) -> Option<&Binding> {
        // When resolving identifiers for reading, skip hoisted bindings
        // that are not yet defined. This prevents the RHS of a hoisted
        // `let` from accidentally seeing the placeholder binding.
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
                .or_else(|| {
                    scope
                        .callables
                        .iter()
                        .rev()
                        .find(|b| b.name == name && b.defined)
                })
            {
                return Some(b);
            }
        }
        None
    }

    pub(super) fn lookup_all_any_defined(&self, name: &str) -> Vec<&Binding> {
        for scope in self.scopes.iter().rev() {
            let mut items: Vec<&Binding> = scope
                .values
                .iter()
                .filter(|b| b.name == name && b.defined)
                .collect();
            items.extend(
                scope
                    .callables
                    .iter()
                    .filter(|b| b.name == name && b.defined),
            );
            if !items.is_empty() {
                return items;
            }
        }
        Vec::new()
    }

    pub(super) fn lookup_value(&self, name: &str) -> Option<&Binding> {
        self.lookup_all_any_defined(name)
            .into_iter()
            .find(|b| matches!(b.kind, BindingKind::Var))
    }

    pub(super) fn lookup_value_with_scope(&self, name: &str) -> Option<(&Binding, usize)> {
        for idx in (0..self.scopes.len()).rev() {
            let scope = &self.scopes[idx];
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
            {
                return Some((b, idx));
            }
        }
        None
    }

    pub(super) fn lookup_all_callables(&self, name: &str) -> Vec<&Binding> {
        let mut items = Vec::new();
        for scope in self.scopes.iter().rev() {
            for b in scope
                .callables
                .iter()
                .filter(|b| b.name == name && b.defined)
            {
                items.push(b);
            }
        }
        items
    }

    pub(super) fn lookup_all_callables_by_symbol(&self, symbol: &str) -> Vec<&Binding> {
        let mut items = Vec::new();
        for scope in self.scopes.iter().rev() {
            for b in scope.callables.iter().filter(|b| {
                b.defined
                    && matches!(
                        &b.kind,
                        BindingKind::Func { symbol: s, .. } if s == symbol
                    )
            }) {
                items.push(b);
            }
        }
        items
    }

    pub(super) fn update_local_function_binding(
        &mut self,
        _ctx: &TypeCtx,
        name: &str,
        span: Span,
        ty: TypeId,
        captures_new: Vec<(String, TypeId)>,
    ) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            for binding in scope.callables.iter_mut().rev() {
                if binding.name != name || binding.span != span {
                    continue;
                }
                binding.ty = ty;
                if let BindingKind::Func { captures, .. } = &mut binding.kind {
                    *captures = captures_new.clone();
                }
                return true;
            }
        }
        false
    }

    /// 同名候補から型シグネチャ一致の関数シンボルを返す。
    ///
    /// typecheck 本体と HIR 生成で関数名決定ロジックを共有し、
    /// hoist した symbol と最終的な HIR 名の不整合を防ぐ。
    pub(super) fn lookup_func_symbol(
        &self,
        name: &str,
        ty: TypeId,
        ctx: &TypeCtx,
    ) -> Option<String> {
        for binding in self.lookup_all_callables(name) {
            if let BindingKind::Func { symbol, .. } = &binding.kind {
                if same_function_signature(ctx, binding.ty, ty) {
                    return Some(symbol.clone());
                }
            }
        }
        None
    }

    pub(super) fn lookup_outer_defined(&self, name: &str) -> Option<&Binding> {
        if self.scopes.len() <= 1 {
            return None;
        }
        for scope in self.scopes[..self.scopes.len() - 1].iter().rev() {
            if let Some(binding) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
                .or_else(|| {
                    scope
                        .callables
                        .iter()
                        .rev()
                        .find(|b| b.name == name && b.defined)
                })
            {
                return Some(binding);
            }
        }
        None
    }

    pub(super) fn lookup_any(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name)
                .or_else(|| scope.callables.iter().rev().find(|b| b.name == name))
            {
                return Some(b);
            }
        }
        None
    }

    pub(super) fn lookup_mut(&mut self, name: &str) -> Option<&mut Binding> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(pos) = scope.values.iter().rposition(|b| b.name == name) {
                return scope.values.get_mut(pos);
            }
        }
        None
    }
}

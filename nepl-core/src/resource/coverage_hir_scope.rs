extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirFunction, HirModule};

use super::drop_call_identity::DropCallIdentityIndex;

pub(super) struct HirCoverageContext<'a> {
    module: Option<&'a HirModule>,
    callable_names: BTreeSet<String>,
    drop_calls: DropCallIdentityIndex,
    local_scopes: Vec<BTreeSet<String>>,
}

impl<'a> HirCoverageContext<'a> {
    pub(super) fn new(function: &HirFunction, module: &'a HirModule) -> Self {
        let drop_calls = DropCallIdentityIndex::new(module);
        let mut callable_names = BTreeSet::new();
        for function in &module.functions {
            callable_names.insert(function.name.clone());
            callable_names.insert(function.origin_name.clone());
        }
        for extern_fn in &module.externs {
            callable_names.insert(extern_fn.local_name.clone());
        }

        let mut root_scope = BTreeSet::new();
        for param in &function.params {
            root_scope.insert(param.name.clone());
        }

        Self {
            module: Some(module),
            callable_names,
            drop_calls,
            local_scopes: alloc::vec![root_scope],
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            module: None,
            callable_names: BTreeSet::new(),
            drop_calls: DropCallIdentityIndex::default(),
            local_scopes: alloc::vec![BTreeSet::new()],
        }
    }

    pub(super) fn push_scope(&mut self) {
        self.local_scopes.push(BTreeSet::new());
    }

    pub(super) fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    pub(super) fn declare_local(&mut self, name: &str) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(String::from(name));
        }
    }

    pub(super) fn var_is_callable_value_reference(&self, name: &str) -> bool {
        !self.local_defined(name) && self.callable_names.contains(name)
    }

    pub(super) fn function(&self, name: &str) -> Option<&'a HirFunction> {
        self.module?
            .functions
            .iter()
            .find(|function| function.name == name)
    }

    pub(super) fn call_is_explicit_drop(&self, callee: &FuncRef) -> bool {
        self.drop_calls.call_is_explicit_drop(callee)
    }

    fn local_defined(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

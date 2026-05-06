extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{HirFunction, HirModule};

pub(super) struct HirCoverageContext {
    callable_names: BTreeSet<String>,
    local_scopes: Vec<BTreeSet<String>>,
}

impl HirCoverageContext {
    pub(super) fn new(function: &HirFunction, module: &HirModule) -> Self {
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
            callable_names,
            local_scopes: alloc::vec![root_scope],
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            callable_names: BTreeSet::new(),
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

    fn local_defined(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

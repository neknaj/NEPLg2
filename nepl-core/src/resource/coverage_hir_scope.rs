extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirFunction, HirModule};

use super::drop_call_identity::DropCallIdentityIndex;

pub(super) struct HirCoverageModuleIndex<'a> {
    functions: BTreeMap<&'a str, &'a HirFunction>,
    callable_names: BTreeSet<&'a str>,
    drop_calls: DropCallIdentityIndex,
}

impl<'a> HirCoverageModuleIndex<'a> {
    pub(super) fn new(module: &'a HirModule) -> Self {
        let mut functions = BTreeMap::new();
        let mut callable_names = BTreeSet::new();
        for function in &module.functions {
            functions.entry(function.name.as_str()).or_insert(function);
            callable_names.insert(function.name.as_str());
            callable_names.insert(function.origin_name.as_str());
        }
        for extern_fn in &module.externs {
            callable_names.insert(extern_fn.local_name.as_str());
        }

        Self {
            functions,
            callable_names,
            drop_calls: DropCallIdentityIndex::new(module),
        }
    }
}

pub(super) struct HirCoverageContext<'a> {
    module_index: Option<&'a HirCoverageModuleIndex<'a>>,
    local_scopes: Vec<BTreeSet<String>>,
}

impl<'a> HirCoverageContext<'a> {
    pub(super) fn new(
        function: &HirFunction,
        module_index: &'a HirCoverageModuleIndex<'a>,
    ) -> Self {
        let mut root_scope = BTreeSet::new();
        for param in &function.params {
            root_scope.insert(param.name.clone());
        }

        Self {
            module_index: Some(module_index),
            local_scopes: alloc::vec![root_scope],
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            module_index: None,
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
        !self.local_defined(name)
            && self
                .module_index
                .is_some_and(|index| index.callable_names.contains(name))
    }

    pub(super) fn function(&self, name: &str) -> Option<&'a HirFunction> {
        self.module_index?.functions.get(name).copied()
    }

    pub(super) fn call_is_explicit_drop(&self, callee: &FuncRef) -> bool {
        self.module_index
            .is_some_and(|index| index.drop_calls.call_is_explicit_drop(callee))
    }

    fn local_defined(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

use crate::ast::TraitCapability;
use crate::hir::{FuncRef, HirModule};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DropCallIdentityIndex {
    drop_impl_functions: BTreeSet<String>,
    drop_trait_methods: BTreeSet<(String, String)>,
}

impl DropCallIdentityIndex {
    pub(super) fn new(module: &HirModule) -> Self {
        let mut drop_trait_methods = BTreeSet::new();
        for tr in &module.traits {
            if !tr
                .capabilities
                .iter()
                .any(|cap| *cap == TraitCapability::Drop)
            {
                continue;
            }
            if !tr.methods.contains_key("drop") {
                continue;
            }
            drop_trait_methods.insert((tr.name.clone(), String::from("drop")));
        }

        let mut drop_impl_origins = BTreeSet::new();
        for imp in &module.impls {
            for method in &imp.methods {
                if drop_trait_methods.contains(&(
                    imp.trait_application.trait_id.as_str().to_string(),
                    method.name.clone(),
                )) {
                    drop_impl_origins.insert(method.func.name.clone());
                }
            }
        }

        let mut drop_impl_functions = BTreeSet::new();
        for function in &module.functions {
            if drop_impl_origins.contains(&function.name)
                || drop_impl_origins.contains(&function.origin_name)
            {
                drop_impl_functions.insert(function.name.clone());
            }
        }

        Self {
            drop_impl_functions,
            drop_trait_methods,
        }
    }

    pub(super) fn call_is_explicit_drop(&self, callee: &FuncRef) -> bool {
        match callee {
            FuncRef::Trait {
                application,
                method,
                ..
            } => self.drop_trait_methods.contains(&(
                application.trait_id.as_str().to_string(),
                method.as_str().to_string(),
            )),
            FuncRef::User(name, _, _) => self.drop_impl_functions.contains(name),
            FuncRef::Builtin(_) => false,
        }
    }
}

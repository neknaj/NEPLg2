use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::Ident;

use super::env::Binding;
use super::signature::same_function_signature;
use super::syntax_helpers::parse_variant_name;
use super::BlockChecker;

impl<'a> BlockChecker<'a> {
    pub(super) fn lookup_qualified_bindings(&self, id: &Ident) -> Option<(String, Vec<Binding>)> {
        let (ns, member) = parse_variant_name(&id.name)?;
        if self.enums.contains_key(ns) || self.traits.contains_key(ns) {
            return None;
        }
        let lookup_targets =
            self.import_resolution
                .qualified_lookup_names(id.span.file_id.0, ns, member)?;
        let mut seen = BTreeSet::new();
        let mut bindings = Vec::new();
        for (target_file, target_name) in lookup_targets {
            for binding in self.env.lookup_all_any_defined(&target_name) {
                if binding.span.file_id.0 != target_file {
                    continue;
                }
                let key = (
                    binding.span.file_id.0,
                    binding.span.start,
                    binding.span.end,
                    binding.name.clone(),
                );
                if seen.insert(key) {
                    bindings.push(binding.clone());
                }
            }
        }
        Some((member.to_string(), bindings))
    }

    pub(super) fn unqualified_lookup_names(&self, id: &Ident) -> Vec<String> {
        self.import_resolution
            .unqualified_lookup_names(id.span.file_id.0, &id.name)
    }

    pub(super) fn binding_is_visible_unqualified(&self, id: &Ident, binding: &Binding) -> bool {
        self.import_resolution.binding_is_visible_unqualified(
            id.span.file_id.0,
            &id.name,
            binding.span.file_id.0,
            &binding.name,
        )
    }

    fn binding_is_local_to_lookup_file(&self, id: &Ident, binding: &Binding) -> bool {
        binding.span.file_id == id.span.file_id
    }

    pub(super) fn lookup_all_unqualified_any_defined(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut local_items = Vec::new();
            let mut import_items = Vec::new();
            for name in &names {
                for binding in scope
                    .values
                    .iter()
                    .chain(scope.callables.iter())
                    .filter(|b| {
                        b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                    })
                {
                    if self.binding_is_local_to_lookup_file(id, binding) {
                        local_items.push(binding);
                    } else {
                        import_items.push(binding);
                    }
                }
            }
            if !local_items.is_empty() {
                return local_items;
            }
            if !import_items.is_empty() {
                return import_items;
            }
        }
        Vec::new()
    }

    pub(super) fn lookup_all_unqualified_callables(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut local_items = Vec::new();
            let mut import_items = Vec::new();
            for name in &names {
                for binding in scope.callables.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }) {
                    if self.binding_is_local_to_lookup_file(id, binding) {
                        local_items.push(binding);
                    } else {
                        import_items.push(binding);
                    }
                }
            }
            if !local_items.is_empty() {
                import_items.retain(|imported| {
                    !local_items.iter().any(|local| {
                        local.name == imported.name
                            && same_function_signature(self.ctx, local.ty, imported.ty)
                    })
                });
                local_items.extend(import_items);
                return local_items;
            }
            if !import_items.is_empty() {
                return import_items;
            }
        }
        Vec::new()
    }

    pub(super) fn lookup_unqualified_callable_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut imported = None;
            for name in &names {
                if let Some(binding) = scope.callables.iter().rev().find(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }) {
                    if self.binding_is_local_to_lookup_file(id, binding) {
                        return Some(binding);
                    }
                    if imported.is_none() {
                        imported = Some(binding);
                    }
                }
            }
            if imported.is_some() {
                return imported;
            }
        }
        None
    }

    pub(super) fn lookup_unqualified_value_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut imported = None;
            for name in &names {
                if let Some(binding) = scope
                    .values
                    .iter()
                    .rev()
                    .find(|b| b.name == *name && self.binding_is_visible_unqualified(id, b))
                {
                    if self.binding_is_local_to_lookup_file(id, binding) {
                        return Some(binding);
                    }
                    if imported.is_none() {
                        imported = Some(binding);
                    }
                }
            }
            if imported.is_some() {
                return imported;
            }
        }
        None
    }

    pub(super) fn lookup_unqualified_value_for_read(
        &self,
        id: &Ident,
        allow_undefined_nonmut: bool,
    ) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut imported = None;
            for name in &names {
                if let Some(binding) = scope.values.iter().rev().find(|b| {
                    if b.name != *name || !self.binding_is_visible_unqualified(id, b) {
                        return false;
                    }
                    b.defined || (allow_undefined_nonmut && !b.mutable)
                }) {
                    if self.binding_is_local_to_lookup_file(id, binding) {
                        return Some(binding);
                    }
                    if imported.is_none() {
                        imported = Some(binding);
                    }
                }
            }
            if imported.is_some() {
                return imported;
            }
        }
        None
    }
}

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::Ident;

use super::env::Binding;
use super::{parse_variant_name, BlockChecker};

impl<'a> BlockChecker<'a> {
    pub(super) fn lookup_qualified_bindings(&self, id: &Ident) -> Option<(String, Vec<Binding>)> {
        let (ns, member) = parse_variant_name(&id.name)?;
        if self.enums.contains_key(ns) || self.traits.contains_key(ns) {
            return None;
        }
        let target_files = self
            .import_resolution
            .qualified_targets_for_alias(id.span.file_id.0, ns)?;
        let bindings = self
            .env
            .lookup_all_any_defined(member)
            .into_iter()
            .filter(|b| target_files.contains(&b.span.file_id.0))
            .cloned()
            .collect::<Vec<_>>();
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

    pub(super) fn lookup_all_unqualified_any_defined(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut items = Vec::new();
            for name in &names {
                items.extend(scope.values.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
                items.extend(scope.callables.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
            }
            if !items.is_empty() {
                return items;
            }
        }
        Vec::new()
    }

    pub(super) fn lookup_all_unqualified_callables(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        let mut items = Vec::new();
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                items.extend(scope.callables.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
            }
        }
        items
    }

    pub(super) fn lookup_unqualified_callable_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                if let Some(binding) = scope.callables.iter().rev().find(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }) {
                    return Some(binding);
                }
            }
        }
        None
    }

    pub(super) fn lookup_unqualified_value_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                if let Some(binding) = scope
                    .values
                    .iter()
                    .rev()
                    .find(|b| b.name == *name && self.binding_is_visible_unqualified(id, b))
                {
                    return Some(binding);
                }
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
            for name in &names {
                if let Some(binding) = scope.values.iter().rev().find(|b| {
                    if b.name != *name || !self.binding_is_visible_unqualified(id, b) {
                        return false;
                    }
                    b.defined || (allow_undefined_nonmut && !b.mutable)
                }) {
                    return Some(binding);
                }
            }
        }
        None
    }
}

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;

use crate::ast::{Directive, ImportClause, ImportItem};
use crate::intrinsic_kinds::FieldAccessorKind;
use crate::qualified_name::split_leading_qualifier;
use crate::source_capability::import_path::SourceCapabilityImportModule;

#[derive(Debug, Default)]
pub(super) struct CoreFieldAccessorImports {
    field_aliases: BTreeSet<String>,
    field_unqualified_members: BTreeMap<String, FieldAccessorKind>,
    field_open_imported: bool,
}

impl CoreFieldAccessorImports {
    pub(super) fn collect_directive(&mut self, directive: &Directive) {
        let Directive::Import { path, clause, .. } = directive else {
            return;
        };
        if SourceCapabilityImportModule::from_path(path)
            != Some(SourceCapabilityImportModule::CoreField)
        {
            return;
        }
        match clause {
            ImportClause::DefaultAlias => {
                self.field_aliases.insert(String::from("field"));
            }
            ImportClause::Alias(alias) => {
                self.field_aliases.insert(alias.clone());
            }
            ImportClause::Open | ImportClause::Merge => {
                self.field_open_imported = true;
            }
            ImportClause::Selective(items) => {
                self.collect_selective_import(items);
            }
        }
    }

    pub(super) fn accessor_kind(&self, symbol: &str) -> Option<FieldAccessorKind> {
        match split_leading_qualifier(symbol) {
            Some((alias, member)) => self.accepts_qualified_accessor(alias, member),
            None => self.accepts_unqualified_accessor(symbol),
        }
    }

    fn collect_selective_import(&mut self, items: &[ImportItem]) {
        for item in items {
            if item.glob {
                self.field_open_imported = true;
                continue;
            }
            let Some(kind) = FieldAccessorKind::from_core_field_member_name(item.name.as_str())
            else {
                continue;
            };
            self.field_unqualified_members.insert(
                item.alias.clone().unwrap_or_else(|| item.name.clone()),
                kind,
            );
        }
    }

    fn accepts_qualified_accessor(&self, alias: &str, member: &str) -> Option<FieldAccessorKind> {
        self.field_aliases
            .contains(alias)
            .then(|| FieldAccessorKind::from_core_field_member_name(member))
            .flatten()
    }

    fn accepts_unqualified_accessor(&self, symbol: &str) -> Option<FieldAccessorKind> {
        if self.field_open_imported {
            if let Some(kind) = FieldAccessorKind::from_core_field_member_name(symbol) {
                return Some(kind);
            }
        }
        self.field_unqualified_members.get(symbol).copied()
    }
}

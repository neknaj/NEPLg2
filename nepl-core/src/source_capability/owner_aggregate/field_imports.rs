use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::ast::{Directive, ImportClause, ImportItem};
use crate::intrinsic_kinds::FieldAccessorKind;
use crate::qualified_name::split_leading_qualifier;

#[derive(Debug, Default)]
pub(super) struct CoreFieldAccessorImports {
    field_aliases: BTreeSet<String>,
    field_unqualified_members: BTreeSet<String>,
    field_open_imported: bool,
}

impl CoreFieldAccessorImports {
    pub(super) fn collect_directive(&mut self, directive: &Directive) {
        let Directive::Import { path, clause, .. } = directive else {
            return;
        };
        if !is_core_field_import_path(path) {
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

    pub(super) fn accepts_symbol(&self, symbol: &str) -> bool {
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
            if FieldAccessorKind::from_core_field_member_name(item.name.as_str()).is_none() {
                continue;
            }
            self.field_unqualified_members
                .insert(item.alias.clone().unwrap_or_else(|| item.name.clone()));
        }
    }

    fn accepts_qualified_accessor(&self, alias: &str, member: &str) -> bool {
        self.field_aliases.contains(alias)
            && FieldAccessorKind::from_core_field_member_name(member).is_some()
    }

    fn accepts_unqualified_accessor(&self, symbol: &str) -> bool {
        (self.field_open_imported
            && FieldAccessorKind::from_core_field_member_name(symbol).is_some())
            || self.field_unqualified_members.contains(symbol)
    }
}

fn is_core_field_import_path(path: &str) -> bool {
    path.strip_suffix(".nepl").unwrap_or(path) == "core/field"
}

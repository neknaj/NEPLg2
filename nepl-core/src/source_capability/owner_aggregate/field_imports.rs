use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::ast::{Directive, ImportClause, ImportItem};
use crate::qualified_name::split_leading_qualifier;

#[derive(Debug, Default)]
pub(super) struct CoreFieldAccessorImports {
    field_aliases: BTreeSet<String>,
    field_unqualified_members: BTreeSet<String>,
    field_open_imported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoreFieldAccessorMember {
    Get,
    GetRef,
    Put,
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
            if CoreFieldAccessorMember::from_name(item.name.as_str()).is_none() {
                continue;
            }
            self.field_unqualified_members
                .insert(item.alias.clone().unwrap_or_else(|| item.name.clone()));
        }
    }

    fn accepts_qualified_accessor(&self, alias: &str, member: &str) -> bool {
        self.field_aliases.contains(alias) && CoreFieldAccessorMember::from_name(member).is_some()
    }

    fn accepts_unqualified_accessor(&self, symbol: &str) -> bool {
        (self.field_open_imported && CoreFieldAccessorMember::from_name(symbol).is_some())
            || self.field_unqualified_members.contains(symbol)
    }
}

impl CoreFieldAccessorMember {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "get" => Some(Self::Get),
            "get_ref" => Some(Self::GetRef),
            "put" => Some(Self::Put),
            _ => None,
        }
    }
}

fn is_core_field_import_path(path: &str) -> bool {
    path.strip_suffix(".nepl").unwrap_or(path) == "core/field"
}

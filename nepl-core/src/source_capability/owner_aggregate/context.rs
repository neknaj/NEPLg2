use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::ast::{Block, FnBody, Module, Stmt};
use crate::intrinsic_kinds::FieldAccessorKind;

use super::field_imports::CoreFieldAccessorImports;

#[derive(Debug, Default)]
pub(in crate::source_capability) struct OwnerAggregateEvidenceContext {
    enum_variants: BTreeSet<String>,
    field_imports: CoreFieldAccessorImports,
}

impl OwnerAggregateEvidenceContext {
    pub(in crate::source_capability) fn from_module(module: &Module) -> Self {
        let mut context = Self::default();
        for directive in &module.directives {
            context.field_imports.collect_directive(directive);
        }
        context.collect_block(&module.root);
        context
    }

    pub(in crate::source_capability) fn is_enum_variant(&self, name: &str) -> bool {
        self.enum_variants.contains(name)
    }

    pub(in crate::source_capability) fn is_core_field_accessor_symbol(&self, symbol: &str) -> bool {
        self.core_field_accessor_kind(symbol).is_some()
    }

    pub(in crate::source_capability) fn core_field_accessor_kind(
        &self,
        symbol: &str,
    ) -> Option<FieldAccessorKind> {
        self.field_imports.accessor_kind(symbol)
    }

    fn collect_block(&mut self, block: &Block) {
        for stmt in &block.items {
            self.collect_stmt(stmt);
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Directive(directive) => {
                self.field_imports.collect_directive(directive);
            }
            Stmt::EnumDef(def) => {
                for variant in &def.variants {
                    self.enum_variants.insert(variant.name.name.clone());
                }
            }
            Stmt::FnDef(def) => {
                if let FnBody::Parsed(block) = &def.body {
                    self.collect_block(block);
                }
            }
            Stmt::Impl(def) => {
                for method in &def.methods {
                    if let FnBody::Parsed(block) = &method.body {
                        self.collect_block(block);
                    }
                }
            }
            Stmt::Expr(_)
            | Stmt::ExprSemi(_, _)
            | Stmt::FnAlias(_)
            | Stmt::StructDef(_)
            | Stmt::Trait(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_) => {}
        }
    }
}

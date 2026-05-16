use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::ast::{Block, Module, PrefixItem, Stmt, Symbol};
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::scope::SourceCapabilityScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum OwnerAggregateCapabilityEvidence {
    Constructor(String),
    FieldAccessor,
}

#[derive(Debug, Default)]
pub(super) struct OwnerAggregateEvidenceContext {
    enum_variants: BTreeSet<String>,
}

impl OwnerAggregateEvidenceContext {
    pub(super) fn from_module(module: &Module) -> Self {
        let mut context = Self::default();
        context.collect_block(&module.root);
        context
    }

    fn collect_block(&mut self, block: &Block) {
        for stmt in &block.items {
            self.collect_stmt(stmt);
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::EnumDef(def) => {
                for variant in &def.variants {
                    self.enum_variants.insert(variant.name.name.clone());
                }
            }
            Stmt::FnDef(def) => {
                if let crate::ast::FnBody::Parsed(block) = &def.body {
                    self.collect_block(block);
                }
            }
            Stmt::Impl(def) => {
                for method in &def.methods {
                    if let crate::ast::FnBody::Parsed(block) = &method.body {
                        self.collect_block(block);
                    }
                }
            }
            Stmt::Expr(_)
            | Stmt::ExprSemi(_, _)
            | Stmt::Directive(_)
            | Stmt::FnAlias(_)
            | Stmt::StructDef(_)
            | Stmt::Trait(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_) => {}
        }
    }

    fn is_enum_variant(&self, name: &str) -> bool {
        self.enum_variants.contains(name)
    }
}

pub(super) fn owner_aggregate_call_head_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
) -> Option<OwnerAggregateCapabilityEvidence> {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) if !scope.shadows(&ident.name) => {
            owner_aggregate_symbol_evidence(ident.name.as_str(), context)
        }
        _ => None,
    }
}

pub(super) fn owner_aggregate_symbol_evidence(
    symbol: &str,
    context: &OwnerAggregateEvidenceContext,
) -> Option<OwnerAggregateCapabilityEvidence> {
    let base = helper_base_name(symbol);
    match base {
        "get" | "get_ref" | "put" | "get_field" | "get_field_ref" => {
            Some(OwnerAggregateCapabilityEvidence::FieldAccessor)
        }
        _ => constructor_evidence_name(symbol, context)
            .map(OwnerAggregateCapabilityEvidence::Constructor),
    }
}

pub(super) fn owner_aggregate_intrinsic_evidence(
    symbol: &str,
) -> Option<OwnerAggregateCapabilityEvidence> {
    match helper_base_name(symbol) {
        "get_field" | "get_field_ref" => Some(OwnerAggregateCapabilityEvidence::FieldAccessor),
        _ => None,
    }
}

fn constructor_evidence_name(
    symbol: &str,
    context: &OwnerAggregateEvidenceContext,
) -> Option<String> {
    if crate::qualified_name::member_tail(symbol) != symbol {
        return None;
    }
    let base = helper_base_name(symbol);
    if context.is_enum_variant(base) {
        return None;
    }
    base.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        .then(|| String::from(base))
}

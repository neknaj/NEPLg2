use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::scope::SourceCapabilityScope;

#[derive(Debug, Default)]
struct OwnerAggregateEvidence {
    constructors: BTreeSet<String>,
    field_accessor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OwnerAggregateCapabilityEvidence {
    Constructor(String),
    FieldAccessor,
}

pub(crate) fn module_owner_aggregate_constructor_evidence(module: &Module) -> Vec<String> {
    collect_module_owner_aggregate_evidence(module)
        .constructors
        .into_iter()
        .collect()
}

pub(crate) fn module_has_owner_aggregate_field_evidence(module: &Module) -> bool {
    collect_module_owner_aggregate_evidence(module).field_accessor
}

fn collect_module_owner_aggregate_evidence(module: &Module) -> OwnerAggregateEvidence {
    let scope = SourceCapabilityScope::from_module(module);
    let mut evidence = OwnerAggregateEvidence::default();
    collect_block_owner_aggregate_evidence(&module.root, &scope, &mut evidence);
    evidence
}

fn collect_block_owner_aggregate_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        collect_stmt_owner_aggregate_evidence(stmt, &block_scope, evidence);
        block_scope.bind_stmt_locals(stmt);
    }
}

fn collect_stmt_owner_aggregate_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            collect_fn_body_owner_aggregate_evidence(&def.body, &fn_scope, evidence);
        }
        Stmt::Impl(def) => {
            for method in &def.methods {
                let method_scope = scope.with_params(&method.params);
                collect_fn_body_owner_aggregate_evidence(&method.body, &method_scope, evidence);
            }
        }
        Stmt::FnAlias(alias) => {
            collect_symbol_owner_aggregate_evidence(alias.target.name.as_str(), scope, evidence);
        }
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            collect_expr_owner_aggregate_evidence(expr, scope, evidence);
        }
        Stmt::Directive(_)
        | Stmt::StructDef(_)
        | Stmt::EnumDef(_)
        | Stmt::Trait(_)
        | Stmt::Wasm(_)
        | Stmt::LlvmIr(_) => {}
    }
}

fn collect_fn_body_owner_aggregate_evidence(
    body: &FnBody,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    match body {
        FnBody::Parsed(block) => collect_block_owner_aggregate_evidence(block, scope, evidence),
        FnBody::Wasm(_) | FnBody::LlvmIr(_) => {}
    }
}

fn collect_expr_owner_aggregate_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    for item in &expr.items {
        collect_prefix_item_owner_aggregate_evidence(item, scope, evidence);
    }
}

fn collect_prefix_item_owner_aggregate_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) if !scope.shadows(&ident.name) => {
            collect_symbol_owner_aggregate_evidence(ident.name.as_str(), scope, evidence);
        }
        PrefixItem::Intrinsic(intrinsic, _) => {
            collect_builtin_owner_aggregate_evidence(intrinsic.name.as_str(), evidence);
            for expr in &intrinsic.args {
                collect_expr_owner_aggregate_evidence(expr, scope, evidence);
            }
        }
        PrefixItem::Block(block, _) => {
            collect_block_owner_aggregate_evidence(block, scope, evidence)
        }
        PrefixItem::Match(m, _) => {
            collect_expr_owner_aggregate_evidence(&m.scrutinee, scope, evidence);
            for arm in &m.arms {
                let mut arm_scope = scope.clone();
                arm_scope.bind_match_pattern(&arm.pattern);
                collect_block_owner_aggregate_evidence(&arm.body, &arm_scope, evidence);
            }
        }
        PrefixItem::Tuple(items, _) => {
            for expr in items {
                collect_expr_owner_aggregate_evidence(expr, scope, evidence);
            }
        }
        PrefixItem::Group(inner, _) => {
            collect_expr_owner_aggregate_evidence(inner, scope, evidence)
        }
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => {}
    }
}

fn collect_symbol_owner_aggregate_evidence(
    symbol: &str,
    scope: &SourceCapabilityScope,
    evidence: &mut OwnerAggregateEvidence,
) {
    if scope.shadows(symbol) {
        return;
    }
    collect_builtin_owner_aggregate_evidence(symbol, evidence);
}

fn collect_builtin_owner_aggregate_evidence(symbol: &str, evidence: &mut OwnerAggregateEvidence) {
    match owner_aggregate_evidence_from_symbol(symbol) {
        Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => {
            evidence.field_accessor = true;
        }
        Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
            evidence.constructors.insert(name);
        }
        None => {}
    }
}

fn owner_aggregate_evidence_from_symbol(symbol: &str) -> Option<OwnerAggregateCapabilityEvidence> {
    let base = helper_base_name(symbol);
    match base {
        "get" | "get_ref" | "put" | "get_field" | "get_field_ref" => {
            Some(OwnerAggregateCapabilityEvidence::FieldAccessor)
        }
        _ => constructor_evidence_name(symbol).map(OwnerAggregateCapabilityEvidence::Constructor),
    }
}

fn constructor_evidence_name(symbol: &str) -> Option<String> {
    if crate::qualified_name::member_tail(symbol) != symbol {
        return None;
    }
    let base = helper_base_name(symbol);
    base.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        .then(|| String::from(base))
}

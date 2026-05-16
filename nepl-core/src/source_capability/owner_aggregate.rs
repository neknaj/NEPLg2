use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

mod evidence;

use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt};
use crate::source_capability::scope::SourceCapabilityScope;

use self::evidence::{
    owner_aggregate_call_head_evidence, owner_aggregate_intrinsic_evidence,
    owner_aggregate_symbol_evidence, OwnerAggregateCapabilityEvidence,
    OwnerAggregateEvidenceContext,
};
use super::prefix_call::PrefixCallHead;

#[derive(Debug, Default)]
struct OwnerAggregateEvidence {
    constructors: BTreeSet<String>,
    field_accessor: bool,
}

pub(crate) fn module_owner_aggregate_constructor_evidence(module: &Module) -> Vec<String> {
    let evidence = collect_module_owner_aggregate_evidence(module);
    evidence.constructors.into_iter().collect()
}

pub(crate) fn module_has_owner_aggregate_field_evidence(module: &Module) -> bool {
    collect_module_owner_aggregate_evidence(module).field_accessor
}

fn collect_module_owner_aggregate_evidence(module: &Module) -> OwnerAggregateEvidence {
    let scope = SourceCapabilityScope::from_module(module);
    let context = OwnerAggregateEvidenceContext::from_module(module);
    let mut evidence = OwnerAggregateEvidence::default();
    collect_block_owner_aggregate_evidence(&module.root, &scope, &context, &mut evidence);
    evidence
}

fn collect_block_owner_aggregate_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        collect_stmt_owner_aggregate_evidence(stmt, &block_scope, context, evidence);
        block_scope.bind_stmt_locals(stmt);
    }
}

fn collect_stmt_owner_aggregate_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            collect_fn_body_owner_aggregate_evidence(&def.body, &fn_scope, context, evidence);
        }
        Stmt::Impl(def) => {
            for method in &def.methods {
                let method_scope = scope.with_params(&method.params);
                collect_fn_body_owner_aggregate_evidence(
                    &method.body,
                    &method_scope,
                    context,
                    evidence,
                );
            }
        }
        Stmt::FnAlias(alias) => {
            collect_symbol_evidence(alias.target.name.as_str(), scope, context, evidence);
        }
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            collect_expr_owner_aggregate_evidence(expr, scope, context, evidence);
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
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    match body {
        FnBody::Parsed(block) => {
            collect_block_owner_aggregate_evidence(block, scope, context, evidence)
        }
        FnBody::Wasm(_) | FnBody::LlvmIr(_) => {}
    }
}

fn collect_expr_owner_aggregate_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    let mut call_head = PrefixCallHead::new();
    for item in &expr.items {
        if call_head.current_item_can_start_call() {
            record_evidence(
                owner_aggregate_call_head_evidence(item, scope, context),
                evidence,
            );
        }
        collect_prefix_item_owner_aggregate_evidence(item, scope, context, evidence);
        call_head.observe_item(item);
    }
}

fn collect_prefix_item_owner_aggregate_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    match item {
        PrefixItem::Intrinsic(intrinsic, _) => {
            record_evidence(
                owner_aggregate_intrinsic_evidence(intrinsic.name.as_str()),
                evidence,
            );
            for expr in &intrinsic.args {
                collect_expr_owner_aggregate_evidence(expr, scope, context, evidence);
            }
        }
        PrefixItem::Block(block, _) => {
            collect_block_owner_aggregate_evidence(block, scope, context, evidence)
        }
        PrefixItem::Match(m, _) => {
            collect_expr_owner_aggregate_evidence(&m.scrutinee, scope, context, evidence);
            for arm in &m.arms {
                let mut arm_scope = scope.clone();
                arm_scope.bind_match_pattern(&arm.pattern);
                collect_block_owner_aggregate_evidence(&arm.body, &arm_scope, context, evidence);
            }
        }
        PrefixItem::Tuple(items, _) => {
            for expr in items {
                collect_expr_owner_aggregate_evidence(expr, scope, context, evidence);
            }
        }
        PrefixItem::Group(inner, _) => {
            collect_expr_owner_aggregate_evidence(inner, scope, context, evidence)
        }
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => {}
    }
}

fn collect_symbol_evidence(
    symbol: &str,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
    evidence: &mut OwnerAggregateEvidence,
) {
    if scope.shadows(symbol) {
        return;
    }
    record_evidence(owner_aggregate_symbol_evidence(symbol, context), evidence);
}

fn record_evidence(
    observed: Option<OwnerAggregateCapabilityEvidence>,
    evidence: &mut OwnerAggregateEvidence,
) {
    match observed {
        Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => evidence.field_accessor = true,
        Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
            evidence.constructors.insert(name);
        }
        None => {}
    }
}

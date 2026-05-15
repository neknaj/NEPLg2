use alloc::vec::Vec;

use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::effects::{
    raw_body_direct_callees, raw_body_memory_operations, raw_memory_op_from_name, RawBodyMemoryOp,
    RawMemoryOp,
};
use crate::hir::HirBody;
use crate::source_capability::scope::SourceCapabilityScope;

mod evidence;

use evidence::{RawMemoryBoundaryEvidence, RawMemoryEvidence};

pub(crate) fn module_has_raw_memory_boundary_evidence(module: &Module) -> bool {
    collect_module_raw_memory_evidence(module).structural_boundary
}

pub(crate) fn module_raw_memory_operation_evidence(module: &Module) -> Vec<RawMemoryOp> {
    collect_module_raw_memory_evidence(module)
        .operations
        .into_iter()
        .collect()
}

pub(crate) fn module_raw_body_memory_operation_evidence(module: &Module) -> Vec<RawBodyMemoryOp> {
    collect_module_raw_memory_evidence(module)
        .raw_body_operations
        .into_iter()
        .collect()
}

fn collect_module_raw_memory_evidence(module: &Module) -> RawMemoryEvidence {
    let scope = SourceCapabilityScope::from_module(module);
    let mut evidence = RawMemoryEvidence::default();
    collect_block_raw_memory_evidence(&module.root, &scope, &mut evidence);
    evidence
}

fn collect_block_raw_memory_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        collect_stmt_raw_memory_evidence(stmt, &block_scope, evidence);
        block_scope.bind_stmt_locals(stmt);
    }
}

fn collect_stmt_raw_memory_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            evidence.merge(collect_named_fn_raw_memory_evidence(
                def.name.name.as_str(),
                &def.body,
                &fn_scope,
            ));
        }
        Stmt::Impl(def) => {
            for method in &def.methods {
                let method_scope = scope.with_params(&method.params);
                evidence.merge(collect_named_fn_raw_memory_evidence(
                    method.name.name.as_str(),
                    &method.body,
                    &method_scope,
                ));
            }
        }
        Stmt::FnAlias(alias) => {
            collect_symbol_raw_memory_evidence(alias.target.name.as_str(), scope, evidence);
        }
        Stmt::Wasm(body) => collect_raw_body_evidence(HirBody::Wasm(body.clone()), evidence),
        Stmt::LlvmIr(body) => collect_raw_body_evidence(HirBody::LlvmIr(body.clone()), evidence),
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            collect_expr_raw_memory_evidence(expr, scope, evidence);
        }
        Stmt::Directive(_) | Stmt::StructDef(_) | Stmt::EnumDef(_) | Stmt::Trait(_) => {}
    }
}

fn collect_named_fn_raw_memory_evidence(
    name: &str,
    body: &FnBody,
    scope: &SourceCapabilityScope,
) -> RawMemoryEvidence {
    let mut evidence = RawMemoryEvidence::default();
    collect_fn_body_raw_memory_evidence(body, scope, &mut evidence);
    if evidence.has_any_evidence() {
        if let Some(operation) = raw_memory_op_from_name(name) {
            evidence.operations.insert(operation);
        }
    }
    evidence
}

fn collect_fn_body_raw_memory_evidence(
    body: &FnBody,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    match body {
        FnBody::Parsed(block) => collect_block_raw_memory_evidence(block, scope, evidence),
        FnBody::Wasm(body) => collect_raw_body_evidence(HirBody::Wasm(body.clone()), evidence),
        FnBody::LlvmIr(body) => collect_raw_body_evidence(HirBody::LlvmIr(body.clone()), evidence),
    }
}

fn collect_raw_body_evidence(body: HirBody, evidence: &mut RawMemoryEvidence) {
    for operation in raw_body_memory_operations(&body) {
        evidence.raw_body_operations.insert(operation);
    }
    for callee in raw_body_direct_callees(&body) {
        if let Some(operation) = raw_memory_op_from_name(&callee) {
            evidence.operations.insert(operation);
        }
    }
}

fn collect_expr_raw_memory_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    for item in &expr.items {
        collect_prefix_item_raw_memory_evidence(item, scope, evidence);
    }
}

fn collect_prefix_item_raw_memory_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) if !scope.shadows(&ident.name) => {
            collect_symbol_raw_memory_evidence(ident.name.as_str(), scope, evidence);
        }
        PrefixItem::Intrinsic(intrinsic, _) => {
            collect_builtin_raw_memory_evidence(intrinsic.name.as_str(), evidence);
            for expr in &intrinsic.args {
                collect_expr_raw_memory_evidence(expr, scope, evidence);
            }
        }
        PrefixItem::Block(block, _) => collect_block_raw_memory_evidence(block, scope, evidence),
        PrefixItem::Match(m, _) => {
            collect_expr_raw_memory_evidence(&m.scrutinee, scope, evidence);
            for arm in &m.arms {
                let mut arm_scope = scope.clone();
                arm_scope.bind_match_pattern(&arm.pattern);
                collect_block_raw_memory_evidence(&arm.body, &arm_scope, evidence);
            }
        }
        PrefixItem::Tuple(items, _) => {
            for expr in items {
                collect_expr_raw_memory_evidence(expr, scope, evidence);
            }
        }
        PrefixItem::Group(inner, _) => collect_expr_raw_memory_evidence(inner, scope, evidence),
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => {}
    }
}

fn collect_symbol_raw_memory_evidence(
    symbol: &str,
    scope: &SourceCapabilityScope,
    evidence: &mut RawMemoryEvidence,
) {
    if scope.shadows(symbol) {
        return;
    }
    collect_builtin_raw_memory_evidence(symbol, evidence);
}

fn collect_builtin_raw_memory_evidence(symbol: &str, evidence: &mut RawMemoryEvidence) {
    if RawMemoryBoundaryEvidence::from_symbol(symbol).is_some() {
        evidence.structural_boundary = true;
    }
    if let Some(operation) = raw_memory_op_from_name(symbol) {
        evidence.operations.insert(operation);
    }
}

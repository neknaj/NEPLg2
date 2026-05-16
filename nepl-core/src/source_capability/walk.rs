use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, StructDef, Symbol};
use crate::hir::HirBody;
use crate::source_capability::prefix_call::PrefixCallHead;
use crate::source_capability::scope::SourceCapabilityScope;

pub(super) trait SourceCapabilityObserver {
    fn observe_named_function_start(&mut self, _name: &str, _scope: &SourceCapabilityScope) {}

    fn observe_named_function_end(&mut self, _name: &str, _scope: &SourceCapabilityScope) {}

    fn observe_fn_alias_target(&mut self, _symbol: &str, _scope: &SourceCapabilityScope) {}

    fn observe_struct_definition(&mut self, _def: &StructDef) {}

    fn observe_call_head_symbol(&mut self, _symbol: &str, _scope: &SourceCapabilityScope) {}

    fn observe_intrinsic(&mut self, _name: &str, _scope: &SourceCapabilityScope) {}

    fn observe_raw_body(&mut self, _body: HirBody) {}
}

pub(super) fn walk_module_capability_evidence(
    module: &Module,
    observer: &mut impl SourceCapabilityObserver,
) {
    let scope = SourceCapabilityScope::from_module(module);
    walk_block_capability_evidence(&module.root, &scope, observer);
}

fn walk_block_capability_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        walk_stmt_capability_evidence(stmt, &block_scope, observer);
        block_scope.bind_stmt_locals(stmt);
    }
}

fn walk_stmt_capability_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            observer.observe_named_function_start(def.name.name.as_str(), &fn_scope);
            walk_fn_body_capability_evidence(&def.body, &fn_scope, observer);
            observer.observe_named_function_end(def.name.name.as_str(), &fn_scope);
        }
        Stmt::Impl(def) => {
            for method in &def.methods {
                let method_scope = scope.with_params(&method.params);
                observer.observe_named_function_start(method.name.name.as_str(), &method_scope);
                walk_fn_body_capability_evidence(&method.body, &method_scope, observer);
                observer.observe_named_function_end(method.name.name.as_str(), &method_scope);
            }
        }
        Stmt::FnAlias(alias) => {
            observer.observe_fn_alias_target(alias.target.name.as_str(), scope);
        }
        Stmt::StructDef(def) => observer.observe_struct_definition(def),
        Stmt::Wasm(body) => observer.observe_raw_body(HirBody::Wasm(body.clone())),
        Stmt::LlvmIr(body) => observer.observe_raw_body(HirBody::LlvmIr(body.clone())),
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            walk_expr_capability_evidence(expr, scope, observer);
        }
        Stmt::Directive(_) | Stmt::EnumDef(_) | Stmt::Trait(_) => {}
    }
}

fn walk_fn_body_capability_evidence(
    body: &FnBody,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match body {
        FnBody::Parsed(block) => walk_block_capability_evidence(block, scope, observer),
        FnBody::Wasm(body) => observer.observe_raw_body(HirBody::Wasm(body.clone())),
        FnBody::LlvmIr(body) => observer.observe_raw_body(HirBody::LlvmIr(body.clone())),
    }
}

fn walk_expr_capability_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    let mut call_head = PrefixCallHead::new();
    for item in &expr.items {
        if call_head.current_item_can_start_call() {
            observe_call_head_item(item, scope, observer);
        }
        walk_prefix_item_capability_evidence(item, scope, observer);
        call_head.observe_item(item);
    }
}

fn observe_call_head_item(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) => {
            observer.observe_call_head_symbol(ident.name.as_str(), scope);
        }
        PrefixItem::Symbol(
            Symbol::Let { .. }
            | Symbol::Set { .. }
            | Symbol::If(_)
            | Symbol::While(_)
            | Symbol::AddrOf { .. }
            | Symbol::Deref(_),
        )
        | PrefixItem::Literal(_, _)
        | PrefixItem::Block(_, _)
        | PrefixItem::Match(_, _)
        | PrefixItem::Tuple(_, _)
        | PrefixItem::Group(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Intrinsic(_, _) => {}
    }
}

fn walk_prefix_item_capability_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match item {
        PrefixItem::Intrinsic(intrinsic, _) => {
            observer.observe_intrinsic(intrinsic.name.as_str(), scope);
            for expr in &intrinsic.args {
                walk_expr_capability_evidence(expr, scope, observer);
            }
        }
        PrefixItem::Block(block, _) => walk_block_capability_evidence(block, scope, observer),
        PrefixItem::Match(m, _) => {
            walk_expr_capability_evidence(&m.scrutinee, scope, observer);
            for arm in &m.arms {
                let mut arm_scope = scope.clone();
                arm_scope.bind_match_pattern(&arm.pattern);
                walk_block_capability_evidence(&arm.body, &arm_scope, observer);
            }
        }
        PrefixItem::Tuple(items, _) => {
            for expr in items {
                walk_expr_capability_evidence(expr, scope, observer);
            }
        }
        PrefixItem::Group(inner, _) => walk_expr_capability_evidence(inner, scope, observer),
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => {}
    }
}

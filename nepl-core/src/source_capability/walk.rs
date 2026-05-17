use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, StructDef, Symbol};
use crate::hir::HirBody;
use crate::source_capability::constructor_position::explicit_constructor_symbol;
use crate::source_capability::field_selector::field_selector_after_call_head;
use crate::source_capability::prefix_call::{call_head_symbol, PrefixCallHead};
use crate::source_capability::scope::SourceCapabilityScope;
use crate::span::Span;

pub(super) trait SourceCapabilityObserver {
    fn observe_named_function_start(
        &mut self,
        _name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
    }
    fn observe_named_function_end(
        &mut self,
        _name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
    }
    fn observe_fn_alias_target(
        &mut self,
        _symbol: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
    }

    fn observe_struct_definition(&mut self, _def: &StructDef) {}

    fn observe_call_head_symbol(
        &mut self,
        _symbol: &str,
        _span: Span,
        _selector: Option<&str>,
        _scope: &SourceCapabilityScope,
    ) {
    }

    fn observe_explicit_constructor_symbol(
        &mut self,
        _symbol: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
    }

    fn observe_intrinsic(
        &mut self,
        _name: &str,
        _args: &[PrefixExpr],
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
    }

    fn observe_raw_body(&mut self, _body: HirBody, _span: Span) {}
}

pub(super) fn walk_module_capability_evidence(
    module: &Module,
    observer: &mut impl SourceCapabilityObserver,
) {
    let scope = SourceCapabilityScope::from_module(module);
    walk_block_capability_evidence(&module.root, None, &scope, observer);
}

fn walk_block_capability_evidence(
    block: &Block,
    raw_body_site: Option<Span>,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        walk_stmt_capability_evidence(stmt, raw_body_site, &block_scope, observer);
        block_scope.bind_stmt_locals(stmt);
    }
}

fn walk_stmt_capability_evidence(
    stmt: &Stmt,
    raw_body_site: Option<Span>,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            observer.observe_named_function_start(def.name.name.as_str(), def.name.span, &fn_scope);
            walk_fn_body_capability_evidence(&def.body, def.name.span, &fn_scope, observer);
            observer.observe_named_function_end(def.name.name.as_str(), def.name.span, &fn_scope);
        }
        Stmt::Impl(def) => {
            for method in &def.methods {
                let method_scope = scope.with_params(&method.params);
                observer.observe_named_function_start(
                    method.name.name.as_str(),
                    method.name.span,
                    &method_scope,
                );
                walk_fn_body_capability_evidence(
                    &method.body,
                    method.name.span,
                    &method_scope,
                    observer,
                );
                observer.observe_named_function_end(
                    method.name.name.as_str(),
                    method.name.span,
                    &method_scope,
                );
            }
        }
        Stmt::FnAlias(alias) => {
            observer.observe_fn_alias_target(alias.target.name.as_str(), alias.target.span, scope);
        }
        Stmt::StructDef(def) => observer.observe_struct_definition(def),
        Stmt::Wasm(body) => observer.observe_raw_body(
            HirBody::Wasm(body.clone()),
            raw_body_site.unwrap_or(body.span),
        ),
        Stmt::LlvmIr(body) => observer.observe_raw_body(
            HirBody::LlvmIr(body.clone()),
            raw_body_site.unwrap_or(body.span),
        ),
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            walk_expr_capability_evidence(expr, scope, observer);
        }
        Stmt::Directive(_) | Stmt::EnumDef(_) | Stmt::Trait(_) => {}
    }
}

fn walk_fn_body_capability_evidence(
    body: &FnBody,
    function_span: Span,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match body {
        FnBody::Parsed(block) => {
            walk_block_capability_evidence(block, Some(function_span), scope, observer)
        }
        FnBody::Wasm(body) => observer.observe_raw_body(HirBody::Wasm(body.clone()), function_span),
        FnBody::LlvmIr(body) => {
            observer.observe_raw_body(HirBody::LlvmIr(body.clone()), function_span)
        }
    }
}

fn walk_expr_capability_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    let mut call_head = PrefixCallHead::new();
    for (index, item) in expr.items.iter().enumerate() {
        observe_call_head_item(
            item,
            index,
            &expr.items,
            call_head.current_item_can_start_call() || expr.items.get(index + 1).is_some(),
            scope,
            observer,
        );
        observe_explicit_constructor_item(index, item, &expr.items, scope, observer);
        walk_prefix_item_capability_evidence(item, scope, observer);
        call_head.observe_item(item);
    }
}

fn observe_call_head_item(
    item: &PrefixItem,
    index: usize,
    items: &[PrefixItem],
    can_start_call: bool,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    if can_start_call {
        if let Some(symbol) = call_head_symbol(item) {
            observer.observe_call_head_symbol(
                symbol,
                prefix_item_span(item),
                field_selector_after_call_head(index, items),
                scope,
            );
        }
    }
}

fn observe_explicit_constructor_item(
    index: usize,
    item: &PrefixItem,
    items: &[PrefixItem],
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    if let Some(symbol) = explicit_constructor_symbol(item, items.get(index + 1).is_some()) {
        observer.observe_explicit_constructor_symbol(symbol, prefix_item_span(item), scope);
    }
}

fn walk_prefix_item_capability_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    observer: &mut impl SourceCapabilityObserver,
) {
    match item {
        PrefixItem::Intrinsic(intrinsic, span) => {
            observer.observe_intrinsic(intrinsic.name.as_str(), &intrinsic.args, *span, scope);
            for expr in &intrinsic.args {
                walk_expr_capability_evidence(expr, scope, observer);
            }
        }
        PrefixItem::Block(block, _) => walk_block_capability_evidence(block, None, scope, observer),
        PrefixItem::Match(m, _) => {
            walk_expr_capability_evidence(&m.scrutinee, scope, observer);
            for arm in &m.arms {
                let mut arm_scope = scope.clone();
                arm_scope.bind_match_pattern(&arm.pattern);
                walk_block_capability_evidence(&arm.body, None, &arm_scope, observer);
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

fn prefix_item_span(item: &PrefixItem) -> Span {
    match item {
        PrefixItem::Symbol(Symbol::Ident(id, _, _)) => id.span,
        PrefixItem::Symbol(Symbol::Let { name, .. }) => name.span,
        PrefixItem::Symbol(Symbol::Set { name }) => name.span,
        PrefixItem::Symbol(Symbol::If(span)) => *span,
        PrefixItem::Symbol(Symbol::While(span)) => *span,
        PrefixItem::Symbol(Symbol::AddrOf { span, .. }) => *span,
        PrefixItem::Symbol(Symbol::Deref(span)) => *span,
        PrefixItem::Literal(_, span)
        | PrefixItem::TypeAnnotation(_, span)
        | PrefixItem::Block(_, span)
        | PrefixItem::Match(_, span)
        | PrefixItem::Pipe(span)
        | PrefixItem::Tuple(_, span)
        | PrefixItem::Group(_, span)
        | PrefixItem::Intrinsic(_, span) => *span,
    }
}

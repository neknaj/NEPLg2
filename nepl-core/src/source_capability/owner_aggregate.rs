use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::scope::SourceCapabilityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerAggregateCapabilityEvidence {
    AggregateConstructor,
    FieldAccessor,
}

impl OwnerAggregateCapabilityEvidence {
    fn from_symbol(name: &str) -> Option<Self> {
        let base = helper_base_name(name);
        match base {
            "get" | "get_ref" | "put" | "get_field" | "get_field_ref" => Some(Self::FieldAccessor),
            _ if constructor_like_symbol(name, base) => Some(Self::AggregateConstructor),
            _ => None,
        }
    }
}

pub(crate) fn module_has_owner_aggregate_constructor_evidence(module: &Module) -> bool {
    module_has_owner_aggregate_evidence(
        module,
        OwnerAggregateCapabilityEvidence::AggregateConstructor,
    )
}

pub(crate) fn module_has_owner_aggregate_field_evidence(module: &Module) -> bool {
    module_has_owner_aggregate_evidence(module, OwnerAggregateCapabilityEvidence::FieldAccessor)
}

fn module_has_owner_aggregate_evidence(
    module: &Module,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    let scope = SourceCapabilityScope::from_module(module);
    block_owner_aggregate_evidence(&module.root, &scope, expected)
}

fn block_owner_aggregate_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        if stmt_owner_aggregate_evidence(stmt, &block_scope, expected) {
            return true;
        }
        block_scope.bind_stmt_locals(stmt);
    }
    false
}

fn stmt_owner_aggregate_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            fn_body_owner_aggregate_evidence(&def.body, &fn_scope, expected)
        }
        Stmt::Impl(def) => def.methods.iter().any(|method| {
            let method_scope = scope.with_params(&method.params);
            fn_body_owner_aggregate_evidence(&method.body, &method_scope, expected)
        }),
        Stmt::FnAlias(alias) => {
            !scope.shadows(alias.target.name.as_str())
                && OwnerAggregateCapabilityEvidence::from_symbol(alias.target.name.as_str())
                    == Some(expected)
        }
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            expr_owner_aggregate_evidence(expr, scope, expected)
        }
        Stmt::Directive(_)
        | Stmt::StructDef(_)
        | Stmt::EnumDef(_)
        | Stmt::Trait(_)
        | Stmt::Wasm(_)
        | Stmt::LlvmIr(_) => false,
    }
}

fn fn_body_owner_aggregate_evidence(
    body: &FnBody,
    scope: &SourceCapabilityScope,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    match body {
        FnBody::Parsed(block) => block_owner_aggregate_evidence(block, scope, expected),
        FnBody::Wasm(_) | FnBody::LlvmIr(_) => false,
    }
}

fn expr_owner_aggregate_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    expr.items
        .iter()
        .any(|item| prefix_item_owner_aggregate_evidence(item, scope, expected))
}

fn prefix_item_owner_aggregate_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
    expected: OwnerAggregateCapabilityEvidence,
) -> bool {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) if !scope.shadows(&ident.name) => {
            OwnerAggregateCapabilityEvidence::from_symbol(ident.name.as_str()) == Some(expected)
        }
        PrefixItem::Intrinsic(intrinsic, _) => intrinsic
            .args
            .iter()
            .any(|expr| expr_owner_aggregate_evidence(expr, scope, expected)),
        PrefixItem::Block(block, _) => block_owner_aggregate_evidence(block, scope, expected),
        PrefixItem::Match(m, _) => {
            expr_owner_aggregate_evidence(&m.scrutinee, scope, expected)
                || m.arms.iter().any(|arm| {
                    let mut arm_scope = scope.clone();
                    arm_scope.bind_match_pattern(&arm.pattern);
                    block_owner_aggregate_evidence(&arm.body, &arm_scope, expected)
                })
        }
        PrefixItem::Tuple(items, _) => items
            .iter()
            .any(|expr| expr_owner_aggregate_evidence(expr, scope, expected)),
        PrefixItem::Group(inner, _) => expr_owner_aggregate_evidence(inner, scope, expected),
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => false,
    }
}

fn constructor_like_symbol(symbol: &str, base: &str) -> bool {
    if crate::qualified_name::member_tail(symbol) != symbol {
        return false;
    }
    base.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
}

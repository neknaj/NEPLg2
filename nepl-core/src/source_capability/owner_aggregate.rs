use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::scope::SourceCapabilityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerAggregateBoundaryEvidence {
    AggregateConstructor,
    FieldAccessor,
}

impl OwnerAggregateBoundaryEvidence {
    fn from_symbol(name: &str) -> Option<Self> {
        let base = helper_base_name(name);
        match base {
            "get" | "get_ref" | "put" | "get_field" | "get_field_ref" => Some(Self::FieldAccessor),
            _ if constructor_like_symbol(base) => Some(Self::AggregateConstructor),
            _ => None,
        }
    }
}

pub(crate) fn module_has_owner_aggregate_boundary_evidence(module: &Module) -> bool {
    let scope = SourceCapabilityScope::from_module(module);
    block_owner_aggregate_boundary_evidence(&module.root, &scope).is_some()
}

fn block_owner_aggregate_boundary_evidence(
    block: &Block,
    scope: &SourceCapabilityScope,
) -> Option<OwnerAggregateBoundaryEvidence> {
    let mut block_scope = scope.clone();
    for stmt in &block.items {
        if let Some(evidence) = stmt_owner_aggregate_boundary_evidence(stmt, &block_scope) {
            return Some(evidence);
        }
        block_scope.bind_stmt_locals(stmt);
    }
    None
}

fn stmt_owner_aggregate_boundary_evidence(
    stmt: &Stmt,
    scope: &SourceCapabilityScope,
) -> Option<OwnerAggregateBoundaryEvidence> {
    match stmt {
        Stmt::FnDef(def) => {
            let fn_scope = scope.with_params(&def.params);
            fn_body_owner_aggregate_boundary_evidence(&def.body, &fn_scope)
        }
        Stmt::Impl(def) => def.methods.iter().find_map(|method| {
            let method_scope = scope.with_params(&method.params);
            fn_body_owner_aggregate_boundary_evidence(&method.body, &method_scope)
        }),
        Stmt::FnAlias(alias) => (!scope.shadows(alias.target.name.as_str()))
            .then(|| OwnerAggregateBoundaryEvidence::from_symbol(alias.target.name.as_str()))
            .flatten(),
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            expr_owner_aggregate_boundary_evidence(expr, scope)
        }
        Stmt::Directive(_)
        | Stmt::StructDef(_)
        | Stmt::EnumDef(_)
        | Stmt::Trait(_)
        | Stmt::Wasm(_)
        | Stmt::LlvmIr(_) => None,
    }
}

fn fn_body_owner_aggregate_boundary_evidence(
    body: &FnBody,
    scope: &SourceCapabilityScope,
) -> Option<OwnerAggregateBoundaryEvidence> {
    match body {
        FnBody::Parsed(block) => block_owner_aggregate_boundary_evidence(block, scope),
        FnBody::Wasm(_) | FnBody::LlvmIr(_) => None,
    }
}

fn expr_owner_aggregate_boundary_evidence(
    expr: &PrefixExpr,
    scope: &SourceCapabilityScope,
) -> Option<OwnerAggregateBoundaryEvidence> {
    expr.items
        .iter()
        .find_map(|item| prefix_item_owner_aggregate_boundary_evidence(item, scope))
}

fn prefix_item_owner_aggregate_boundary_evidence(
    item: &PrefixItem,
    scope: &SourceCapabilityScope,
) -> Option<OwnerAggregateBoundaryEvidence> {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) if !scope.shadows(&ident.name) => {
            OwnerAggregateBoundaryEvidence::from_symbol(ident.name.as_str())
        }
        PrefixItem::Intrinsic(intrinsic, _) => intrinsic
            .args
            .iter()
            .find_map(|expr| expr_owner_aggregate_boundary_evidence(expr, scope)),
        PrefixItem::Block(block, _) => block_owner_aggregate_boundary_evidence(block, scope),
        PrefixItem::Match(m, _) => expr_owner_aggregate_boundary_evidence(&m.scrutinee, scope)
            .or_else(|| {
                m.arms.iter().find_map(|arm| {
                    let mut arm_scope = scope.clone();
                    arm_scope.bind_match_pattern(&arm.pattern);
                    block_owner_aggregate_boundary_evidence(&arm.body, &arm_scope)
                })
            }),
        PrefixItem::Tuple(items, _) => items
            .iter()
            .find_map(|expr| expr_owner_aggregate_boundary_evidence(expr, scope)),
        PrefixItem::Group(inner, _) => expr_owner_aggregate_boundary_evidence(inner, scope),
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => None,
    }
}

fn constructor_like_symbol(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
}

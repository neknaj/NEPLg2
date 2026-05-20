use crate::ast::Stmt;
use crate::span::Span;

pub(super) fn span_key(span: Span) -> (u32, u32, u32) {
    (span.file_id.0, span.start, span.end)
}

pub(super) fn top_level_definition_span(item: &Stmt) -> Option<Span> {
    let span = match item {
        Stmt::FnDef(def) => def.name.span,
        Stmt::FnAlias(alias) => alias.name.span,
        Stmt::StructDef(def) => def.name.span,
        Stmt::EnumDef(def) => def.name.span,
        Stmt::Trait(def) => def.span,
        Stmt::Impl(def) => def.span,
        Stmt::Wasm(block) => block.span,
        Stmt::LlvmIr(block) => block.span,
        Stmt::Directive(_) | Stmt::Expr(_) | Stmt::ExprSemi(_, _) => return None,
    };
    (span != Span::dummy()).then_some(span)
}

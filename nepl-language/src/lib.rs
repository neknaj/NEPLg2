//! エディタ拡張向けの共通解析ライブラリ。
//!
//! `nepl-web` の Web 向け API とは別に、Zed / VSCode / 将来の LSP 実装が
//! 共有できる Rust ネイティブな解析結果を提供します。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use nepl_core::ast::{
    Block, Directive, FnBody, MatchArm, MatchPattern, Module, PrefixExpr, PrefixItem, Stmt, Symbol,
};
use nepl_core::compiler::BuildProfile;
use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::diagnostic_codes::{DiagnosticCode, LoaderDiagnosticCode};
use nepl_core::hir::{HirBlock, HirExpr, HirExprKind, HirLine, HirModule};
use nepl_core::lexer::{lex, Token, TokenKind};
use nepl_core::loader::{LoadResult, Loader, LoaderError, SourceMap};
use nepl_core::nm::Document as NmDocument;
use nepl_core::parser::parse_tokens;
use nepl_core::span::{FileId, Span};
use nepl_core::typecheck::typecheck;
use nepl_core::types::TypeCtx;
use nepl_core::CompileTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPosition {
    pub byte: u32,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRange {
    pub file_id: u32,
    pub path: Option<PathBuf>,
    pub start: TextPosition,
    pub end: TextPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenInfo {
    pub kind: String,
    pub value: Option<String>,
    pub debug: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexAnalysis {
    pub ok: bool,
    pub indent_width: usize,
    pub tokens: Vec<TokenInfo>,
    pub diagnostics: Vec<EditorDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameResolutionOptions {
    pub warn_important_shadow: bool,
}

impl Default for NameResolutionOptions {
    fn default() -> Self {
        Self {
            warn_important_shadow: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameDefinitionInfo {
    pub id: usize,
    pub name: String,
    pub kind: &'static str,
    pub range: TextRange,
    pub scope_depth: usize,
    pub doc: Option<String>,
    pub doc_ast: Option<NmDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameReferenceInfo {
    pub name: String,
    pub range: TextRange,
    pub scope_depth: usize,
    pub resolved_def_id: Option<usize>,
    pub candidate_def_ids: Vec<usize>,
    pub resolved_definition: Option<NameDefinitionInfo>,
    pub candidate_definitions: Vec<NameDefinitionInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameShadowInfo {
    pub name: String,
    pub event_kind: &'static str,
    pub range: TextRange,
    pub scope_depth: usize,
    pub selected_def_id: Option<usize>,
    pub shadowed_def_ids: Vec<usize>,
    pub severity: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameIndexEntry {
    pub definitions: Vec<usize>,
    pub references: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameResolutionPolicy {
    pub selection: &'static str,
    pub hoist: &'static str,
    pub warn_important_shadow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameResolutionAnalysis {
    pub ok: bool,
    pub diagnostics: Vec<EditorDiagnostic>,
    pub definitions: Vec<NameDefinitionInfo>,
    pub references: Vec<NameReferenceInfo>,
    pub shadows: Vec<NameShadowInfo>,
    pub shadow_diagnostics: Vec<NameShadowInfo>,
    pub by_name: BTreeMap<String, NameIndexEntry>,
    pub policy: NameResolutionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticExpressionInfo {
    pub id: usize,
    pub function_name: String,
    pub kind: &'static str,
    pub range: TextRange,
    pub inferred_type: String,
    pub parent_id: Option<usize>,
    pub argument_ranges: Vec<TextRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticTokenInfo {
    pub token_index: usize,
    pub inferred_expr_id: Option<usize>,
    pub inferred_type: Option<String>,
    pub expression_range: Option<TextRange>,
    pub arg_index: Option<usize>,
    pub arg_range: Option<TextRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticFunctionInfo {
    pub name: String,
    pub range: TextRange,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenHintInfo {
    pub token_index: usize,
    pub inferred_expr_id: Option<usize>,
    pub inferred_type: Option<String>,
    pub expression_range: Option<TextRange>,
    pub arg_index: Option<usize>,
    pub arg_range: Option<TextRange>,
    pub name: Option<String>,
    pub ref_range: Option<TextRange>,
    pub resolved_def_id: Option<usize>,
    pub candidate_def_ids: Vec<usize>,
    pub resolved_definition: Option<NameDefinitionInfo>,
    pub candidate_definitions: Vec<NameDefinitionInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticsAnalysis {
    pub ok: bool,
    pub tokens: Vec<TokenInfo>,
    pub diagnostics: Vec<EditorDiagnostic>,
    pub expressions: Vec<SemanticExpressionInfo>,
    pub token_semantics: Vec<SemanticTokenInfo>,
    pub token_hints: Vec<TokenHintInfo>,
    pub functions: Vec<SemanticFunctionInfo>,
    pub name_resolution: Option<NameResolutionAnalysis>,
    pub token_resolution: Vec<NameReferenceInfo>,
}

#[derive(Clone)]
struct NameDefTrace {
    id: usize,
    name: String,
    kind: &'static str,
    span: Span,
    scope_depth: usize,
    doc: Option<String>,
}

#[derive(Clone)]
struct NameRefTrace {
    name: String,
    span: Span,
    scope_depth: usize,
    resolved_def_id: Option<usize>,
    candidate_def_ids: Vec<usize>,
}

#[derive(Clone)]
struct NameShadowTrace {
    name: String,
    event_kind: &'static str,
    span: Span,
    scope_depth: usize,
    selected_def_id: Option<usize>,
    shadowed_def_ids: Vec<usize>,
    severity: &'static str,
    message: String,
}

#[derive(Clone)]
struct SemanticExprTrace {
    id: usize,
    function_name: String,
    kind: &'static str,
    span: Span,
    ty: String,
    parent_id: Option<usize>,
    arg_spans: Vec<Span>,
}

#[derive(Clone)]
struct SemanticTokenTrace {
    token_index: usize,
    inferred_expr_id: Option<usize>,
    inferred_type: Option<String>,
    expr_span: Option<Span>,
    arg_index: Option<usize>,
    arg_span: Option<Span>,
}

#[derive(Default)]
struct NameResolutionTrace {
    defs: Vec<NameDefTrace>,
    refs: Vec<NameRefTrace>,
    shadows: Vec<NameShadowTrace>,
    scopes: Vec<BTreeMap<String, Vec<usize>>>,
    warn_important_shadow: bool,
}

impl NameResolutionTrace {
    fn new_with_options(warn_important_shadow: bool) -> Self {
        Self {
            defs: Vec::new(),
            refs: Vec::new(),
            shadows: Vec::new(),
            scopes: vec![BTreeMap::new()],
            warn_important_shadow,
        }
    }

    fn current_depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define(
        &mut self,
        name: String,
        kind: &'static str,
        span: Span,
        doc: Option<String>,
    ) -> usize {
        let existing_candidates = self.lookup_candidates(&name);
        let id = self.defs.len();
        let depth = self.current_depth();
        self.defs.push(NameDefTrace {
            id,
            name: name.clone(),
            kind,
            span,
            scope_depth: depth,
            doc,
        });

        if !existing_candidates.is_empty() {
            let severity = if self.warn_important_shadow
                && is_important_shadow_symbol(&name)
                && is_variable_def_kind(kind)
            {
                "warning"
            } else {
                "info"
            };
            let message = if severity == "warning" {
                format!(
                    "important symbol '{}' is shadowed by {} definition",
                    name, kind
                )
            } else {
                format!("'{}' shadows an outer definition", name)
            };
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "definition_shadow",
                span,
                scope_depth: depth,
                selected_def_id: Some(id),
                shadowed_def_ids: existing_candidates,
                severity,
                message,
            });
        } else if self.warn_important_shadow
            && is_important_shadow_symbol(&name)
            && is_variable_def_kind(kind)
        {
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "important_name",
                span,
                scope_depth: depth,
                selected_def_id: Some(id),
                shadowed_def_ids: Vec::new(),
                severity: "warning",
                message: format!("definition '{}' may shadow important stdlib symbol", name),
            });
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.entry(name).or_default().push(id);
        }
        id
    }

    fn lookup_candidates(&self, name: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for scope in self.scopes.iter().rev() {
            if let Some(ids) = scope.get(name) {
                out.extend(ids.iter().rev().copied());
            }
        }
        out
    }

    fn reference(&mut self, name: String, span: Span) {
        let candidates = self.lookup_candidates(&name);
        let resolved = candidates.first().copied();
        if candidates.len() > 1 {
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "reference_shadow",
                span,
                scope_depth: self.current_depth(),
                selected_def_id: resolved,
                shadowed_def_ids: candidates[1..].to_vec(),
                severity: "info",
                message: format!(
                    "'{}' resolved to nearest definition with {} shadowed candidate(s)",
                    name,
                    candidates.len().saturating_sub(1)
                ),
            });
        }
        self.refs.push(NameRefTrace {
            name,
            span,
            scope_depth: self.current_depth(),
            resolved_def_id: resolved,
            candidate_def_ids: candidates,
        });
    }
}

pub fn load_module_from_fs(
    stdlib_root: impl Into<PathBuf>,
    entry_path: impl Into<PathBuf>,
) -> Result<LoadResult, LoaderError> {
    let mut loader = Loader::new(stdlib_root.into());
    loader.load(&entry_path.into())
}

pub fn load_inline_module(
    stdlib_root: impl Into<PathBuf>,
    entry_path: impl Into<PathBuf>,
    source: impl Into<String>,
) -> Result<LoadResult, LoaderError> {
    let mut loader = Loader::new(stdlib_root.into());
    loader.load_inline(entry_path.into(), source.into())
}

pub fn load_inline_module_with_provider(
    stdlib_root: impl Into<PathBuf>,
    entry_path: impl Into<PathBuf>,
    source: impl Into<String>,
    provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
) -> Result<LoadResult, LoaderError> {
    let mut loader = Loader::new(stdlib_root.into());
    loader.load_inline_with_provider(entry_path.into(), source.into(), provider)
}

pub fn analyze_lex(source: &str) -> LexAnalysis {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let diagnostics = diagnostics_to_editor(source, None, &lex_result.diagnostics);
    let ok = !lex_result
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    LexAnalysis {
        ok,
        indent_width: lex_result.indent_width,
        tokens: tokens_to_editor(source, None, &lex_result.tokens),
        diagnostics,
    }
}

pub fn analyze_name_resolution(
    source: &str,
    options: NameResolutionOptions,
) -> NameResolutionAnalysis {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let parse_result = parse_tokens(file_id, lex_result);
    match parse_result.module {
        Some(module) => {
            let mut trace = NameResolutionTrace::new_with_options(options.warn_important_shadow);
            trace_block(&mut trace, &module.root);
            name_resolution_to_editor(
                source,
                None,
                !parse_result
                    .diagnostics
                    .iter()
                    .any(|d| matches!(d.severity, Severity::Error)),
                &parse_result.diagnostics,
                &trace,
            )
        }
        None => empty_name_resolution(
            source,
            None,
            false,
            &parse_result.diagnostics,
            options.warn_important_shadow,
        ),
    }
}

pub fn analyze_loaded_name_resolution(
    source: &str,
    loaded: &LoadResult,
    options: NameResolutionOptions,
) -> NameResolutionAnalysis {
    let mut trace = NameResolutionTrace::new_with_options(options.warn_important_shadow);
    trace_block(&mut trace, &loaded.module.root);
    name_resolution_to_editor(source, Some(&loaded.source_map), true, &[], &trace)
}

pub fn analyze_semantics(source: &str) -> SemanticsAnalysis {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let tokens = lex_result.tokens.clone();
    let token_infos = tokens_to_editor(source, None, &tokens);
    let parse_result = parse_tokens(file_id, lex_result);
    analyze_semantics_from_parts(
        source,
        None,
        tokens,
        token_infos,
        parse_result.module,
        parse_result.diagnostics,
    )
}

pub fn analyze_loaded_semantics(source: &str, loaded: &LoadResult) -> SemanticsAnalysis {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let tokens = lex_result.tokens.clone();
    let token_infos = tokens_to_editor(source, Some(&loaded.source_map), &tokens);
    analyze_semantics_from_loaded(
        source,
        Some(&loaded.source_map),
        tokens,
        token_infos,
        &loaded.module,
        lex_result.diagnostics,
    )
}

fn analyze_semantics_from_parts(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: Vec<Token>,
    token_infos: Vec<TokenInfo>,
    module: Option<Module>,
    diagnostics: Vec<Diagnostic>,
) -> SemanticsAnalysis {
    let mut all_diags = diagnostics;
    let mut has_error = all_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));

    let Some(module) = module else {
        return SemanticsAnalysis {
            ok: false,
            tokens: token_infos,
            diagnostics: diagnostics_to_editor(source, source_map, &all_diags),
            expressions: Vec::new(),
            token_semantics: Vec::new(),
            token_hints: Vec::new(),
            functions: Vec::new(),
            name_resolution: None,
            token_resolution: Vec::new(),
        };
    };

    let mut resolve_trace = NameResolutionTrace::new_with_options(true);
    trace_block(&mut resolve_trace, &module.root);
    let name_resolution = name_resolution_to_editor(source, source_map, true, &[], &resolve_trace);

    let (target, mut target_diags) = resolve_target_for_analysis(&module);
    has_error |= target_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    all_diags.append(&mut target_diags);

    let tc = typecheck(&module, target, BuildProfile::Debug, source_map);
    has_error |= tc
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    all_diags.extend(tc.diagnostics.clone());

    build_semantics_output(
        source,
        source_map,
        token_infos,
        &tokens,
        &resolve_trace,
        name_resolution,
        tc.module.as_ref(),
        &tc.types,
        diagnostics_to_editor(source, source_map, &all_diags),
        !has_error,
    )
}

fn analyze_semantics_from_loaded(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: Vec<Token>,
    token_infos: Vec<TokenInfo>,
    module: &Module,
    diagnostics: Vec<Diagnostic>,
) -> SemanticsAnalysis {
    let mut all_diags = diagnostics;
    let mut has_error = all_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));

    let mut resolve_trace = NameResolutionTrace::new_with_options(true);
    trace_block(&mut resolve_trace, &module.root);
    let name_resolution = name_resolution_to_editor(source, source_map, true, &[], &resolve_trace);

    let (target, mut target_diags) = resolve_target_for_analysis(module);
    has_error |= target_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    all_diags.append(&mut target_diags);

    let tc = typecheck(module, target, BuildProfile::Debug, source_map);
    has_error |= tc
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    all_diags.extend(tc.diagnostics.clone());

    build_semantics_output(
        source,
        source_map,
        token_infos,
        &tokens,
        &resolve_trace,
        name_resolution,
        tc.module.as_ref(),
        &tc.types,
        diagnostics_to_editor(source, source_map, &all_diags),
        !has_error,
    )
}

fn build_semantics_output(
    source: &str,
    source_map: Option<&SourceMap>,
    token_infos: Vec<TokenInfo>,
    tokens: &[Token],
    resolve_trace: &NameResolutionTrace,
    name_resolution: NameResolutionAnalysis,
    hir_module: Option<&HirModule>,
    types: &TypeCtx,
    diagnostics: Vec<EditorDiagnostic>,
    ok: bool,
) -> SemanticsAnalysis {
    let Some(hir_module) = hir_module else {
        return SemanticsAnalysis {
            ok: false,
            tokens: token_infos,
            diagnostics,
            expressions: Vec::new(),
            token_semantics: Vec::new(),
            token_hints: Vec::new(),
            functions: Vec::new(),
            name_resolution: Some(name_resolution),
            token_resolution: Vec::new(),
        };
    };

    let mut exprs = Vec::<SemanticExprTrace>::new();
    let mut functions = Vec::new();
    for function in &hir_module.functions {
        functions.push(SemanticFunctionInfo {
            name: function.name.clone(),
            range: range_from_span(source, source_map, function.span),
            signature: types.type_to_string(function.func_ty),
        });
        if let nepl_core::hir::HirBody::Block(body) = &function.body {
            for line in &body.lines {
                collect_semantic_expr_from_line(line, &function.name, types, &mut exprs);
            }
        }
    }

    let expressions = exprs
        .iter()
        .map(|expr| SemanticExpressionInfo {
            id: expr.id,
            function_name: expr.function_name.clone(),
            kind: expr.kind,
            range: range_from_span(source, source_map, expr.span),
            inferred_type: expr.ty.clone(),
            parent_id: expr.parent_id,
            argument_ranges: expr
                .arg_spans
                .iter()
                .map(|span| range_from_span(source, source_map, *span))
                .collect(),
        })
        .collect::<Vec<_>>();

    let token_resolution = build_token_resolution(source, source_map, tokens, resolve_trace);
    let token_semantics_trace = build_token_semantics(tokens, &exprs);
    let token_semantics = token_semantics_trace
        .iter()
        .map(|item| semantic_token_to_editor(source, source_map, item))
        .collect::<Vec<_>>();
    let token_hints = build_token_hints(
        source,
        source_map,
        tokens,
        &token_semantics_trace,
        resolve_trace,
    );

    SemanticsAnalysis {
        ok,
        tokens: token_infos,
        diagnostics,
        expressions,
        token_semantics,
        token_hints,
        functions,
        name_resolution: Some(name_resolution),
        token_resolution,
    }
}

fn build_token_resolution(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: &[Token],
    resolve_trace: &NameResolutionTrace,
) -> Vec<NameReferenceInfo> {
    let mut out = Vec::new();
    for token in tokens {
        if let Some(reference) = best_ref_for_token(resolve_trace, token.span) {
            out.push(ref_trace_to_editor(
                source,
                source_map,
                reference,
                &resolve_trace.defs,
            ));
        }
    }
    out
}

fn build_token_semantics(tokens: &[Token], exprs: &[SemanticExprTrace]) -> Vec<SemanticTokenTrace> {
    let mut out = Vec::with_capacity(tokens.len());
    for (token_index, token) in tokens.iter().enumerate() {
        let mut best_expr: Option<&SemanticExprTrace> = None;
        for expr in exprs {
            if span_contains(expr.span, token.span) {
                if let Some(prev) = best_expr {
                    if span_width(expr.span) < span_width(prev.span) {
                        best_expr = Some(expr);
                    }
                } else {
                    best_expr = Some(expr);
                }
            }
        }

        let mut arg_hit: Option<(usize, Span)> = None;
        for expr in exprs {
            for (arg_index, arg_span) in expr.arg_spans.iter().enumerate() {
                if span_contains(*arg_span, token.span) {
                    if let Some((_, prev_span)) = arg_hit {
                        if span_width(*arg_span) < span_width(prev_span) {
                            arg_hit = Some((arg_index, *arg_span));
                        }
                    } else {
                        arg_hit = Some((arg_index, *arg_span));
                    }
                }
            }
        }

        out.push(SemanticTokenTrace {
            token_index,
            inferred_expr_id: best_expr.map(|expr| expr.id),
            inferred_type: best_expr.map(|expr| expr.ty.clone()),
            expr_span: best_expr.map(|expr| expr.span),
            arg_index: arg_hit.map(|(index, _)| index),
            arg_span: arg_hit.map(|(_, span)| span),
        });
    }
    out
}

fn build_token_hints(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: &[Token],
    token_semantics: &[SemanticTokenTrace],
    resolve_trace: &NameResolutionTrace,
) -> Vec<TokenHintInfo> {
    let mut out = Vec::with_capacity(tokens.len());
    for (token_index, token) in tokens.iter().enumerate() {
        let semantics = token_semantics.get(token_index);
        let reference = best_ref_for_token(resolve_trace, token.span);

        let (
            resolved_definition,
            candidate_definitions,
            candidate_def_ids,
            resolved_def_id,
            name,
            ref_range,
        ) = if let Some(reference) = reference {
            let resolved_definition = reference
                .resolved_def_id
                .and_then(|id| resolve_trace.defs.get(id))
                .map(|def| def_trace_to_editor(source, source_map, def));
            let candidate_definitions = reference
                .candidate_def_ids
                .iter()
                .filter_map(|id| resolve_trace.defs.get(*id))
                .map(|def| def_trace_to_editor(source, source_map, def))
                .collect::<Vec<_>>();
            (
                resolved_definition,
                candidate_definitions,
                reference.candidate_def_ids.clone(),
                reference.resolved_def_id,
                Some(reference.name.clone()),
                Some(range_from_span(source, source_map, reference.span)),
            )
        } else {
            (None, Vec::new(), Vec::new(), None, None, None)
        };

        out.push(TokenHintInfo {
            token_index,
            inferred_expr_id: semantics.and_then(|item| item.inferred_expr_id),
            inferred_type: semantics.and_then(|item| item.inferred_type.clone()),
            expression_range: semantics
                .and_then(|item| item.expr_span)
                .map(|span| range_from_span(source, source_map, span)),
            arg_index: semantics.and_then(|item| item.arg_index),
            arg_range: semantics
                .and_then(|item| item.arg_span)
                .map(|span| range_from_span(source, source_map, span)),
            name,
            ref_range,
            resolved_def_id,
            candidate_def_ids,
            resolved_definition,
            candidate_definitions,
        });
    }
    out
}

fn tokens_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: &[Token],
) -> Vec<TokenInfo> {
    tokens
        .iter()
        .map(|token| TokenInfo {
            kind: token_kind_name(&token.kind).to_string(),
            value: token_value(&token.kind),
            debug: format!("{:?}", token.kind),
            range: range_from_span(source, source_map, token.span),
        })
        .collect()
}

fn diagnostics_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    diagnostics: &[Diagnostic],
) -> Vec<EditorDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| EditorDiagnostic {
            severity: diagnostic.severity,
            code: diagnostic.code.as_str(),
            message: diagnostic.message.clone(),
            notes: diagnostic.notes.clone(),
            helps: diagnostic.helps.clone(),
            range: range_from_span(source, source_map, diagnostic.primary.span),
        })
        .collect()
}

fn empty_name_resolution(
    source: &str,
    source_map: Option<&SourceMap>,
    ok: bool,
    diagnostics: &[Diagnostic],
    warn_important_shadow: bool,
) -> NameResolutionAnalysis {
    NameResolutionAnalysis {
        ok,
        diagnostics: diagnostics_to_editor(source, source_map, diagnostics),
        definitions: Vec::new(),
        references: Vec::new(),
        shadows: Vec::new(),
        shadow_diagnostics: Vec::new(),
        by_name: BTreeMap::new(),
        policy: NameResolutionPolicy {
            selection: "nearest_scope_first",
            hoist: "fn and non-mut let",
            warn_important_shadow,
        },
    }
}

fn name_resolution_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    ok: bool,
    diagnostics: &[Diagnostic],
    trace: &NameResolutionTrace,
) -> NameResolutionAnalysis {
    let definitions = trace
        .defs
        .iter()
        .map(|definition| def_trace_to_editor(source, source_map, definition))
        .collect::<Vec<_>>();
    let references = trace
        .refs
        .iter()
        .map(|reference| ref_trace_to_editor(source, source_map, reference, &trace.defs))
        .collect::<Vec<_>>();
    let shadows = trace
        .shadows
        .iter()
        .map(|shadow| shadow_trace_to_editor(source, source_map, shadow))
        .collect::<Vec<_>>();
    let shadow_diagnostics = trace
        .shadows
        .iter()
        .filter(|shadow| matches!(shadow.severity, "warning" | "info"))
        .map(|shadow| shadow_trace_to_editor(source, source_map, shadow))
        .collect::<Vec<_>>();

    let mut by_name = BTreeMap::<String, NameIndexEntry>::new();
    for definition in &trace.defs {
        by_name
            .entry(definition.name.clone())
            .or_insert_with(|| NameIndexEntry {
                definitions: Vec::new(),
                references: Vec::new(),
            })
            .definitions
            .push(definition.id);
    }
    for (reference_index, reference) in trace.refs.iter().enumerate() {
        by_name
            .entry(reference.name.clone())
            .or_insert_with(|| NameIndexEntry {
                definitions: Vec::new(),
                references: Vec::new(),
            })
            .references
            .push(reference_index);
    }

    NameResolutionAnalysis {
        ok,
        diagnostics: diagnostics_to_editor(source, source_map, diagnostics),
        definitions,
        references,
        shadows,
        shadow_diagnostics,
        by_name,
        policy: NameResolutionPolicy {
            selection: "nearest_scope_first",
            hoist: "fn and non-mut let",
            warn_important_shadow: trace.warn_important_shadow,
        },
    }
}

fn def_trace_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    definition: &NameDefTrace,
) -> NameDefinitionInfo {
    NameDefinitionInfo {
        id: definition.id,
        name: definition.name.clone(),
        kind: definition.kind,
        range: range_from_span(source, source_map, definition.span),
        scope_depth: definition.scope_depth,
        doc: definition.doc.clone(),
        doc_ast: definition
            .doc
            .as_ref()
            .map(|doc| nepl_core::nm::parse_document(doc)),
    }
}

fn ref_trace_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    reference: &NameRefTrace,
    definitions: &[NameDefTrace],
) -> NameReferenceInfo {
    NameReferenceInfo {
        name: reference.name.clone(),
        range: range_from_span(source, source_map, reference.span),
        scope_depth: reference.scope_depth,
        resolved_def_id: reference.resolved_def_id,
        candidate_def_ids: reference.candidate_def_ids.clone(),
        resolved_definition: reference
            .resolved_def_id
            .and_then(|id| definitions.get(id))
            .map(|definition| def_trace_to_editor(source, source_map, definition)),
        candidate_definitions: reference
            .candidate_def_ids
            .iter()
            .filter_map(|id| definitions.get(*id))
            .map(|definition| def_trace_to_editor(source, source_map, definition))
            .collect(),
    }
}

fn shadow_trace_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    shadow: &NameShadowTrace,
) -> NameShadowInfo {
    NameShadowInfo {
        name: shadow.name.clone(),
        event_kind: shadow.event_kind,
        range: range_from_span(source, source_map, shadow.span),
        scope_depth: shadow.scope_depth,
        selected_def_id: shadow.selected_def_id,
        shadowed_def_ids: shadow.shadowed_def_ids.clone(),
        severity: shadow.severity,
        message: shadow.message.clone(),
    }
}

fn semantic_token_to_editor(
    source: &str,
    source_map: Option<&SourceMap>,
    item: &SemanticTokenTrace,
) -> SemanticTokenInfo {
    SemanticTokenInfo {
        token_index: item.token_index,
        inferred_expr_id: item.inferred_expr_id,
        inferred_type: item.inferred_type.clone(),
        expression_range: item
            .expr_span
            .map(|span| range_from_span(source, source_map, span)),
        arg_index: item.arg_index,
        arg_range: item
            .arg_span
            .map(|span| range_from_span(source, source_map, span)),
    }
}

fn range_from_span(source: &str, source_map: Option<&SourceMap>, span: Span) -> TextRange {
    let file_id = span.file_id.0;
    let path = source_map.and_then(|map| {
        map.path(span.file_id)
            .map(|path| PathBuf::from(path.as_str()))
    });
    let text = source_map
        .and_then(|map| map.get(span.file_id))
        .unwrap_or(source);

    let start = line_col_in_text(text, span.start);
    let end = line_col_in_text(text, span.end);

    TextRange {
        file_id,
        path,
        start: TextPosition {
            byte: span.start,
            line: start.0,
            column: start.1,
        },
        end: TextPosition {
            byte: span.end,
            line: end.0,
            column: end.1,
        },
    }
}

fn line_col_in_text(source: &str, byte: u32) -> (usize, usize) {
    let mut line = 0usize;
    let mut column = 0usize;
    let mut offset = 0u32;
    for ch in source.bytes() {
        if offset == byte {
            return (line, column);
        }
        offset += 1;
        if ch == b'\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn token_kind_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Indent => "Indent",
        TokenKind::Dedent => "Dedent",
        TokenKind::Newline => "Newline",
        TokenKind::Eof => "Eof",
        TokenKind::Colon => "Colon",
        TokenKind::Semicolon => "Semicolon",
        TokenKind::Pipe => "Pipe",
        TokenKind::LParen => "LParen",
        TokenKind::RParen => "RParen",
        TokenKind::Comma => "Comma",
        TokenKind::LAngle => "LAngle",
        TokenKind::RAngle => "RAngle",
        TokenKind::Arrow(_) => "Arrow",
        TokenKind::PathSep => "PathSep",
        TokenKind::At => "At",
        TokenKind::Dot => "Dot",
        TokenKind::Ampersand => "Ampersand",
        TokenKind::Star => "Star",
        TokenKind::Minus => "Minus",
        TokenKind::Equals => "Equals",
        TokenKind::Ident(_) => "Ident",
        TokenKind::IntLiteral(_) => "IntLiteral",
        TokenKind::FloatLiteral(_) => "FloatLiteral",
        TokenKind::BoolLiteral(_) => "BoolLiteral",
        TokenKind::CharLiteral(_) => "CharLiteral",
        TokenKind::StringLiteral(_) => "StringLiteral",
        TokenKind::UnitLiteral => "UnitLiteral",
        TokenKind::KwFn => "KwFn",
        TokenKind::KwLet => "KwLet",
        TokenKind::KwMut => "KwMut",
        TokenKind::KwNoShadow => "KwNoShadow",
        TokenKind::KwSet => "KwSet",
        TokenKind::KwIf => "KwIf",
        TokenKind::KwWhile => "KwWhile",
        TokenKind::KwCond => "KwCond",
        TokenKind::KwThen => "KwThen",
        TokenKind::KwElse => "KwElse",
        TokenKind::KwDo => "KwDo",
        TokenKind::KwStruct => "KwStruct",
        TokenKind::KwEnum => "KwEnum",
        TokenKind::KwMatch => "KwMatch",
        TokenKind::KwTrait => "KwTrait",
        TokenKind::KwImpl => "KwImpl",
        TokenKind::KwFor => "KwFor",
        TokenKind::KwPub => "KwPub",
        TokenKind::KwBlock => "KwBlock",
        TokenKind::KwTuple => "KwTuple",
        TokenKind::KwMlstr => "KwMlstr",
        TokenKind::DirEntry(_) => "DirEntry",
        TokenKind::DirTarget(_) => "DirTarget",
        TokenKind::DirImport(_) => "DirImport",
        TokenKind::DirUse(_) => "DirUse",
        TokenKind::DirIfTarget(_) => "DirIfTarget",
        TokenKind::DirIfProfile(_) => "DirIfProfile",
        TokenKind::DirCapability(_) => "DirCapability",
        TokenKind::DirWasm => "DirWasm",
        TokenKind::DirLlvmIr => "DirLlvmIr",
        TokenKind::DirIndentWidth(_) => "DirIndentWidth",
        TokenKind::DirInclude(_) => "DirInclude",
        TokenKind::DirExtern { .. } => "DirExtern",
        TokenKind::DirIntrinsic => "DirIntrinsic",
        TokenKind::DirPrelude(_) => "DirPrelude",
        TokenKind::DirNoPrelude => "DirNoPrelude",
        TokenKind::WasmText(_) => "WasmText",
        TokenKind::LlvmIrText(_) => "LlvmIrText",
        TokenKind::MlstrLine(_) => "MlstrLine",
        TokenKind::DocComment(_) => "DocComment",
    }
}

fn token_value(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Ident(value)
        | TokenKind::IntLiteral(value)
        | TokenKind::FloatLiteral(value)
        | TokenKind::StringLiteral(value)
        | TokenKind::DirEntry(value)
        | TokenKind::DirTarget(value)
        | TokenKind::DirImport(value)
        | TokenKind::DirUse(value)
        | TokenKind::DirIfTarget(value)
        | TokenKind::DirIfProfile(value)
        | TokenKind::DirCapability(value)
        | TokenKind::DirInclude(value)
        | TokenKind::DirPrelude(value)
        | TokenKind::WasmText(value)
        | TokenKind::LlvmIrText(value)
        | TokenKind::MlstrLine(value)
        | TokenKind::DocComment(value) => Some(value.clone()),
        TokenKind::BoolLiteral(value) => Some(value.to_string()),
        TokenKind::CharLiteral(value) => Some(value.to_string()),
        TokenKind::Arrow(effect) => Some(format!("{:?}", effect)),
        TokenKind::DirIndentWidth(value) => Some(value.to_string()),
        TokenKind::DirExtern {
            module,
            name,
            func,
            signature,
        } => Some(format!(
            "module={module}, name={name}, func={func}, signature={signature}"
        )),
        _ => None,
    }
}

fn is_layout_marker(name: &str) -> bool {
    matches!(name, "cond" | "then" | "else" | "do" | "block")
}

fn is_important_shadow_symbol(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "print_i32"
            | "println_i32"
            | "add"
            | "sub"
            | "mul"
            | "div"
            | "eq"
            | "lt"
            | "le"
            | "gt"
            | "ge"
            | "map"
            | "len"
    )
}

fn is_variable_def_kind(kind: &str) -> bool {
    matches!(kind, "let_hoisted" | "let_mut" | "param" | "match_bind")
}

fn hoist_block_defs(trace: &mut NameResolutionTrace, block: &Block) {
    for stmt in &block.items {
        match stmt {
            Stmt::FnDef(definition) => {
                trace.define(
                    definition.name.name.clone(),
                    "fn",
                    definition.name.span,
                    definition.doc.clone(),
                );
            }
            Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                if let Some(PrefixItem::Symbol(Symbol::Let { name, mutable, .. })) =
                    expr.items.first()
                {
                    if !*mutable {
                        trace.define(name.name.clone(), "let_hoisted", name.span, None);
                    }
                }
            }
            Stmt::StructDef(definition) => {
                trace.define(
                    definition.name.name.clone(),
                    "struct",
                    definition.name.span,
                    definition.doc.clone(),
                );
            }
            Stmt::EnumDef(definition) => {
                trace.define(
                    definition.name.name.clone(),
                    "enum",
                    definition.name.span,
                    definition.doc.clone(),
                );
            }
            Stmt::Trait(definition) => {
                trace.define(
                    definition.name.name.clone(),
                    "trait",
                    definition.name.span,
                    definition.doc.clone(),
                );
            }
            _ => {}
        }
    }
}

fn trace_match_arm(trace: &mut NameResolutionTrace, arm: &MatchArm) {
    trace.push_scope();
    if let MatchPattern::Variant {
        bind: Some(bind), ..
    } = &arm.pattern
    {
        trace.define(bind.name.clone(), "match_bind", bind.span, None);
    }
    trace_block(trace, &arm.body);
    trace.pop_scope();
}

fn trace_prefix_expr(trace: &mut NameResolutionTrace, expr: &PrefixExpr) {
    for (index, item) in expr.items.iter().enumerate() {
        match item {
            PrefixItem::Symbol(Symbol::Let { name, mutable, .. }) => {
                if *mutable {
                    trace.define(name.name.clone(), "let_mut", name.span, None);
                }
                if index != 0 {
                    trace.reference(name.name.clone(), name.span);
                }
            }
            PrefixItem::Symbol(Symbol::Set { name }) => {
                trace.reference(name.name.clone(), name.span);
            }
            PrefixItem::Symbol(Symbol::Ident(identifier, _, _)) => {
                if !is_layout_marker(&identifier.name) {
                    trace.reference(identifier.name.clone(), identifier.span);
                }
            }
            PrefixItem::Block(block, _) => {
                trace.push_scope();
                trace_block(trace, block);
                trace.pop_scope();
            }
            PrefixItem::Match(match_expr, _) => {
                trace_prefix_expr(trace, &match_expr.scrutinee);
                for arm in &match_expr.arms {
                    trace_match_arm(trace, arm);
                }
            }
            PrefixItem::Tuple(items, _) => {
                for item in items {
                    trace_prefix_expr(trace, item);
                }
            }
            PrefixItem::Group(inner, _) => {
                trace_prefix_expr(trace, inner);
            }
            PrefixItem::Intrinsic(intrinsic, _) => {
                for arg in &intrinsic.args {
                    trace_prefix_expr(trace, arg);
                }
            }
            PrefixItem::Literal(_, _)
            | PrefixItem::TypeAnnotation(_, _)
            | PrefixItem::Pipe(_)
            | PrefixItem::Symbol(Symbol::If(_))
            | PrefixItem::Symbol(Symbol::While(_))
            | PrefixItem::Symbol(Symbol::AddrOf { .. })
            | PrefixItem::Symbol(Symbol::Deref(_)) => {}
        }
    }
}

fn trace_stmt(trace: &mut NameResolutionTrace, stmt: &Stmt) {
    match stmt {
        Stmt::FnDef(definition) => match &definition.body {
            FnBody::Parsed(body) => {
                trace.push_scope();
                for param in &definition.params {
                    trace.define(param.name.clone(), "param", param.span, None);
                }
                trace_block(trace, body);
                trace.pop_scope();
            }
            FnBody::Wasm(_) | FnBody::LlvmIr(_) => {}
        },
        Stmt::FnAlias(alias) => {
            trace.reference(alias.target.name.clone(), alias.target.span);
            trace.define(
                alias.name.name.clone(),
                "fn_alias",
                alias.name.span,
                alias.doc.clone(),
            );
        }
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            trace_prefix_expr(trace, expr);
        }
        _ => {}
    }
}

fn trace_block(trace: &mut NameResolutionTrace, block: &Block) {
    hoist_block_defs(trace, block);
    for stmt in &block.items {
        trace_stmt(trace, stmt);
    }
}

fn best_ref_for_token<'a>(
    trace: &'a NameResolutionTrace,
    token_span: Span,
) -> Option<&'a NameRefTrace> {
    let mut best: Option<&NameRefTrace> = None;
    for reference in &trace.refs {
        if span_contains(reference.span, token_span) {
            if let Some(prev) = best {
                if span_width(reference.span) < span_width(prev.span) {
                    best = Some(reference);
                }
            } else {
                best = Some(reference);
            }
        }
    }
    best
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.file_id == inner.file_id && outer.start <= inner.start && inner.end <= outer.end
}

fn span_width(span: Span) -> usize {
    span.end.saturating_sub(span.start) as usize
}

fn hir_kind_name(kind: &HirExprKind) -> &'static str {
    match kind {
        HirExprKind::LiteralI32(_) => "LiteralI32",
        HirExprKind::LiteralF32(_) => "LiteralF32",
        HirExprKind::LiteralBool(_) => "LiteralBool",
        HirExprKind::LiteralStr(_) => "LiteralStr",
        HirExprKind::Unit => "Unit",
        HirExprKind::Var(_) => "Var",
        HirExprKind::FnValue(_) => "FnValue",
        HirExprKind::Call { .. } => "Call",
        HirExprKind::CallIndirect { .. } => "CallIndirect",
        HirExprKind::If { .. } => "If",
        HirExprKind::While { .. } => "While",
        HirExprKind::Match { .. } => "Match",
        HirExprKind::EnumConstruct { .. } => "EnumConstruct",
        HirExprKind::StructConstruct { .. } => "StructConstruct",
        HirExprKind::TupleConstruct { .. } => "TupleConstruct",
        HirExprKind::Block(_) => "Block",
        HirExprKind::Let { .. } => "Let",
        HirExprKind::Set { .. } => "Set",
        HirExprKind::Intrinsic { .. } => "Intrinsic",
        HirExprKind::AddrOf(_) => "AddrOf",
        HirExprKind::Deref(_) => "Deref",
        HirExprKind::Drop { .. } => "Drop",
    }
}

fn collect_semantic_expr_from_line(
    line: &HirLine,
    function_name: &str,
    types: &TypeCtx,
    out: &mut Vec<SemanticExprTrace>,
) {
    collect_semantic_expr(&line.expr, function_name, types, None, out);
}

fn collect_semantic_expr_from_block(
    block: &HirBlock,
    function_name: &str,
    types: &TypeCtx,
    parent_id: Option<usize>,
    out: &mut Vec<SemanticExprTrace>,
) {
    for line in &block.lines {
        collect_semantic_expr(&line.expr, function_name, types, parent_id, out);
    }
}

fn collect_semantic_expr(
    expr: &HirExpr,
    function_name: &str,
    types: &TypeCtx,
    parent_id: Option<usize>,
    out: &mut Vec<SemanticExprTrace>,
) -> usize {
    let id = out.len();
    out.push(SemanticExprTrace {
        id,
        function_name: function_name.to_string(),
        kind: hir_kind_name(&expr.kind),
        span: expr.span,
        ty: types.type_to_string(expr.ty),
        parent_id,
        arg_spans: Vec::new(),
    });

    let mut arg_spans = Vec::new();
    match &expr.kind {
        HirExprKind::Call { args, .. } => {
            for arg in args {
                arg_spans.push(arg.span);
                collect_semantic_expr(arg, function_name, types, Some(id), out);
            }
        }
        HirExprKind::FnValue(_) => {}
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_semantic_expr(callee, function_name, types, Some(id), out);
            for arg in args {
                arg_spans.push(arg.span);
                collect_semantic_expr(arg, function_name, types, Some(id), out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            arg_spans.push(cond.span);
            arg_spans.push(then_branch.span);
            arg_spans.push(else_branch.span);
            collect_semantic_expr(cond, function_name, types, Some(id), out);
            collect_semantic_expr(then_branch, function_name, types, Some(id), out);
            collect_semantic_expr(else_branch, function_name, types, Some(id), out);
        }
        HirExprKind::While { cond, body } => {
            arg_spans.push(cond.span);
            arg_spans.push(body.span);
            collect_semantic_expr(cond, function_name, types, Some(id), out);
            collect_semantic_expr(body, function_name, types, Some(id), out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            arg_spans.push(scrutinee.span);
            collect_semantic_expr(scrutinee, function_name, types, Some(id), out);
            for arm in arms {
                arg_spans.push(arm.body.span);
                collect_semantic_expr(&arm.body, function_name, types, Some(id), out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                arg_spans.push(payload.span);
                collect_semantic_expr(payload, function_name, types, Some(id), out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                arg_spans.push(field.span);
                collect_semantic_expr(field, function_name, types, Some(id), out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                arg_spans.push(item.span);
                collect_semantic_expr(item, function_name, types, Some(id), out);
            }
        }
        HirExprKind::Block(block) => {
            collect_semantic_expr_from_block(block, function_name, types, Some(id), out);
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            arg_spans.push(value.span);
            collect_semantic_expr(value, function_name, types, Some(id), out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                arg_spans.push(arg.span);
                collect_semantic_expr(arg, function_name, types, Some(id), out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            arg_spans.push(inner.span);
            collect_semantic_expr(inner, function_name, types, Some(id), out);
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::Drop { .. } => {}
    }

    out[id].arg_spans = arg_spans;
    id
}

fn resolve_target_for_analysis(module: &Module) -> (CompileTarget, Vec<Diagnostic>) {
    let mut found: Option<(CompileTarget, Span)> = None;
    let mut diagnostics = Vec::new();

    for directive in &module.directives {
        if let Directive::Target { target, span } = directive {
            let parsed = parse_target_name(target);
            if let Some(target) = parsed {
                if let Some((_, prev_span)) = found {
                    diagnostics.push(
                        loader_error(
                            LoaderDiagnosticCode::TargetMultipleDirective,
                            "multiple #target directives are not allowed",
                            *span,
                        )
                        .with_secondary_label(prev_span, Some("previous #target here".into())),
                    );
                } else {
                    found = Some((target, *span));
                }
            } else {
                diagnostics.push(loader_error(
                    LoaderDiagnosticCode::TargetUnknown,
                    "unknown target in #target",
                    *span,
                ));
            }
        }
    }

    if found.is_none() {
        for item in &module.root.items {
            if let Stmt::Directive(Directive::Target { target, span }) = item {
                let parsed = parse_target_name(target);
                if let Some(target) = parsed {
                    if let Some((_, prev_span)) = found {
                        diagnostics.push(
                            loader_error(
                                LoaderDiagnosticCode::TargetMultipleDirective,
                                "multiple #target directives are not allowed",
                                *span,
                            )
                            .with_secondary_label(prev_span, Some("previous #target here".into())),
                        );
                    } else {
                        found = Some((target, *span));
                    }
                } else {
                    diagnostics.push(loader_error(
                        LoaderDiagnosticCode::TargetUnknown,
                        "unknown target in #target",
                        *span,
                    ));
                }
            }
        }
    }

    (
        found
            .map(|(target, _)| target)
            .unwrap_or(CompileTarget::Wasm),
        diagnostics,
    )
}

fn loader_error(code: LoaderDiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Loader(code), message, span)
}

fn parse_target_name(target: &str) -> Option<CompileTarget> {
    match target {
        "wasm" | "core" => Some(CompileTarget::Wasm),
        "wasi" | "std" => Some(CompileTarget::Wasi),
        "llvm" => Some(CompileTarget::Llvm),
        _ => None,
    }
}

pub fn default_stdlib_root(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root.as_ref().join("stdlib")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_analysis_reports_doc_comment_token() {
        let source = "//: adds numbers\nfn add <(i32,i32)->i32> (a,b):\n    add a b;\n";
        let analysis = analyze_lex(source);
        assert!(analysis.ok);
        assert!(analysis
            .tokens
            .iter()
            .any(|token| token.kind == "DocComment"));
    }

    #[test]
    fn semantics_analysis_reports_hover_doc_and_type() {
        let source = r#"#no_prelude
//: adds one
fn inc <(i32)->i32> (x):
    x

fn main <()->i32> ():
    inc 41
"#;
        let analysis = analyze_semantics(source);
        assert!(analysis.ok, "{:?}", analysis.diagnostics);
        let resolved = analysis
            .token_hints
            .iter()
            .find(|hint| hint.name.as_deref() == Some("inc") && hint.resolved_definition.is_some())
            .and_then(|hint| hint.resolved_definition.clone())
            .expect("resolved definition for inc");
        assert_eq!(resolved.doc.as_deref(), Some("adds one"));
        assert!(resolved.doc_ast.is_some());
        assert!(analysis
            .token_semantics
            .iter()
            .any(|item| item.inferred_type.as_deref() == Some("i32")));
    }

    #[test]
    fn loaded_name_resolution_keeps_cross_file_path() {
        let entry_path = PathBuf::from("/workspace/main.nepl");
        let source = r#"#no_prelude
#import "dep" as *

fn main <()->i32> ():
    dep_value
"#;
        let mut provider = |path: &PathBuf| -> Result<String, LoaderError> {
            if path.ends_with("dep.nepl") {
                Ok("#no_prelude\n//: from dep\nfn dep_value <()->i32> ():\n    7\n".to_string())
            } else {
                Err(LoaderError::Io(format!(
                    "missing source: {}",
                    path.display()
                )))
            }
        };
        let loaded = load_inline_module_with_provider("/stdlib", entry_path, source, &mut provider)
            .expect("load module with provider");
        let analysis =
            analyze_loaded_name_resolution(source, &loaded, NameResolutionOptions::default());
        let reference = analysis
            .references
            .iter()
            .find(|reference| {
                reference.name == "dep_value" && reference.resolved_definition.is_some()
            })
            .expect("resolved dep_value");
        let resolved = reference
            .resolved_definition
            .clone()
            .expect("resolved definition");
        assert_eq!(resolved.doc.as_deref(), Some("from dep"));
        assert!(resolved.range.path.is_some());
    }

    #[test]
    fn target_directive_diagnostics_keep_loader_codes() {
        let unknown =
            analyze_semantics("#target nope\n#no_prelude\nfn main <()->i32> ():\n    0\n");
        assert!(
            unknown
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "loader.target.unknown"),
            "{:?}",
            unknown.diagnostics
        );

        let duplicate = analyze_semantics(
            "#target wasm\n#target wasi\n#no_prelude\nfn main <()->i32> ():\n    0\n",
        );
        assert!(
            duplicate
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "loader.target.multiple_directive" }),
            "{:?}",
            duplicate.diagnostics
        );
    }
}

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::span::FileId;
use crate::span::Span;

/// Effect of a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Pure,
    Impure,
}

/// Surface-level type expression (before inference).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Unit,
    I32,
    U8,
    F32,
    Bool,
    Char,
    Never,
    Str,
    Label(Option<String>), // . or .label
    Named(String),
    Apply(Box<TypeExpr>, Vec<TypeExpr>),
    Boxed(Box<TypeExpr>),
    Reference(Box<TypeExpr>, bool), // (inner, is_mut)
    Tuple(Vec<TypeExpr>),
    Function {
        params: Vec<TypeExpr>,
        result: Box<TypeExpr>,
        effect: Effect,
    },
    Spanned(Box<TypeExpr>, Span),
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Spanned(_, span) => *span,
            TypeExpr::Apply(base, args) => args
                .last()
                .and_then(|last| base.span().join(last.span()))
                .unwrap_or_else(|| base.span()),
            TypeExpr::Boxed(inner) | TypeExpr::Reference(inner, _) => inner.span(),
            TypeExpr::Tuple(items) => items
                .first()
                .zip(items.last())
                .and_then(|(first, last)| first.span().join(last.span()))
                .unwrap_or_else(Span::dummy),
            TypeExpr::Function { params, result, .. } => params
                .first()
                .map(|first| first.span())
                .unwrap_or_else(|| result.span())
                .join(result.span())
                .unwrap_or_else(|| result.span()),
            _ => Span::dummy(),
        }
    }

    pub fn as_unspanned(&self) -> &TypeExpr {
        let mut current = self;
        loop {
            match current {
                TypeExpr::Spanned(inner, _) => current = inner.as_ref(),
                other => return other,
            }
        }
    }

    pub fn into_unspanned(self) -> TypeExpr {
        match self {
            TypeExpr::Spanned(inner, _) => inner.into_unspanned(),
            other => other,
        }
    }

    pub fn with_span(self, span: Span) -> TypeExpr {
        TypeExpr::Spanned(Box::new(self), span)
    }
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    Float(String),
    Bool(bool),
    Char(u32),
    Str(String),
    Unit,
}

/// Identifier with span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

/// A prefix expression line.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrefixExpr {
    pub items: Vec<PrefixItem>,
    pub trailing_semis: u32,
    pub trailing_semi_span: Option<Span>,
    pub span: Span,
}

/// Items that compose a prefix expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixItem {
    Symbol(Symbol),
    Literal(Literal, Span),
    TypeAnnotation(TypeExpr, Span),
    Block(Block, Span),
    Match(MatchExpr, Span),
    Pipe(Span),
    Tuple(Vec<PrefixExpr>, Span),
    Group(PrefixExpr, Span),
    Intrinsic(IntrinsicExpr, Span),
}

/// Special symbols in the language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// `forced_value=true` when parsed from `@ident`.
    Ident(Ident, Vec<TypeExpr>, bool),
    Let {
        name: Ident,
        mutable: bool,
        no_shadow: bool,
    },
    Set {
        name: Ident,
    },
    If(Span),
    While(Span),
    AddrOf {
        span: Span,
        mutable: bool,
    },
    Deref(Span),
}

/// A block of statements (introduced by `:` or the file root).
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub items: Vec<Stmt>,
    pub span: Span,
}

/// Function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub doc: Option<String>,
    pub vis: Visibility,
    pub name: Ident,
    pub no_shadow: bool,
    pub type_params: Vec<TypeParam>,
    pub signature: TypeExpr,
    pub params: Vec<Ident>,
    pub body: FnBody,
}

/// Function alias definition.
#[derive(Debug, Clone, PartialEq)]
pub struct FnAlias {
    pub doc: Option<String>,
    pub vis: Visibility,
    pub name: Ident,
    pub no_shadow: bool,
    pub target: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: Ident,
    pub bounds: Vec<TraitRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    pub name: Ident,
    pub args: Vec<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub doc: Option<String>,
    pub vis: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub capabilities: Vec<TraitCapability>,
    pub methods: Vec<FnDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitCapability {
    Copy,
    Clone,
    Drop,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDef {
    pub doc: Option<String>,
    pub type_params: Vec<TypeParam>,
    pub trait_ref: Option<TraitRef>, // None for inherent impl
    pub target_ty: TypeExpr,
    pub methods: Vec<FnDef>,
    pub span: Span,
}

/// Function body kind.
#[derive(Debug, Clone, PartialEq)]
pub enum FnBody {
    Parsed(Block),
    Wasm(WasmBlock),
    LlvmIr(LlvmIrBlock),
}

/// Wasm text block collected from `#wasm:` lines.
#[derive(Debug, Clone, PartialEq)]
pub struct WasmBlock {
    pub lines: Vec<String>,
    pub span: Span,
}

/// LLVM IR text block collected from `#llvmir:` lines.
#[derive(Debug, Clone, PartialEq)]
pub struct LlvmIrBlock {
    pub lines: Vec<String>,
    pub span: Span,
}

/// Top-level directives.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Entry {
        name: Ident,
    },
    Target {
        target: String,
        span: Span,
    },
    /// Module import with visibility and clause.
    Import {
        path: String,
        clause: ImportClause,
        vis: Visibility,
        span: Span,
    },
    Use {
        path: String,
        span: Span,
    },
    IfTarget {
        target: String,
        span: Span,
    },
    IfProfile {
        profile: String,
        span: Span,
    },
    Test {
        span: Span,
    },
    IndentWidth {
        width: usize,
        span: Span,
    },
    Extern {
        vis: Visibility,
        module: String,
        name: String,
        func: Ident,
        signature: TypeExpr,
        span: Span,
    },
    Include {
        path: String,
        span: Span,
    },
    Prelude {
        path: String,
        span: Span,
    },
    NoPrelude {
        span: Span,
    },
}

/// A single statement inside a block.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Directive(Directive),
    FnDef(FnDef),
    FnAlias(FnAlias),
    StructDef(StructDef),
    EnumDef(EnumDef),
    Wasm(WasmBlock),
    LlvmIr(LlvmIrBlock),
    Trait(TraitDef),
    Impl(ImplDef),
    Expr(PrefixExpr),
    ExprSemi(PrefixExpr, Option<Span>),
}

/// Parsed module.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub doc: Option<String>,
    pub indent_width: usize,
    pub directives: Vec<Directive>,
    pub root: Block,
}

/// Visibility for items/imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Pub,
    Private,
}

/// Import clause detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportClause {
    /// No clause: default alias = last path segment.
    DefaultAlias,
    /// `as name`
    Alias(String),
    /// `as *`
    Open,
    /// `as { ... }`
    Selective(Vec<ImportItem>),
    /// `as @merge`
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub glob: bool,
}

/// Struct definition (simple positional fields).
#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub doc: Option<String>,
    pub vis: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<(Ident, TypeExpr)>,
}

/// Enum definition with optional single payload per variant.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Ident,
    pub payload: Option<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub doc: Option<String>,
    pub vis: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
}

/// Match expression arms.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Variant { name: Ident, bind: Option<Ident> },
    IntLiteral { text: String, span: Span },
    BoolLiteral { value: bool, span: Span },
    CharLiteral { value: u32, span: Span },
    Wildcard { span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Block,
    pub span: Span,
}

/// Match expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub scrutinee: PrefixExpr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Intrinsic expression: `#intrinsic "name" <Args...> (Exprs...)`
#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicExpr {
    pub name: String,
    pub name_span: Span,
    pub type_args: Vec<TypeExpr>,
    pub args: Vec<PrefixExpr>,
    pub span: Span,
}

/// Rewrite the source-file identity of every precise span inside an AST module.
///
/// Parsed-module caches cannot keep the original `FileId`, because each load
/// creates a fresh `SourceMap` and assigns file ids by append order.  The byte
/// ranges remain valid while the source text is unchanged, so the loader can
/// cache a parsed module in a neutral file namespace and project it back onto
/// the `FileId` allocated for the current compile.  `Span::dummy()` is kept
/// intact because it represents "no precise source location" rather than a
/// real range in file 0.
pub(crate) fn remap_module_file_id(module: &mut Module, from: FileId, to: FileId) {
    for directive in &mut module.directives {
        remap_directive_file_id(directive, from, to);
    }
    remap_block_file_id(&mut module.root, from, to);
}

fn remap_span_file_id(span: &mut Span, from: FileId, to: FileId) {
    if span.file_id == from && *span != Span::dummy() {
        span.file_id = to;
    }
}

fn remap_ident_file_id(ident: &mut Ident, from: FileId, to: FileId) {
    remap_span_file_id(&mut ident.span, from, to);
}

fn remap_type_expr_file_id(ty: &mut TypeExpr, from: FileId, to: FileId) {
    match ty {
        TypeExpr::Apply(base, args) => {
            remap_type_expr_file_id(base, from, to);
            for arg in args {
                remap_type_expr_file_id(arg, from, to);
            }
        }
        TypeExpr::Boxed(inner) | TypeExpr::Reference(inner, _) => {
            remap_type_expr_file_id(inner, from, to);
        }
        TypeExpr::Tuple(items) => {
            for item in items {
                remap_type_expr_file_id(item, from, to);
            }
        }
        TypeExpr::Function { params, result, .. } => {
            for param in params {
                remap_type_expr_file_id(param, from, to);
            }
            remap_type_expr_file_id(result, from, to);
        }
        TypeExpr::Spanned(inner, span) => {
            remap_type_expr_file_id(inner, from, to);
            remap_span_file_id(span, from, to);
        }
        TypeExpr::Unit
        | TypeExpr::I32
        | TypeExpr::U8
        | TypeExpr::F32
        | TypeExpr::Bool
        | TypeExpr::Char
        | TypeExpr::Never
        | TypeExpr::Str
        | TypeExpr::Label(_)
        | TypeExpr::Named(_) => {}
    }
}

fn remap_prefix_expr_file_id(expr: &mut PrefixExpr, from: FileId, to: FileId) {
    for item in &mut expr.items {
        remap_prefix_item_file_id(item, from, to);
    }
    if let Some(span) = &mut expr.trailing_semi_span {
        remap_span_file_id(span, from, to);
    }
    remap_span_file_id(&mut expr.span, from, to);
}

fn remap_prefix_item_file_id(item: &mut PrefixItem, from: FileId, to: FileId) {
    match item {
        PrefixItem::Symbol(symbol) => remap_symbol_file_id(symbol, from, to),
        PrefixItem::Literal(_, span)
        | PrefixItem::TypeAnnotation(_, span)
        | PrefixItem::Block(_, span)
        | PrefixItem::Match(_, span)
        | PrefixItem::Pipe(span)
        | PrefixItem::Tuple(_, span)
        | PrefixItem::Group(_, span)
        | PrefixItem::Intrinsic(_, span) => remap_span_file_id(span, from, to),
    }
    match item {
        PrefixItem::TypeAnnotation(ty, _) => remap_type_expr_file_id(ty, from, to),
        PrefixItem::Block(block, _) => remap_block_file_id(block, from, to),
        PrefixItem::Match(match_expr, _) => remap_match_expr_file_id(match_expr, from, to),
        PrefixItem::Tuple(items, _) => {
            for expr in items {
                remap_prefix_expr_file_id(expr, from, to);
            }
        }
        PrefixItem::Group(expr, _) => remap_prefix_expr_file_id(expr, from, to),
        PrefixItem::Intrinsic(intrinsic, _) => remap_intrinsic_expr_file_id(intrinsic, from, to),
        PrefixItem::Symbol(_) | PrefixItem::Literal(_, _) | PrefixItem::Pipe(_) => {}
    }
}

fn remap_symbol_file_id(symbol: &mut Symbol, from: FileId, to: FileId) {
    match symbol {
        Symbol::Ident(ident, type_args, _) => {
            remap_ident_file_id(ident, from, to);
            for ty in type_args {
                remap_type_expr_file_id(ty, from, to);
            }
        }
        Symbol::Let { name, .. } | Symbol::Set { name } => {
            remap_ident_file_id(name, from, to);
        }
        Symbol::If(span) | Symbol::While(span) | Symbol::Deref(span) => {
            remap_span_file_id(span, from, to);
        }
        Symbol::AddrOf { span, .. } => {
            remap_span_file_id(span, from, to);
        }
    }
}

fn remap_block_file_id(block: &mut Block, from: FileId, to: FileId) {
    for item in &mut block.items {
        remap_stmt_file_id(item, from, to);
    }
    remap_span_file_id(&mut block.span, from, to);
}

fn remap_fn_def_file_id(def: &mut FnDef, from: FileId, to: FileId) {
    remap_ident_file_id(&mut def.name, from, to);
    for param in &mut def.type_params {
        remap_type_param_file_id(param, from, to);
    }
    remap_type_expr_file_id(&mut def.signature, from, to);
    for param in &mut def.params {
        remap_ident_file_id(param, from, to);
    }
    remap_fn_body_file_id(&mut def.body, from, to);
}

fn remap_fn_alias_file_id(alias: &mut FnAlias, from: FileId, to: FileId) {
    remap_ident_file_id(&mut alias.name, from, to);
    remap_ident_file_id(&mut alias.target, from, to);
}

fn remap_type_param_file_id(param: &mut TypeParam, from: FileId, to: FileId) {
    remap_ident_file_id(&mut param.name, from, to);
    for bound in &mut param.bounds {
        remap_trait_ref_file_id(bound, from, to);
    }
}

fn remap_trait_ref_file_id(trait_ref: &mut TraitRef, from: FileId, to: FileId) {
    remap_ident_file_id(&mut trait_ref.name, from, to);
    for arg in &mut trait_ref.args {
        remap_type_expr_file_id(arg, from, to);
    }
}

fn remap_trait_def_file_id(def: &mut TraitDef, from: FileId, to: FileId) {
    remap_ident_file_id(&mut def.name, from, to);
    for param in &mut def.type_params {
        remap_type_param_file_id(param, from, to);
    }
    for method in &mut def.methods {
        remap_fn_def_file_id(method, from, to);
    }
    remap_span_file_id(&mut def.span, from, to);
}

fn remap_impl_def_file_id(def: &mut ImplDef, from: FileId, to: FileId) {
    for param in &mut def.type_params {
        remap_type_param_file_id(param, from, to);
    }
    if let Some(trait_ref) = &mut def.trait_ref {
        remap_trait_ref_file_id(trait_ref, from, to);
    }
    remap_type_expr_file_id(&mut def.target_ty, from, to);
    for method in &mut def.methods {
        remap_fn_def_file_id(method, from, to);
    }
    remap_span_file_id(&mut def.span, from, to);
}

fn remap_fn_body_file_id(body: &mut FnBody, from: FileId, to: FileId) {
    match body {
        FnBody::Parsed(block) => remap_block_file_id(block, from, to),
        FnBody::Wasm(block) => remap_wasm_block_file_id(block, from, to),
        FnBody::LlvmIr(block) => remap_llvm_ir_block_file_id(block, from, to),
    }
}

fn remap_wasm_block_file_id(block: &mut WasmBlock, from: FileId, to: FileId) {
    remap_span_file_id(&mut block.span, from, to);
}

fn remap_llvm_ir_block_file_id(block: &mut LlvmIrBlock, from: FileId, to: FileId) {
    remap_span_file_id(&mut block.span, from, to);
}

fn remap_directive_file_id(directive: &mut Directive, from: FileId, to: FileId) {
    match directive {
        Directive::Entry { name } => remap_ident_file_id(name, from, to),
        Directive::Target { span, .. }
        | Directive::Import { span, .. }
        | Directive::Use { span, .. }
        | Directive::IfTarget { span, .. }
        | Directive::IfProfile { span, .. }
        | Directive::Test { span }
        | Directive::IndentWidth { span, .. }
        | Directive::Include { span, .. }
        | Directive::Prelude { span, .. }
        | Directive::NoPrelude { span } => remap_span_file_id(span, from, to),
        Directive::Extern {
            func,
            signature,
            span,
            ..
        } => {
            remap_ident_file_id(func, from, to);
            remap_type_expr_file_id(signature, from, to);
            remap_span_file_id(span, from, to);
        }
    }
}

fn remap_stmt_file_id(stmt: &mut Stmt, from: FileId, to: FileId) {
    match stmt {
        Stmt::Directive(directive) => remap_directive_file_id(directive, from, to),
        Stmt::FnDef(def) => remap_fn_def_file_id(def, from, to),
        Stmt::FnAlias(alias) => remap_fn_alias_file_id(alias, from, to),
        Stmt::StructDef(def) => remap_struct_def_file_id(def, from, to),
        Stmt::EnumDef(def) => remap_enum_def_file_id(def, from, to),
        Stmt::Wasm(block) => remap_wasm_block_file_id(block, from, to),
        Stmt::LlvmIr(block) => remap_llvm_ir_block_file_id(block, from, to),
        Stmt::Trait(def) => remap_trait_def_file_id(def, from, to),
        Stmt::Impl(def) => remap_impl_def_file_id(def, from, to),
        Stmt::Expr(expr) => remap_prefix_expr_file_id(expr, from, to),
        Stmt::ExprSemi(expr, trailing_semi_span) => {
            remap_prefix_expr_file_id(expr, from, to);
            if let Some(span) = trailing_semi_span {
                remap_span_file_id(span, from, to);
            }
        }
    }
}

fn remap_struct_def_file_id(def: &mut StructDef, from: FileId, to: FileId) {
    remap_ident_file_id(&mut def.name, from, to);
    for param in &mut def.type_params {
        remap_type_param_file_id(param, from, to);
    }
    for (field, ty) in &mut def.fields {
        remap_ident_file_id(field, from, to);
        remap_type_expr_file_id(ty, from, to);
    }
}

fn remap_enum_def_file_id(def: &mut EnumDef, from: FileId, to: FileId) {
    remap_ident_file_id(&mut def.name, from, to);
    for param in &mut def.type_params {
        remap_type_param_file_id(param, from, to);
    }
    for variant in &mut def.variants {
        remap_ident_file_id(&mut variant.name, from, to);
        if let Some(payload) = &mut variant.payload {
            remap_type_expr_file_id(payload, from, to);
        }
    }
}

fn remap_match_pattern_file_id(pattern: &mut MatchPattern, from: FileId, to: FileId) {
    match pattern {
        MatchPattern::Variant { name, bind } => {
            remap_ident_file_id(name, from, to);
            if let Some(bind) = bind {
                remap_ident_file_id(bind, from, to);
            }
        }
        MatchPattern::IntLiteral { span, .. }
        | MatchPattern::BoolLiteral { span, .. }
        | MatchPattern::CharLiteral { span, .. }
        | MatchPattern::Wildcard { span } => remap_span_file_id(span, from, to),
    }
}

fn remap_match_arm_file_id(arm: &mut MatchArm, from: FileId, to: FileId) {
    remap_match_pattern_file_id(&mut arm.pattern, from, to);
    remap_block_file_id(&mut arm.body, from, to);
    remap_span_file_id(&mut arm.span, from, to);
}

fn remap_match_expr_file_id(expr: &mut MatchExpr, from: FileId, to: FileId) {
    remap_prefix_expr_file_id(&mut expr.scrutinee, from, to);
    for arm in &mut expr.arms {
        remap_match_arm_file_id(arm, from, to);
    }
    remap_span_file_id(&mut expr.span, from, to);
}

fn remap_intrinsic_expr_file_id(expr: &mut IntrinsicExpr, from: FileId, to: FileId) {
    remap_span_file_id(&mut expr.name_span, from, to);
    for ty in &mut expr.type_args {
        remap_type_expr_file_id(ty, from, to);
    }
    for arg in &mut expr.args {
        remap_prefix_expr_file_id(arg, from, to);
    }
    remap_span_file_id(&mut expr.span, from, to);
}

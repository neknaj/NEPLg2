extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

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
    pub type_args: Vec<TypeExpr>,
    pub args: Vec<PrefixExpr>,
    pub span: Span,
}

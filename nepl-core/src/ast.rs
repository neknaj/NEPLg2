#![no_std]
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
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Unit,
    I32,
    F32,
    Bool,
    Label(Option<String>), // . or .label
    Function {
        params: Vec<TypeExpr>,
        result: Box<TypeExpr>,
        effect: Effect,
    },
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(String),
    Float(String),
    Bool(bool),
    Unit,
}

/// Identifier with span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

/// A prefix expression line.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixExpr {
    pub items: Vec<PrefixItem>,
    pub span: Span,
}

/// Items that compose a prefix expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PrefixItem {
    Symbol(Symbol),
    Literal(Literal, Span),
    TypeAnnotation(TypeExpr, Span),
    Block(Block, Span),
    Semi(Span),
}

/// Special symbols in the language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Ident(Ident),
    Let { name: Ident, mutable: bool },
    Set { name: Ident },
    If(Span),
    While(Span),
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
    pub name: Ident,
    pub signature: TypeExpr,
    pub params: Vec<Ident>,
    pub body: FnBody,
}

/// Function body kind.
#[derive(Debug, Clone, PartialEq)]
pub enum FnBody {
    Parsed(Block),
    Wasm(WasmBlock),
}

/// Wasm text block collected from `#wasm:` lines.
#[derive(Debug, Clone, PartialEq)]
pub struct WasmBlock {
    pub lines: Vec<String>,
    pub span: Span,
}

/// Top-level directives.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Entry { name: Ident },
    Import { path: String, span: Span },
    Use { path: String, span: Span },
    IfTarget { target: String, span: Span },
    IndentWidth { width: usize, span: Span },
}

/// A single statement inside a block.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Directive(Directive),
    FnDef(FnDef),
    Wasm(WasmBlock),
    Expr(PrefixExpr),
}

/// Parsed module.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub indent_width: usize,
    pub directives: Vec<Directive>,
    pub root: Block,
}

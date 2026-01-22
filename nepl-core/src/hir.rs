#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, WasmBlock};
use crate::span::Span;
use crate::types::TypeId;

#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub entry: Option<String>,
    pub externs: Vec<HirExtern>,
    pub string_literals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub result: TypeId,
    pub effect: Effect,
    pub body: HirBody,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: TypeId,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct HirExtern {
    pub module: String,
    pub name: String,
    pub local_name: String,
    pub params: Vec<TypeId>,
    pub result: TypeId,
    pub effect: Effect,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirBody {
    Block(HirBlock),
    Wasm(WasmBlock),
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub lines: Vec<HirLine>,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirLine {
    pub expr: HirExpr,
    pub drop_result: bool,
}

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub ty: TypeId,
    pub kind: HirExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirExprKind {
    LiteralI32(i32),
    LiteralF32(f32),
    LiteralBool(bool),
    LiteralStr(u32),
    Unit,
    Var(String),
    Call {
        callee: FuncRef,
        args: Vec<HirExpr>,
    },
    If {
        cond: Box<HirExpr>,
        then_branch: Box<HirExpr>,
        else_branch: Box<HirExpr>,
    },
    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },
    Block(HirBlock),
    Let {
        name: String,
        mutable: bool,
        value: Box<HirExpr>,
    },
    Set {
        name: String,
        value: Box<HirExpr>,
    },
}

#[derive(Debug, Clone)]
pub enum FuncRef {
    Builtin(String),
    User(String),
}

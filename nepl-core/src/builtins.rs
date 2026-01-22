#![no_std]
extern crate alloc;

use alloc::vec;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    AddI32,
    SubI32,
    LtI32,
    PrintI32,
}

#[derive(Debug, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub ty: TypeId,
    pub effect: Effect,
    pub kind: BuiltinKind,
}

pub fn builtins(ctx: &mut TypeCtx) -> alloc::vec::Vec<Builtin> {
    let i32 = ctx.i32();
    let bool_ty = ctx.bool();
    let unit = ctx.unit();

    let add_ty = ctx.function(vec![i32, i32], i32, Effect::Pure);
    let sub_ty = ctx.function(vec![i32, i32], i32, Effect::Pure);
    let lt_ty = ctx.function(vec![i32, i32], bool_ty, Effect::Pure);
    let print_ty = ctx.function(vec![i32], unit, Effect::Impure);

    alloc::vec![
        Builtin {
            name: "add",
            ty: add_ty,
            effect: Effect::Pure,
            kind: BuiltinKind::AddI32,
        },
        Builtin {
            name: "sub",
            ty: sub_ty,
            effect: Effect::Pure,
            kind: BuiltinKind::SubI32,
        },
        Builtin {
            name: "lt",
            ty: lt_ty,
            effect: Effect::Pure,
            kind: BuiltinKind::LtI32,
        },
        Builtin {
            name: "print_i32",
            ty: print_ty,
            effect: Effect::Impure,
            kind: BuiltinKind::PrintI32,
        },
    ]
}

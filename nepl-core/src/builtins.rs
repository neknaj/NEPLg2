#![no_std]
extern crate alloc;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    MemAlloc,
    MemDealloc,
    MemRealloc,
    MemLoadI32,
    MemStoreI32,
}

#[derive(Debug, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub ty: TypeId,
    pub effect: Effect,
    pub kind: BuiltinKind,
}

pub fn builtins(ctx: &mut TypeCtx) -> alloc::vec::Vec<Builtin> {
    let mut v = alloc::vec::Vec::new();

    // alloc(size: i32) -> i32 (ptr)
    let alloc_ty = ctx.function(alloc::vec![ctx.i32()], ctx.i32(), Effect::Impure);
    v.push(Builtin {
        name: "alloc",
        ty: alloc_ty,
        effect: Effect::Impure,
        kind: BuiltinKind::MemAlloc,
    });

    // dealloc(ptr: i32, size: i32) -> ()
    let dealloc_ty = ctx.function(alloc::vec![ctx.i32(), ctx.i32()], ctx.unit(), Effect::Impure);
    v.push(Builtin {
        name: "dealloc",
        ty: dealloc_ty,
        effect: Effect::Impure,
        kind: BuiltinKind::MemDealloc,
    });

    // realloc(ptr: i32, old: i32, new: i32) -> i32
    let realloc_ty = ctx.function(alloc::vec![ctx.i32(), ctx.i32(), ctx.i32()], ctx.i32(), Effect::Impure);
    v.push(Builtin {
        name: "realloc",
        ty: realloc_ty,
        effect: Effect::Impure,
        kind: BuiltinKind::MemRealloc,
    });

    // Note: low-level load/store are implemented in stdlib with inline wasm ops.

    v
}

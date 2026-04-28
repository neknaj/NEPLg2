use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::TypeExpr;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) type LabelEnv = BTreeMap<String, TypeId>;

#[derive(Debug)]
pub(super) struct StringTable {
    map: BTreeMap<String, u32>,
    items: Vec<String>,
}

impl StringTable {
    pub(super) fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            items: Vec::new(),
        }
    }

    pub(super) fn intern(&mut self, s: String) -> u32 {
        if let Some(id) = self.map.get(&s) {
            *id
        } else {
            let id = self.items.len() as u32;
            self.items.push(s.clone());
            self.map.insert(s, id);
            id
        }
    }

    pub(super) fn get(&self, id: u32) -> Option<&String> {
        self.items.get(id as usize)
    }

    pub(super) fn into_vec(self) -> Vec<String> {
        self.items
    }
}

pub(super) fn type_from_expr(ctx: &mut TypeCtx, labels: &mut LabelEnv, t: &TypeExpr) -> TypeId {
    match t.as_unspanned() {
        TypeExpr::Unit => ctx.unit(),
        TypeExpr::I32 => ctx.i32(),
        TypeExpr::U8 => ctx.u8(),
        TypeExpr::F32 => ctx.f32(),
        TypeExpr::Bool => ctx.bool(),
        TypeExpr::Char => ctx.char(),
        TypeExpr::Str => ctx.str(),
        TypeExpr::Never => ctx.never(),
        TypeExpr::Named(name) => match name.as_str() {
            "i32" => ctx.i32(),
            "u8" => ctx.u8(),
            "f32" => ctx.f32(),
            "bool" => ctx.bool(),
            "char" => ctx.char(),
            "str" => ctx.str(),
            "never" => ctx.never(),
            _ => {
                if let Some(id) = labels.get(name) {
                    return *id;
                }
                if let Some(id) = ctx.lookup_named(name) {
                    id
                } else {
                    ctx.register_named(name.clone(), TypeKind::Named(name.clone()))
                }
            }
        },
        TypeExpr::Apply(base, args) => {
            let b = type_from_expr(ctx, labels, base);
            let mut arg_tys = Vec::new();
            for a in args {
                arg_tys.push(type_from_expr(ctx, labels, a));
            }
            ctx.apply(b, arg_tys)
        }
        TypeExpr::Label(label) => {
            if let Some(name) = label {
                if let Some(existing) = labels.get(name) {
                    *existing
                } else {
                    let id = ctx.fresh_var(Some(name.clone()));
                    labels.insert(name.clone(), id);
                    id
                }
            } else {
                ctx.fresh_var(None)
            }
        }
        TypeExpr::Function {
            params,
            result,
            effect,
        } => {
            let mut p = Vec::new();
            for ty in params {
                p.push(type_from_expr(ctx, labels, ty));
            }
            let r = type_from_expr(ctx, labels, result);
            ctx.function(Vec::new(), p, r, *effect)
        }
        TypeExpr::Tuple(items) => {
            let mut elems = Vec::new();
            for ty in items {
                elems.push(type_from_expr(ctx, labels, ty));
            }
            ctx.tuple(elems)
        }
        TypeExpr::Boxed(inner) => {
            let i = type_from_expr(ctx, labels, inner);
            ctx.box_ty(i)
        }
        TypeExpr::Reference(inner, is_mut) => {
            let i = type_from_expr(ctx, labels, inner);
            ctx.reference(i, *is_mut)
        }
        TypeExpr::Spanned(inner, _) => type_from_expr(ctx, labels, inner),
    }
}

#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

use crate::ast::Effect;

/// Identifier for a type stored in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Unit,
    I32,
    F32,
    Bool,
    Str,
    Never,
    Named(String),
    Enum {
        name: String,
        variants: Vec<EnumVariantInfo>,
    },
    Struct {
        name: String,
        fields: Vec<TypeId>,
    },
    Function {
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
    },
    Var(TypeVar),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeVar {
    pub label: Option<alloc::string::String>,
    pub binding: Option<TypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantInfo {
    pub name: alloc::string::String,
    pub payload: Option<TypeId>,
}

/// Arena-based type context with simple unification.
#[derive(Debug)]
pub struct TypeCtx {
    arena: Vec<TypeKind>,
    unit: TypeId,
    i32_ty: TypeId,
    f32_ty: TypeId,
    bool_ty: TypeId,
    str_ty: TypeId,
    never_ty: TypeId,
    named: alloc::collections::BTreeMap<alloc::string::String, TypeId>,
}

impl TypeCtx {
    pub fn new() -> Self {
        let mut arena = Vec::new();
        let unit = TypeId(arena.len());
        arena.push(TypeKind::Unit);
        let i32_ty = TypeId(arena.len());
        arena.push(TypeKind::I32);
        let f32_ty = TypeId(arena.len());
        arena.push(TypeKind::F32);
        let bool_ty = TypeId(arena.len());
        arena.push(TypeKind::Bool);
        let str_ty = TypeId(arena.len());
        arena.push(TypeKind::Str);
        let never_ty = TypeId(arena.len());
        arena.push(TypeKind::Never);

        Self {
            arena,
            unit,
            i32_ty,
            f32_ty,
            bool_ty,
            str_ty,
            never_ty,
            named: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn unit(&self) -> TypeId {
        self.unit
    }
    pub fn i32(&self) -> TypeId {
        self.i32_ty
    }
    pub fn f32(&self) -> TypeId {
        self.f32_ty
    }
    pub fn bool(&self) -> TypeId {
        self.bool_ty
    }
    pub fn str(&self) -> TypeId {
        self.str_ty
    }
    pub fn never(&self) -> TypeId {
        self.never_ty
    }

    pub fn fresh_var(&mut self, label: Option<alloc::string::String>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Var(TypeVar {
            label,
            binding: None,
        }));
        id
    }

    pub fn register_named(&mut self, name: alloc::string::String, kind: TypeKind) -> TypeId {
        if let Some(existing) = self.named.get(&name) {
            // upgrade placeholder Named to concrete kind
            let eid = *existing;
            match &self.arena[eid.0] {
                TypeKind::Named(_) => {
                    self.arena[eid.0] = kind;
                }
                _ => {}
            }
            eid
        } else {
            let id = TypeId(self.arena.len());
            self.arena.push(kind);
            self.named.insert(name, id);
            id
        }
    }

    pub fn lookup_named(&self, name: &str) -> Option<TypeId> {
        self.named.get(name).copied()
    }

    pub fn function(&mut self, params: Vec<TypeId>, result: TypeId, effect: Effect) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Function {
            params,
            result,
            effect,
        });
        id
    }

    pub fn get(&self, id: TypeId) -> TypeKind {
        match &self.arena[id.0] {
            TypeKind::Var(tv) => {
                if let Some(b) = tv.binding {
                    self.get(b)
                } else {
                    TypeKind::Var(tv.clone())
                }
            }
            other => other.clone(),
        }
    }

    pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<TypeId, UnifyError> {
        let ak = self.get(a);
        let bk = self.get(b);

        match (ak, bk) {
            (TypeKind::Var(mut va), TypeKind::Var(mut vb)) => {
                if let (Some(la), Some(lb)) = (&va.label, &vb.label) {
                    if la != lb {
                        return Err(UnifyError::Mismatch);
                    }
                }
                let target = if va.label.is_some() { a } else { b };
                self.bind_var(target, if target == a { b } else { a });
                Ok(target)
            }
            (TypeKind::Var(mut va), other) => {
                if let Some(label) = va.label.take() {
                    if !label_matches(&label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                self.bind_var_value(a, &other);
                Ok(b)
            }
            (other, TypeKind::Var(mut vb)) => {
                if let Some(label) = vb.label.take() {
                    if !label_matches(&label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                self.bind_var_value(b, &other);
                Ok(a)
            }
            (TypeKind::Unit, TypeKind::Unit) => Ok(self.unit),
            (TypeKind::I32, TypeKind::I32) => Ok(self.i32_ty),
            (TypeKind::F32, TypeKind::F32) => Ok(self.f32_ty),
            (TypeKind::Str, TypeKind::I32) | (TypeKind::I32, TypeKind::Str) => {
                Ok(self.i32_ty)
            }
            (TypeKind::Bool, TypeKind::Bool) => Ok(self.bool_ty),
            (TypeKind::Str, TypeKind::Str) => Ok(self.str_ty),
            (TypeKind::Never, _) => Ok(b),
            (_, TypeKind::Never) => Ok(a),
            (TypeKind::Named(na), TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (
                TypeKind::Enum {
                    name: na,
                    variants: va,
                },
                TypeKind::Enum {
                    name: nb,
                    variants: vb,
                },
            ) => {
                if na != nb || va.len() != vb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (a_var, b_var) in va.iter().zip(vb.iter()) {
                    if a_var.name != b_var.name {
                        return Err(UnifyError::Mismatch);
                    }
                    if let (Some(pa), Some(pb)) = (a_var.payload, b_var.payload) {
                        self.unify(pa, pb)?;
                    } else if a_var.payload.is_some() || b_var.payload.is_some() {
                        return Err(UnifyError::Mismatch);
                    }
                }
                Ok(a)
            }
            (
                TypeKind::Struct { name: na, fields: fa },
                TypeKind::Struct { name: nb, fields: fb },
            ) => {
                if na != nb || fa.len() != fb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (ta, tb) in fa.iter().zip(fb.iter()) {
                    self.unify(*ta, *tb)?;
                }
                Ok(a)
            }
            (
                TypeKind::Function {
                    params: pa,
                    result: ra,
                    effect: ea,
                },
                TypeKind::Function {
                    params: pb,
                    result: rb,
                    effect: eb,
                },
            ) => {
                if ea != eb || pa.len() != pb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in pa.iter().zip(pb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                self.unify(ra, rb)?;
                Ok(self.function(pa, ra, ea))
            }
            _ => Err(UnifyError::Mismatch),
        }
    }

    fn bind_var(&mut self, var: TypeId, target: TypeId) {
        if let TypeKind::Var(tv) = &mut self.arena[var.0] {
            tv.binding = Some(target);
        }
    }

    fn bind_var_value(&mut self, var: TypeId, value: &TypeKind) {
        self.arena[var.0] = TypeKind::Var(TypeVar {
            label: match value {
                TypeKind::Var(tv) => tv.label.clone(),
                _ => None,
            },
            binding: Some(self.store(value.clone())),
        });
    }

    fn store(&mut self, kind: TypeKind) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(kind);
        id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifyError {
    Mismatch,
}

fn label_matches(label: &str, ty: &TypeKind) -> bool {
    match ty {
        TypeKind::Var(tv) => tv.label.as_ref().map(|l| l == label).unwrap_or(true),
        _ => true,
    }
}

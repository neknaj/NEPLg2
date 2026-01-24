#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::Effect;

/// Identifier for a type stored in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
        type_params: Vec<TypeId>, // TypeId(Var)
        variants: Vec<EnumVariantInfo>,
    },
    Struct {
        name: String,
        type_params: Vec<TypeId>, // TypeId(Var)
        fields: Vec<TypeId>,
    },
    Function {
        type_params: Vec<TypeId>, // new
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
    },
    Var(TypeVar),
    Apply {
        base: TypeId,
        args: Vec<TypeId>,
    },
    Box(TypeId),
    Reference(TypeId, bool),
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

    pub fn function(
        &mut self,
        type_params: Vec<TypeId>,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
    ) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        });
        id
    }

    pub fn is_copy(&self, id: TypeId) -> bool {
        match self.get_ref(id) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Str
            | TypeKind::Never => true,
            TypeKind::Reference(_, _) => true,
            TypeKind::Box(_) => false,
            TypeKind::Enum { .. } => false,
            TypeKind::Struct { .. } => false,
            TypeKind::Apply { .. } => false,
            TypeKind::Function { .. } => false,
            TypeKind::Var(v) => {
                if let Some(b) = v.binding {
                    self.is_copy(b)
                } else {
                    false
                }
            }
            TypeKind::Named(_) => false,
        }
    }

    pub fn get_ref(&self, id: TypeId) -> &TypeKind {
        &self.arena[id.0]
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
            (TypeKind::Var(va), TypeKind::Var(vb)) => {
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
            (TypeKind::Str, TypeKind::I32) | (TypeKind::I32, TypeKind::Str) => Ok(self.i32_ty),
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
                    type_params: _,
                    variants: va,
                },
                TypeKind::Enum {
                    name: nb,
                    type_params: _,
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
                TypeKind::Struct {
                    name: na,
                    fields: fa,
                    type_params: _,
                },
                TypeKind::Struct {
                    name: nb,
                    fields: fb,
                    type_params: _,
                },
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
                    type_params: ta,
                    params: pa,
                    result: ra,
                    effect: ea,
                },
                TypeKind::Function {
                    type_params: tb,
                    params: pb,
                    result: rb,
                    effect: eb,
                },
            ) => {
                if ea != eb || pa.len() != pb.len() || ta.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in ta.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                for (xa, xb) in pa.iter().zip(pb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                self.unify(ra, rb)?;
                Ok(self.function(ta, pa, ra, ea))
            }
            (TypeKind::Box(inner_a), TypeKind::Box(inner_b)) => {
                let inner = self.unify(inner_a, inner_b)?;
                Ok(self.box_ty(inner))
            }
            (TypeKind::Reference(inner_a, mut_a), TypeKind::Reference(inner_b, mut_b)) => {
                if mut_a != mut_b {
                    return Err(UnifyError::Mismatch);
                }
                let inner = self.unify(inner_a, inner_b)?;
                Ok(self.reference(inner, mut_a))
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

    pub fn apply(&mut self, base: TypeId, args: Vec<TypeId>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Apply { base, args });
        id
    }

    pub fn reference(&mut self, inner: TypeId, is_mut: bool) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Reference(inner, is_mut));
        id
    }

    pub fn box_ty(&mut self, inner: TypeId) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Box(inner));
        id
    }

    pub fn substitute(
        &mut self,
        ty: TypeId,
        mapping: &alloc::collections::BTreeMap<TypeId, TypeId>,
    ) -> TypeId {
        if let Some(target) = mapping.get(&ty) {
            return *target;
        }
        match self.get(ty) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Str
            | TypeKind::Never => ty,
            TypeKind::Named(_) => ty,
            TypeKind::Var(_) => ty,
            TypeKind::Enum {
                name,
                type_params,
                variants,
            } => {
                let mut new_tps = Vec::new();
                for tp in type_params {
                    new_tps.push(self.substitute(tp, mapping));
                }
                let mut new_vars = Vec::new();
                for v in variants {
                    new_vars.push(EnumVariantInfo {
                        name: v.name.clone(),
                        payload: v.payload.map(|p| self.substitute(p, mapping)),
                    });
                }
                self.store(TypeKind::Enum {
                    name,
                    type_params: new_tps,
                    variants: new_vars,
                })
            }
            TypeKind::Struct {
                name,
                type_params,
                fields,
            } => {
                let mut new_tps = Vec::new();
                for tp in type_params {
                    new_tps.push(self.substitute(tp, mapping));
                }
                let mut new_fs = Vec::new();
                for f in fields {
                    new_fs.push(self.substitute(f, mapping));
                }
                self.store(TypeKind::Struct {
                    name,
                    type_params: new_tps,
                    fields: new_fs,
                })
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let mut new_tps = Vec::new();
                for tp in type_params {
                    new_tps.push(self.substitute(tp, mapping));
                }
                let mut new_ps = Vec::new();
                for p in params {
                    new_ps.push(self.substitute(p, mapping));
                }
                let new_r = self.substitute(result, mapping);
                self.function(new_tps, new_ps, new_r, effect)
            }
            TypeKind::Apply { base, args } => {
                let mut new_args = Vec::new();
                for a in args {
                    new_args.push(self.substitute(a, mapping));
                }
                let new_base = self.substitute(base, mapping);
                self.apply(new_base, new_args)
            }
            TypeKind::Box(inner) => {
                let ni = self.substitute(inner, mapping);
                self.box_ty(ni)
            }
            TypeKind::Reference(inner, is_mut) => {
                let ni = self.substitute(inner, mapping);
                self.reference(ni, is_mut)
            }
        }
    }

    pub fn resolve(&mut self, ty: TypeId) -> TypeId {
        match self.get(ty) {
            TypeKind::Apply { base, args } => {
                let base_ty = self.resolve(base);
                match self.get(base_ty) {
                    TypeKind::Enum { type_params, .. }
                    | TypeKind::Struct { type_params, .. }
                    | TypeKind::Function { type_params, .. } => {
                        if type_params.len() != args.len() {
                            // This should be a diagnostic, but here we just return ty
                            return ty;
                        }
                        let mut mapping = alloc::collections::BTreeMap::new();
                        for (tp, arg) in type_params.iter().zip(args.iter()) {
                            mapping.insert(*tp, *arg);
                        }
                        // We want to substitute into the base, but we need to avoid infinite recursion if base is Apply again (unlikely)
                        // Actually, substitute into the base KIND but without the type_params becoming empty.
                        // Or better: create a version where type_params are substituted.
                        match self.get(base_ty) {
                            TypeKind::Enum { name, variants, .. } => {
                                let mut new_vars = Vec::new();
                                for v in variants {
                                    new_vars.push(EnumVariantInfo {
                                        name: v.name.clone(),
                                        payload: v.payload.map(|p| self.substitute(p, &mapping)),
                                    });
                                }
                                self.store(TypeKind::Enum {
                                    name,
                                    type_params: Vec::new(),
                                    variants: new_vars,
                                })
                            }
                            TypeKind::Struct { name, fields, .. } => {
                                let mut new_fs = Vec::new();
                                for f in fields {
                                    new_fs.push(self.substitute(f, &mapping));
                                }
                                self.store(TypeKind::Struct {
                                    name,
                                    type_params: Vec::new(),
                                    fields: new_fs,
                                })
                            }
                            TypeKind::Function {
                                params,
                                result,
                                effect,
                                ..
                            } => {
                                let mut new_ps = Vec::new();
                                for p in params {
                                    new_ps.push(self.substitute(p, &mapping));
                                }
                                let new_r = self.substitute(result, &mapping);
                                self.function(Vec::new(), new_ps, new_r, effect)
                            }
                            _ => ty,
                        }
                    }
                    _ => ty,
                }
            }
            _ => ty,
        }
    }

    pub fn instantiate(&mut self, ty: TypeId) -> (TypeId, Vec<TypeId>) {
        if let TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } = self.get(ty)
        {
            if type_params.is_empty() {
                return (ty, Vec::new());
            }
            let mut mapping = alloc::collections::BTreeMap::new();
            let mut fresh_args = Vec::new();
            for tp in &type_params {
                let fresh = self.fresh_var(None);
                mapping.insert(*tp, fresh);
                fresh_args.push(fresh);
            }
            let new_params = params
                .iter()
                .map(|p| self.substitute(*p, &mapping))
                .collect();
            let new_result = self.substitute(result, &mapping);
            (
                self.function(Vec::new(), new_params, new_result, effect),
                fresh_args,
            )
        } else {
            (ty, Vec::new())
        }
    }

    pub fn type_to_string(&self, ty: TypeId) -> String {
        match self.get(ty) {
            TypeKind::Unit => String::from("unit"),
            TypeKind::I32 => String::from("i32"),
            TypeKind::F32 => String::from("f32"),
            TypeKind::Bool => String::from("bool"),
            TypeKind::Str => String::from("str"),
            TypeKind::Never => String::from("never"),
            TypeKind::Named(name) => name.clone(),
            TypeKind::Enum { name, .. } => name.clone(),
            TypeKind::Struct { name, .. } => name.clone(),
            TypeKind::Function { .. } => String::from("func"),
            TypeKind::Var(_) => String::from("var"),
            TypeKind::Apply { base, args } => {
                let mut s = self.type_to_string(base);
                s.push('_');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string(*arg));
                }
                s
            }
            TypeKind::Box(inner) => {
                let mut s = String::from("box_");
                s.push_str(&self.type_to_string(inner));
                s
            }
            TypeKind::Reference(inner, is_mut) => {
                let mut s = if is_mut {
                    String::from("refmut_")
                } else {
                    String::from("ref_")
                };
                s.push_str(&self.type_to_string(inner));
                s
            }
        }
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

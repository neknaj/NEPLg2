#![no_std]
extern crate alloc;
extern crate std;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::Effect;

/// Identifier for a type stored in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Unit,
    I32,
    U8,
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
        field_names: Vec<String>,
    },
    Tuple {
        items: Vec<TypeId>,
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
#[derive(Debug, Clone)]
pub struct TypeCtx {
    arena: Vec<TypeKind>,
    unit: TypeId,
    i32_ty: TypeId,
    u8_ty: TypeId,
    f32_ty: TypeId,
    bool_ty: TypeId,
    str_ty: TypeId,
    never_ty: TypeId,
    named: alloc::collections::BTreeMap<alloc::string::String, TypeId>,
}

static GLOBAL_UNIFY_DEPTH: AtomicUsize = AtomicUsize::new(0);

struct UnifyDepthGuard;
impl Drop for UnifyDepthGuard {
    fn drop(&mut self) {
        GLOBAL_UNIFY_DEPTH.fetch_sub(1, Ordering::SeqCst);
    }
}

impl TypeCtx {
    pub fn new() -> Self {
        let mut arena = Vec::new();
        let unit = TypeId(arena.len());
        arena.push(TypeKind::Unit);
        let i32_ty = TypeId(arena.len());
        arena.push(TypeKind::I32);
        let u8_ty = TypeId(arena.len());
        arena.push(TypeKind::U8);
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
            u8_ty,
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
    pub fn u8(&self) -> TypeId {
        self.u8_ty
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

    pub fn tuple(&mut self, items: Vec<TypeId>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Tuple { items });
        id
    }

    pub fn resolve_id(&self, id: TypeId) -> TypeId {
        let mut cur = id;
        let mut i = 0;
        loop {
            if i > 5000 {
                return cur;
            }
            match &self.arena[cur.0] {
                TypeKind::Var(tv) => {
                    if let Some(next) = tv.binding {
                        cur = next;
                    } else {
                        return cur;
                    }
                }
                _ => return cur,
            }
            i += 1;
        }
    }

    pub fn is_copy(&self, id: TypeId) -> bool {
        let mut seen = BTreeSet::new();
        self.is_copy_inner(id, &mut seen)
    }

    fn is_copy_inner(&self, id: TypeId, seen: &mut BTreeSet<TypeId>) -> bool {
        let resolved = self.resolve_id(id);
        if !seen.insert(resolved) {
            return false;
        }
        match self.get_ref(resolved) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Str
            | TypeKind::Never => true,
            TypeKind::Reference(_, _) => true,
            TypeKind::Box(_) => false,
            TypeKind::Enum { .. } => false,
            TypeKind::Struct { .. } => false,
            TypeKind::Tuple { items } => items.iter().all(|t| self.is_copy_inner(*t, seen)),
            TypeKind::Apply { .. } => false,
            TypeKind::Function { .. } => false,
            TypeKind::Var(v) => {
                if let Some(b) = v.binding {
                    self.is_copy_inner(b, seen)
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
        let resolved = self.resolve_id(id);
        match &self.arena[resolved.0] {
            TypeKind::Var(tv) => TypeKind::Var(tv.clone()),
            other => other.clone(),
        }
    }

    pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<TypeId, UnifyError> {
        // recursion guard to avoid native stack overflow in pathological cases
        let depth = GLOBAL_UNIFY_DEPTH.fetch_add(1, Ordering::SeqCst) + 1;
        let _guard = UnifyDepthGuard;
        if depth > 5000 {
            return Err(UnifyError::Mismatch);
        }

        let ra = self.resolve_id(a);
        let rb = self.resolve_id(b);
        if ra != a || rb != b {
            return self.unify(ra, rb);
        }
        if crate::log::is_verbose() {
            std::eprintln!("unify: {:?} with {:?}", self.get(ra), self.get(rb));
        }
        let ra = self.resolve(ra);
        let rb = self.resolve(rb);
        if ra != a || rb != b {
            return self.unify(ra, rb);
        }
        if ra == rb {
            return Ok(ra);
        }
        if self.apply_arity_mismatch(a) || self.apply_arity_mismatch(b) {
            return Err(UnifyError::Mismatch);
        }
        let ak = self.arena[ra.0].clone();
        let bk = self.arena[rb.0].clone();

        match (ak, bk) {
            (TypeKind::Var(_), TypeKind::Never) => Ok(a),
            (TypeKind::Never, TypeKind::Var(_)) => Ok(b),
            (TypeKind::Var(va), TypeKind::Var(vb)) => {
                if let (Some(la), Some(lb)) = (&va.label, &vb.label) {
                    if la != lb && la != "Self" && lb != "Self" {
                        return Err(UnifyError::Mismatch);
                    }
                }
                match (va.label.is_some(), vb.label.is_some()) {
                    (true, false) => {
                        self.bind_var(b, a);
                        Ok(a)
                    }
                    (false, true) => {
                        self.bind_var(a, b);
                        Ok(b)
                    }
                    _ => {
                        self.bind_var(b, a);
                        Ok(a)
                    }
                }
            }
            (TypeKind::Var(va), other) => {
                if self.occurs_in(ra, rb, &mut BTreeSet::new()) {
                    return Err(UnifyError::Mismatch);
                }
                if let Some(label) = &va.label {
                    if !label_matches(label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                let other_cloned = other.clone();
                self.bind_var_value(ra, &other_cloned);
                Ok(rb)
            }
            (other, TypeKind::Var(vb)) => {
                if self.occurs_in(rb, ra, &mut BTreeSet::new()) {
                    return Err(UnifyError::Mismatch);
                }
                if let Some(label) = &vb.label {
                    if !label_matches(label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                let other_cloned = other.clone();
                self.bind_var_value(rb, &other_cloned);
                Ok(ra)
            }
            (TypeKind::Unit, TypeKind::Unit) => Ok(self.unit),
            (TypeKind::I32, TypeKind::I32) => Ok(self.i32_ty),
            (TypeKind::U8, TypeKind::U8) => Ok(self.u8_ty),
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
                        if let Err(e) = self.unify(pa, pb) {
                            if crate::log::is_verbose() {
                                std::eprintln!("unify: variant {} payload mismatch", a_var.name);
                            }
                            return Err(e);
                        }
                    } else if a_var.payload.is_some() || b_var.payload.is_some() {
                        if crate::log::is_verbose() {
                            std::eprintln!(
                                "unify: variant {} payload presence mismatch",
                                a_var.name
                            );
                        }
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
                    field_names: _,
                },
                TypeKind::Struct {
                    name: nb,
                    fields: fb,
                    type_params: _,
                    field_names: _,
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
            (TypeKind::Tuple { items: ta }, TypeKind::Tuple { items: tb }) => {
                if ta.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in ta.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
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
                Ok(self.function(ta.clone(), pa.clone(), ra, ea))
            }
            (TypeKind::Named(na), TypeKind::Enum { name: nb, .. })
            | (TypeKind::Enum { name: na, .. }, TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (TypeKind::Named(na), TypeKind::Struct { name: nb, .. })
            | (TypeKind::Struct { name: na, .. }, TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (TypeKind::Box(inner_a), TypeKind::Box(inner_b)) => {
                self.unify(inner_a, inner_b)?;
                Ok(a)
            }
            (TypeKind::Reference(inner_a, mut_a), TypeKind::Reference(inner_b, mut_b)) => {
                if mut_a != mut_b {
                    return Err(UnifyError::Mismatch);
                }
                self.unify(inner_a, inner_b)?;
                Ok(a)
            }
            (TypeKind::Tuple { items: ta }, TypeKind::Tuple { items: tb }) => {
                if ta.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in ta.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (TypeKind::Apply { base: ba, args: aa }, TypeKind::Apply { base: bb, args: ab }) => {
                if aa.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                self.unify(ba, bb)?;
                for (xa, xb) in aa.iter().zip(ab.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (TypeKind::Enum { name: na, type_params: ta, .. }, TypeKind::Apply { base: bb, args: ab }) => {
                if ta.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(bb); // Use resolve_id for simple lookup
                match &self.arena[resolved_base.0] {
                    TypeKind::Enum { name: nb, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(nb) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                for (xa, xb) in ta.iter().zip(ab.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (TypeKind::Apply { base: ba, args: aa }, TypeKind::Enum { name: nb, type_params: tb, .. }) => {
                if aa.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(ba);
                match &self.arena[resolved_base.0] {
                    TypeKind::Enum { name: na, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(na) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                for (xa, xb) in aa.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (TypeKind::Struct { name: na, type_params: ta, .. }, TypeKind::Apply { base: bb, args: ab }) => {
                if ta.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(bb);
                match &self.arena[resolved_base.0] {
                    TypeKind::Struct { name: nb, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(nb) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                for (xa, xb) in ta.iter().zip(ab.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (TypeKind::Apply { base: ba, args: aa }, TypeKind::Struct { name: nb, type_params: tb, .. }) => {
                if aa.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(ba);
                match &self.arena[resolved_base.0] {
                    TypeKind::Struct { name: na, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(na) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                for (xa, xb) in aa.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
        _ => Err(UnifyError::Mismatch),
        }
    }

    fn bind_var(&mut self, var: TypeId, target: TypeId) {
        let target = self.resolve_id(target);
        if target == var {
            return;
        }
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
        let mut seen = BTreeSet::new();
        self.substitute_inner(ty, mapping, &mut seen)
    }

    fn substitute_inner(
        &mut self,
        ty: TypeId,
        mapping: &alloc::collections::BTreeMap<TypeId, TypeId>,
        seen: &mut BTreeSet<TypeId>,
    ) -> TypeId {
        let ty = self.resolve_id(ty);
        if let Some(target) = mapping.get(&ty) {
            // std::eprintln!("substitute: found {:?} -> {:?}", ty, target);
            return *target;
        }
        // std::eprintln!("substitute: NOT found {:?} in {:?}", ty, mapping.keys().collect::<Vec<_>>());
        if !seen.insert(ty) {
            return ty;
        }
        let kind = self.arena[ty.0].clone();
        match kind {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
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
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp { changed = true; }
                    new_tps.push(nt);
                }
                let mut new_vars = Vec::new();
                for v in variants {
                    let new_payload = v.payload.map(|p| {
                        let np = self.substitute_inner(p, mapping, seen);
                        if np != p { changed = true; }
                        np
                    });
                    new_vars.push(EnumVariantInfo {
                        name: v.name.clone(),
                        payload: new_payload,
                    });
                }
                if changed {
                    self.store(TypeKind::Enum {
                        name: name.clone(),
                        type_params: new_tps,
                        variants: new_vars,
                    })
                } else {
                    ty
                }
            }
            TypeKind::Struct {
                name,
                type_params,
                fields,
                field_names,
            } => {
                let mut new_tps = Vec::new();
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp { changed = true; }
                    new_tps.push(nt);
                }
                let mut new_fs = Vec::new();
                for f in fields {
                    let nf = self.substitute_inner(f, mapping, seen);
                    if nf != f { changed = true; }
                    new_fs.push(nf);
                }
                if changed {
                    self.store(TypeKind::Struct {
                        name: name.clone(),
                        type_params: new_tps,
                        fields: new_fs,
                        field_names: field_names.clone(),
                    })
                } else {
                    ty
                }
            }
            TypeKind::Tuple { items } => {
                let mut new_items = Vec::new();
                let mut changed = false;
                for item in items {
                    let ni = self.substitute_inner(item, mapping, seen);
                    if ni != item { changed = true; }
                    new_items.push(ni);
                }
                if changed {
                    self.store(TypeKind::Tuple { items: new_items })
                } else {
                    ty
                }
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let mut new_tps = Vec::new();
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp { changed = true; }
                    new_tps.push(nt);
                }
                let mut new_ps = Vec::new();
                for p in params {
                    let np = self.substitute_inner(p, mapping, seen);
                    if np != p { changed = true; }
                    new_ps.push(np);
                }
                let new_r = self.substitute_inner(result, mapping, seen);
                if new_r != result { changed = true; }
                
                if changed {
                    self.function(new_tps, new_ps, new_r, effect)
                } else {
                    ty
                }
            }
            TypeKind::Apply { base, args } => {
                let mut new_args = Vec::new();
                let mut changed = false;
                for a in args {
                    let na = self.substitute_inner(a, mapping, seen);
                    if na != a { changed = true; }
                    new_args.push(na);
                }
                let new_base = self.substitute_inner(base, mapping, seen);
                if new_base != base { changed = true; }
                
                if changed {
                    self.apply(new_base, new_args)
                } else {
                    ty
                }
            }
            TypeKind::Box(inner) => {
                let ni = self.substitute_inner(inner, mapping, seen);
                if ni != inner {
                    self.box_ty(ni)
                } else {
                    ty
                }
            }
            TypeKind::Reference(inner, is_mut) => {
                let ni = self.substitute_inner(inner, mapping, seen);
                if ni != inner {
                    self.reference(ni, is_mut)
                } else {
                    ty
                }
            }
        }
    }

    pub fn resolve(&mut self, ty: TypeId) -> TypeId {
        match self.get(ty) {
            TypeKind::Named(name) => {
                if let Some(actual) = self.named.get(&name).copied() {
                    if actual == ty {
                        return ty;
                    }
                    return self.resolve(actual);
                }
                ty
            }
            _ => ty,
        }
    }

    pub fn instantiate(&mut self, ty: TypeId) -> (TypeId, Vec<TypeId>) {
        let ty = self.resolve_id(ty);
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
        let mut seen = BTreeSet::new();
        self.type_to_string_inner(ty, &mut seen)
    }

    fn type_to_string_inner(&self, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> String {
        let ty = self.resolve_id(ty);
        if !seen.insert(ty) {
            std::eprintln!("CYCLE DETECTED in type_to_string: {:?}", ty);
            return String::from("cycle");
        }
        let res = match self.get(ty) {
            TypeKind::Unit => String::from("unit"),
            TypeKind::I32 => String::from("i32"),
            TypeKind::U8 => String::from("u8"),
            TypeKind::F32 => String::from("f32"),
            TypeKind::Bool => String::from("bool"),
            TypeKind::Str => String::from("str"),
            TypeKind::Never => String::from("never"),
            TypeKind::Named(name) => name.clone(),
            TypeKind::Enum {
                name,
                type_params,
                ..
            } => {
                if type_params.is_empty() {
                    name.clone()
                } else {
                    let mut s = name.clone();
                    s.push('_');
                    for (i, tp) in type_params.iter().enumerate() {
                        if i > 0 {
                            s.push('_');
                        }
                        s.push_str(&self.type_to_string_inner(*tp, seen));
                    }
                    s
                }
            }
            TypeKind::Struct {
                name,
                type_params,
                ..
            } => {
                if type_params.is_empty() {
                    name.clone()
                } else {
                    let mut s = name.clone();
                    s.push('_');
                    for (i, tp) in type_params.iter().enumerate() {
                        if i > 0 {
                            s.push('_');
                        }
                        s.push_str(&self.type_to_string_inner(*tp, seen));
                    }
                    s
                }
            }
            TypeKind::Tuple { items } => {
                let mut s = String::from("tuple_");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string_inner(*item, seen));
                }
                s
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let mut s = String::from("fn");
                if !type_params.is_empty() {
                    s.push_str("_gen_");
                    s.push_str(&type_params.len().to_string());
                }
                s.push_str("__");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string_inner(*p, seen));
                }
                s.push_str("__");
                s.push_str(&self.type_to_string_inner(result, seen));
                match effect {
                    Effect::Pure => s.push_str("__pure"),
                    Effect::Impure => s.push_str("__imp"),
                }
                s
            }
            TypeKind::Var(tv) => match tv.label {
                Some(ref label) => label.clone(),
                None => format!("var_{}", ty.0),
            },
            TypeKind::Apply { base, args } => {
                let mut s = self.type_to_string_inner(base, seen);
                s.push('_');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string_inner(*arg, seen));
                }
                s
            }
            TypeKind::Box(inner) => {
                let mut s = String::from("box_");
                s.push_str(&self.type_to_string_inner(inner, seen));
                s
            }
            TypeKind::Reference(inner, is_mut) => {
                let mut s = String::from("ref_");
                if is_mut {
                    s.push_str("mut_");
                }
                s.push_str(&self.type_to_string_inner(inner, seen));
                s
            }
        };
        seen.remove(&ty);
        res
    }

    fn store(&mut self, kind: TypeKind) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(kind);
        id
    }

    fn apply_arity_mismatch(&self, ty: TypeId) -> bool {
        match self.get(ty) {
            TypeKind::Apply { base, args } => match self.get(base) {
                TypeKind::Enum { type_params, .. }
                | TypeKind::Struct { type_params, .. }
                | TypeKind::Function { type_params, .. } => type_params.len() != args.len(),
                _ => false,
            },
            _ => false,
        }
    }

    fn occurs_in(&self, var: TypeId, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> bool {
        let ty = self.resolve_id(ty);
        if ty == var {
            return true;
        }
        if !seen.insert(ty) {
            return false;
        }
        match self.get(ty) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_) => false,
            TypeKind::Var(tv) => {
                if let Some(b) = tv.binding {
                    self.occurs_in(var, b, seen)
                } else {
                    false
                }
            }
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for v in variants {
                    if let Some(p) = v.payload {
                        if self.occurs_in(var, p, seen) {
                            return true;
                        }
                    }
                }
                false
            }
            TypeKind::Struct {
                type_params,
                fields,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for f in fields {
                    if self.occurs_in(var, f, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Tuple { items } => {
                for item in items {
                    if self.occurs_in(var, item, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for p in params {
                    if self.occurs_in(var, p, seen) {
                        return true;
                    }
                }
                self.occurs_in(var, result, seen)
            }
            TypeKind::Apply { base, args } => {
                if self.occurs_in(var, base, seen) {
                    return true;
                }
                for a in args {
                    if self.occurs_in(var, a, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Box(inner) => self.occurs_in(var, inner, seen),
            TypeKind::Reference(inner, _) => self.occurs_in(var, inner, seen),
        }
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

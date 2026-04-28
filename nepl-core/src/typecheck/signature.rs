use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn mangle_function_symbol(base: &str, func_ty: TypeId, ctx: &TypeCtx) -> String {
    let mut s = String::new();
    s.push_str(base);
    if let TypeKind::Function {
        params,
        result,
        effect,
        ..
    } = ctx.get(func_ty)
    {
        s.push_str("__");
        if params.is_empty() {
            s.push_str("unit");
        } else {
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&ctx.type_to_string(*p));
            }
        }
        s.push_str("__");
        s.push_str(&ctx.type_to_string(result));
        match effect {
            Effect::Pure => s.push_str("__pure"),
            Effect::Impure => s.push_str("__imp"),
        }
    }
    s
}

pub(super) fn mangle_impl_method(
    trait_name: &str,
    method: &str,
    target_ty: TypeId,
    ctx: &TypeCtx,
) -> String {
    let mut name = String::new();
    name.push_str(trait_name);
    name.push_str("::");
    name.push_str(method);
    name.push_str("__");
    name.push_str(&ctx.type_to_string(target_ty));
    name
}

pub(super) fn function_signature_string(ctx: &TypeCtx, ty: TypeId) -> String {
    let resolved = ctx.resolve_id(ty);
    match ctx.get(resolved) {
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            let mut generics = BTreeMap::new();
            for (i, tp) in type_params.iter().enumerate() {
                let mut name = String::from("$T");
                name.push_str(&i.to_string());
                generics.insert(ctx.resolve_id(*tp), name);
            }
            let mut s = String::from("func");
            if !type_params.is_empty() {
                s.push_str("_gen_");
                s.push_str(&type_params.len().to_string());
            }
            s.push_str("__");
            if params.is_empty() {
                s.push_str("unit");
            } else {
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *p, &generics));
                }
            }
            s.push_str("__");
            s.push_str(&signature_type_string(ctx, result, &generics));
            match effect {
                Effect::Pure => s.push_str("__pure"),
                Effect::Impure => s.push_str("__imp"),
            }
            s
        }
        _ => ctx.type_to_string(resolved),
    }
}

pub(super) fn same_function_signature(ctx: &TypeCtx, a: TypeId, b: TypeId) -> bool {
    let ra = ctx.resolve_id(a);
    let rb = ctx.resolve_id(b);
    let (tpa, pa, resa, ea) = match ctx.get(ra) {
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => (type_params, params, result, effect),
        _ => return ctx.same_type(ra, rb),
    };
    let (tpb, pb, resb, eb) = match ctx.get(rb) {
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => (type_params, params, result, effect),
        _ => return false,
    };
    if ea != eb || tpa.len() != tpb.len() || pa.len() != pb.len() {
        return false;
    }
    let mut map_ab: BTreeMap<TypeId, TypeId> = BTreeMap::new();
    let mut map_ba: BTreeMap<TypeId, TypeId> = BTreeMap::new();
    for (ta, tb) in tpa.iter().zip(tpb.iter()) {
        map_ab.insert(ctx.resolve_id(*ta), ctx.resolve_id(*tb));
        map_ba.insert(ctx.resolve_id(*tb), ctx.resolve_id(*ta));
    }
    let mut seen = BTreeSet::new();
    for (ta, tb) in pa.iter().zip(pb.iter()) {
        if !same_type_with_signature_generics(ctx, *ta, *tb, &map_ab, &map_ba, &mut seen) {
            return false;
        }
    }
    same_type_with_signature_generics(ctx, resa, resb, &map_ab, &map_ba, &mut seen)
}

pub(super) fn same_type_with_signature_generics(
    ctx: &TypeCtx,
    a: TypeId,
    b: TypeId,
    map_ab: &BTreeMap<TypeId, TypeId>,
    map_ba: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<(TypeId, TypeId)>,
) -> bool {
    let ra = ctx.resolve_id(a);
    let rb = ctx.resolve_id(b);
    if ra == rb {
        return true;
    }
    if let Some(mapped) = map_ab.get(&ra) {
        return *mapped == rb;
    }
    if let Some(mapped) = map_ba.get(&rb) {
        return *mapped == ra;
    }
    let key = if ra <= rb { (ra, rb) } else { (rb, ra) };
    if !seen.insert(key) {
        return true;
    }
    let result = match (ctx.get(ra), ctx.get(rb)) {
        (TypeKind::Unit, TypeKind::Unit)
        | (TypeKind::I32, TypeKind::I32)
        | (TypeKind::U8, TypeKind::U8)
        | (TypeKind::F32, TypeKind::F32)
        | (TypeKind::Bool, TypeKind::Bool)
        | (TypeKind::Char, TypeKind::Char)
        | (TypeKind::Str, TypeKind::Str)
        | (TypeKind::Never, TypeKind::Never) => true,
        (TypeKind::Named(na), TypeKind::Named(nb)) => na == nb,
        (TypeKind::Box(ia), TypeKind::Box(ib)) => {
            same_type_with_signature_generics(ctx, ia, ib, map_ab, map_ba, seen)
        }
        (TypeKind::Reference(ia, ma), TypeKind::Reference(ib, mb)) => {
            ma == mb && same_type_with_signature_generics(ctx, ia, ib, map_ab, map_ba, seen)
        }
        (TypeKind::Tuple { items: ia }, TypeKind::Tuple { items: ib }) => {
            ia.len() == ib.len()
                && ia.iter().zip(ib.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (TypeKind::Apply { base: ba, args: aa }, TypeKind::Apply { base: bb, args: ab }) => {
            aa.len() == ab.len()
                && same_type_with_signature_generics(ctx, ba, bb, map_ab, map_ba, seen)
                && aa.iter().zip(ab.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (
            TypeKind::Function {
                type_params: tpa,
                params: pa,
                result: resa,
                effect: ea,
            },
            TypeKind::Function {
                type_params: tpb,
                params: pb,
                result: resb,
                effect: eb,
            },
        ) => {
            if ea != eb || tpa.len() != tpb.len() || pa.len() != pb.len() {
                false
            } else {
                let mut nested_ab = map_ab.clone();
                let mut nested_ba = map_ba.clone();
                for (ta, tb) in tpa.iter().zip(tpb.iter()) {
                    nested_ab.insert(ctx.resolve_id(*ta), ctx.resolve_id(*tb));
                    nested_ba.insert(ctx.resolve_id(*tb), ctx.resolve_id(*ta));
                }
                pa.iter().zip(pb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, &nested_ab, &nested_ba, seen)
                }) && same_type_with_signature_generics(
                    ctx, resa, resb, &nested_ab, &nested_ba, seen,
                )
            }
        }
        (TypeKind::Var(va), TypeKind::Var(vb)) => match (va.binding, vb.binding) {
            (Some(ba), Some(bb)) => {
                same_type_with_signature_generics(ctx, ba, bb, map_ab, map_ba, seen)
            }
            (Some(ba), None) => {
                same_type_with_signature_generics(ctx, ba, rb, map_ab, map_ba, seen)
            }
            (None, Some(bb)) => {
                same_type_with_signature_generics(ctx, ra, bb, map_ab, map_ba, seen)
            }
            (None, None) => va.label == vb.label,
        },
        (TypeKind::Var(va), _) => va
            .binding
            .map(|ba| same_type_with_signature_generics(ctx, ba, rb, map_ab, map_ba, seen))
            .unwrap_or(false),
        (_, TypeKind::Var(vb)) => vb
            .binding
            .map(|bb| same_type_with_signature_generics(ctx, ra, bb, map_ab, map_ba, seen))
            .unwrap_or(false),
        (
            TypeKind::Struct {
                name: na,
                type_params: tpa,
                fields: fa,
                field_names: fna,
                ..
            },
            TypeKind::Struct {
                name: nb,
                type_params: tpb,
                fields: fb,
                field_names: fnb,
                ..
            },
        ) => {
            na == nb
                && fna == fnb
                && tpa.len() == tpb.len()
                && fa.len() == fb.len()
                && tpa.iter().zip(tpb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
                && fa.iter().zip(fb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (
            TypeKind::Enum {
                name: na,
                type_params: tpa,
                variants: va,
                ..
            },
            TypeKind::Enum {
                name: nb,
                type_params: tpb,
                variants: vb,
                ..
            },
        ) => {
            na == nb
                && tpa.len() == tpb.len()
                && va.len() == vb.len()
                && tpa.iter().zip(tpb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
                && va.iter().zip(vb.iter()).all(|(a, b)| {
                    a.name == b.name
                        && match (a.payload, b.payload) {
                            (Some(pa), Some(pb)) => {
                                same_type_with_signature_generics(ctx, pa, pb, map_ab, map_ba, seen)
                            }
                            (None, None) => true,
                            _ => false,
                        }
                })
        }
        _ => false,
    };
    seen.remove(&key);
    result
}

pub(super) fn signature_type_string(
    ctx: &TypeCtx,
    ty: TypeId,
    generics: &BTreeMap<TypeId, String>,
) -> String {
    let resolved = ctx.resolve_id(ty);
    if let Some(name) = generics.get(&resolved) {
        return name.clone();
    }
    match ctx.get(resolved) {
        TypeKind::Unit => String::from("unit"),
        TypeKind::I32 => String::from("i32"),
        TypeKind::U8 => String::from("u8"),
        TypeKind::F32 => String::from("f32"),
        TypeKind::Bool => String::from("bool"),
        TypeKind::Char => String::from("char"),
        TypeKind::Str => String::from("str"),
        TypeKind::Never => String::from("never"),
        TypeKind::Named(name) => name,
        TypeKind::Var(tv) => {
            if let Some(binding) = tv.binding {
                signature_type_string(ctx, binding, generics)
            } else {
                tv.label.unwrap_or_else(|| format!("var_{}", resolved.0))
            }
        }
        TypeKind::Tuple { items } => {
            let mut s = String::from("tuple_");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *item, generics));
            }
            s
        }
        TypeKind::Apply { base, args } => {
            let mut s = signature_type_string(ctx, base, generics);
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *arg, generics));
            }
            s
        }
        TypeKind::Box(inner) => {
            let mut s = String::from("box_");
            s.push_str(&signature_type_string(ctx, inner, generics));
            s
        }
        TypeKind::Reference(inner, is_mut) => {
            let mut s = String::from("ref_");
            if is_mut {
                s.push_str("mut_");
            }
            s.push_str(&signature_type_string(ctx, inner, generics));
            s
        }
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => {
            let mut s = String::from("fn__");
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *p, generics));
            }
            s.push_str("__");
            s.push_str(&signature_type_string(ctx, result, generics));
            match effect {
                Effect::Pure => s.push_str("__pure"),
                Effect::Impure => s.push_str("__imp"),
            }
            s
        }
        TypeKind::Enum {
            name, type_params, ..
        } => {
            if type_params.is_empty() {
                name
            } else {
                let mut s = name;
                s.push('_');
                for (i, tp) in type_params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *tp, generics));
                }
                s
            }
        }
        TypeKind::Struct {
            name, type_params, ..
        } => {
            if type_params.is_empty() {
                name
            } else {
                let mut s = name;
                s.push('_');
                for (i, tp) in type_params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *tp, generics));
                }
                s
            }
        }
    }
}

pub(super) fn contains_same_type(ctx: &TypeCtx, list: &[TypeId], ty: TypeId) -> bool {
    list.iter().any(|t| ctx.same_type(*t, ty))
}

pub(super) fn push_unique_type(ctx: &TypeCtx, list: &mut Vec<TypeId>, ty: TypeId) {
    if !contains_same_type(ctx, list, ty) {
        list.push(ctx.resolve_id(ty));
    }
}

pub(super) fn type_contains_unbound_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let ty = ctx.resolve_id(ty);
    match ctx.get(ty) {
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => false,
        TypeKind::Var(tv) => tv.binding.is_none(),
        TypeKind::Enum { type_params, .. } | TypeKind::Struct { type_params, .. } => {
            !type_params.is_empty()
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            if !type_params.is_empty() {
                return true;
            }
            params.iter().any(|p| type_contains_unbound_var(ctx, *p))
                || type_contains_unbound_var(ctx, result)
        }
        TypeKind::Tuple { items } => items.iter().any(|t| type_contains_unbound_var(ctx, *t)),
        TypeKind::Apply { base: _, args } => {
            args.iter().any(|t| type_contains_unbound_var(ctx, *t))
        }
        TypeKind::Box(inner) => type_contains_unbound_var(ctx, inner),
        TypeKind::Reference(inner, _) => type_contains_unbound_var(ctx, inner),
    }
}

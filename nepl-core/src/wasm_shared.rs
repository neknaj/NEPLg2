extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind, HirFunction, HirModule, HirParam};
use crate::types::{TypeCtx, TypeId, TypeKind};
use wasm_encoder::{Instruction, MemArg, ValType};

fn valtype(kind: &TypeKind) -> Option<ValType> {
    match kind {
        TypeKind::Unit => None,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Str => Some(ValType::I32),
        TypeKind::F32 => Some(ValType::F32),
        TypeKind::Enum { .. } | TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            Some(ValType::I32)
        }
        TypeKind::Reference(_, _) | TypeKind::Box(_) => Some(ValType::I32),
        TypeKind::Function { .. } => Some(ValType::I32),
        TypeKind::Var(_) => Some(ValType::I32),
        TypeKind::Named(name) => match name.as_str() {
            "i64" | "u64" => Some(ValType::I64),
            "f64" => Some(ValType::F64),
            _ => Some(ValType::I32),
        },
        TypeKind::Apply { .. } => Some(ValType::I32),
        _ => None,
    }
}

fn param_valtype(kind: &TypeKind) -> Option<Option<ValType>> {
    match kind {
        TypeKind::Unit => Some(None),
        _ => valtype(kind).map(Some),
    }
}

pub(crate) fn wasm_sig(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[HirParam],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(ctx.resolve_id(p.ty));
        match param_valtype(&vk) {
            Some(Some(v)) => param_types.push(v),
            Some(None) => {}
            None => return None,
        }
    }
    let res_kind = ctx.get(ctx.resolve_id(result));
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            return None;
        }
        Vec::new()
    };
    Some((param_types, res))
}

pub(crate) fn wasm_sig_ids(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[TypeId],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(ctx.resolve_id(*p));
        match param_valtype(&vk) {
            Some(Some(v)) => param_types.push(v),
            Some(None) => {}
            None => return None,
        }
    }
    let res_kind = ctx.get(ctx.resolve_id(result));
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            return None;
        }
        Vec::new()
    };
    Some((param_types, res))
}

fn has_unbound_type_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let resolved = ctx.resolve_id(ty);
    match ctx.get(resolved) {
        TypeKind::Var(tv) => match tv.binding {
            Some(next) => has_unbound_type_var(ctx, next),
            None => true,
        },
        TypeKind::Enum { type_params, .. } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || fields.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Tuple { items } => items.iter().any(|t| has_unbound_type_var(ctx, *t)),
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || has_unbound_type_var(ctx, result)
        }
        TypeKind::Apply { base, args } => match ctx.get(ctx.resolve_id(base)) {
            TypeKind::Enum { .. } | TypeKind::Struct { .. } => {
                args.iter().any(|t| has_unbound_type_var(ctx, *t))
            }
            _ => {
                has_unbound_type_var(ctx, base)
                    || args.iter().any(|t| has_unbound_type_var(ctx, *t))
            }
        },
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => has_unbound_type_var(ctx, inner),
        _ => false,
    }
}

pub(crate) fn should_skip_wasm_codegen_for_generic(ctx: &TypeCtx, f: &HirFunction) -> bool {
    let fty = ctx.get(ctx.resolve_id(f.func_ty));
    if let TypeKind::Function {
        type_params,
        params,
        result,
        ..
    } = fty
    {
        if !type_params.is_empty() {
            return true;
        }
        if has_unbound_type_var(ctx, result) {
            return true;
        }
        for p in params {
            if has_unbound_type_var(ctx, p) {
                return true;
            }
        }
    }
    false
}

fn collect_called_functions_from_expr(
    expr: &HirExpr,
    out: &mut BTreeSet<String>,
    has_indirect: &mut bool,
) {
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Call { callee, args } => {
                if let FuncRef::User(name, _) = callee {
                    out.insert(name.clone());
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                *has_indirect = true;
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                stack.push(else_branch);
                stack.push(then_branch);
                stack.push(cond);
            }
            HirExprKind::While { cond, body } => {
                stack.push(body);
                stack.push(cond);
            }
            HirExprKind::Match { scrutinee, arms } => {
                for arm in arms.iter().rev() {
                    stack.push(&arm.body);
                }
                stack.push(scrutinee);
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::Block(block) => {
                for line in block.lines.iter().rev() {
                    stack.push(&line.expr);
                }
            }
            HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                stack.push(value);
            }
            HirExprKind::Intrinsic { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                stack.push(inner);
            }
            HirExprKind::Var(name) | HirExprKind::FnValue(name) => {
                out.insert(name.clone());
            }
            HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
}

pub(crate) fn collect_reachable_wasm_functions(module: &HirModule) -> BTreeSet<String> {
    let all_names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
    if all_names.is_empty() {
        return all_names;
    }

    let mut roots = BTreeSet::new();
    if let Some(entry) = &module.entry {
        if all_names.contains(entry) {
            roots.insert(entry.clone());
        }
    }
    if roots.is_empty() {
        return all_names;
    }

    let mut map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        map.insert(f.name.clone(), f);
    }

    let mut reachable = BTreeSet::new();
    let mut stack: Vec<String> = roots.iter().cloned().collect();
    let mut has_indirect = false;
    let resolve_name = |name: &str| -> String {
        if all_names.contains(name) {
            return name.to_string();
        }
        let mut prefix = String::from(name);
        prefix.push_str("__");
        let mut found: Option<String> = None;
        for cand in &all_names {
            if cand.starts_with(&prefix) {
                if found.is_some() {
                    return name.to_string();
                }
                found = Some(cand.clone());
            }
        }
        found.unwrap_or_else(|| name.to_string())
    };

    while let Some(name) = stack.pop() {
        let resolved_name = resolve_name(&name);
        if !reachable.insert(resolved_name.clone()) {
            continue;
        }
        let Some(func) = map.get(&resolved_name) else {
            continue;
        };
        if let HirBody::Block(b) = &func.body {
            let mut called = BTreeSet::new();
            for line in &b.lines {
                collect_called_functions_from_expr(&line.expr, &mut called, &mut has_indirect);
            }
            for callee in called {
                let resolved_callee = resolve_name(&callee);
                if all_names.contains(&resolved_callee) && !reachable.contains(&resolved_callee) {
                    stack.push(resolved_callee);
                }
            }
        }
    }

    if has_indirect {
        return all_names;
    }
    reachable
}

fn collect_indirect_sigs(
    expr: &HirExpr,
    out: &mut Vec<(Vec<ValType>, Vec<ValType>)>,
    ctx: &TypeCtx,
) {
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
            } => {
                let mut param_types = Vec::new();
                let mut ok = true;
                for ty in params {
                    let kind = ctx.get(ctx.resolve_id(*ty));
                    if let Some(vt) = valtype(&kind) {
                        param_types.push(vt);
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    let res_kind = ctx.get(ctx.resolve_id(*result));
                    let results = if let Some(vt) = valtype(&res_kind) {
                        vec![vt]
                    } else {
                        Vec::new()
                    };
                    out.push((param_types, results));
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                stack.push(else_branch);
                stack.push(then_branch);
                stack.push(cond);
            }
            HirExprKind::While { cond, body } => {
                stack.push(body);
                stack.push(cond);
            }
            HirExprKind::Match { scrutinee, arms } => {
                for arm in arms.iter().rev() {
                    stack.push(&arm.body);
                }
                stack.push(scrutinee);
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::Block(block) => {
                for line in block.lines.iter().rev() {
                    stack.push(&line.expr);
                }
            }
            HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                stack.push(value);
            }
            HirExprKind::Intrinsic { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                stack.push(inner);
            }
            HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Var(_)
            | HirExprKind::FnValue(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
}

pub(crate) fn collect_wasm_signature_set(
    ctx: &TypeCtx,
    module: &HirModule,
) -> BTreeSet<(Vec<ValType>, Vec<ValType>)> {
    let mut out = BTreeSet::new();
    let reachable_functions = collect_reachable_wasm_functions(module);
    for ext in &module.externs {
        if let Some((params, results)) = wasm_sig_ids(ctx, ext.result, &ext.params) {
            out.insert((params, results));
        }
    }
    for f in &module.functions {
        if !reachable_functions.contains(&f.name) {
            continue;
        }
        if let Some((params, results)) = wasm_sig(ctx, f.result, &f.params) {
            out.insert((params, results));
        }
    }
    let mut indirect_sigs = Vec::new();
    for f in &module.functions {
        if let HirBody::Block(b) = &f.body {
            for line in &b.lines {
                collect_indirect_sigs(&line.expr, &mut indirect_sigs, ctx);
            }
        }
    }
    for (params, results) in indirect_sigs {
        out.insert((params, results));
    }
    out
}

pub(crate) fn is_supported_wasm_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "size_of"
            | "align_of"
            | "load"
            | "store"
            | "get_field"
            | "set_field"
            | "callsite_span"
            | "i32_to_f32"
            | "i32_to_u8"
            | "i32_to_u32"
            | "f32_to_i32"
            | "u8_to_i32"
            | "u32_to_i32"
            | "i64_to_u64"
            | "u64_to_i64"
            | "reinterpret_i32_f32"
            | "reinterpret_f32_i32"
            | "add"
            | "unreachable"
    )
}

fn parse_local<F>(text: &str, lookup_local: &mut F) -> Option<u32>
where
    F: FnMut(&str) -> Option<u32>,
{
    if let Some(stripped) = text.strip_prefix('$') {
        if let Ok(idx) = stripped.parse::<u32>() {
            Some(idx)
        } else {
            lookup_local(stripped)
        }
    } else {
        text.parse::<u32>().ok()
    }
}

pub(crate) fn parse_wasm_line_with_lookup<F>(
    line: &str,
    mut lookup_local: F,
) -> Result<Vec<Instruction<'static>>, String>
where
    F: FnMut(&str) -> Option<u32>,
{
    let mut insts = Vec::new();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(insts);
    }
    if parts[0].starts_with(";;") {
        return Ok(insts);
    }
    match parts[0] {
        "local.get" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], &mut lookup_local) {
                insts.push(Instruction::LocalGet(idx));
            } else {
                return Err(format!("unknown local in #wasm: {}", parts[1]));
            }
        }
        "local.set" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], &mut lookup_local) {
                insts.push(Instruction::LocalSet(idx));
            } else {
                return Err(format!("unknown local in #wasm: {}", parts[1]));
            }
        }
        "i32.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<i32>() {
                insts.push(Instruction::I32Const(v));
            } else {
                return Err(format!("invalid i32.const immediate: {}", parts[1]));
            }
        }
        "i32.add" => insts.push(Instruction::I32Add),
        "i32.sub" => insts.push(Instruction::I32Sub),
        "i32.mul" => insts.push(Instruction::I32Mul),
        "i32.div_s" => insts.push(Instruction::I32DivS),
        "i32.div_u" => insts.push(Instruction::I32DivU),
        "i32.rem_s" => insts.push(Instruction::I32RemS),
        "i32.rem_u" => insts.push(Instruction::I32RemU),
        "i32.and" => insts.push(Instruction::I32And),
        "i32.or" => insts.push(Instruction::I32Or),
        "i32.xor" => insts.push(Instruction::I32Xor),
        "i32.shl" => insts.push(Instruction::I32Shl),
        "i32.shr_s" => insts.push(Instruction::I32ShrS),
        "i32.shr_u" => insts.push(Instruction::I32ShrU),
        "i32.rotl" => insts.push(Instruction::I32Rotl),
        "i32.rotr" => insts.push(Instruction::I32Rotr),
        "i32.clz" => insts.push(Instruction::I32Clz),
        "i32.ctz" => insts.push(Instruction::I32Ctz),
        "i32.popcnt" => insts.push(Instruction::I32Popcnt),
        "i32.eqz" => insts.push(Instruction::I32Eqz),
        "i32.eq" => insts.push(Instruction::I32Eq),
        "i32.ne" => insts.push(Instruction::I32Ne),
        "i32.lt_s" => insts.push(Instruction::I32LtS),
        "i32.lt_u" => insts.push(Instruction::I32LtU),
        "i32.le_s" => insts.push(Instruction::I32LeS),
        "i32.le_u" => insts.push(Instruction::I32LeU),
        "i32.gt_s" => insts.push(Instruction::I32GtS),
        "i32.gt_u" => insts.push(Instruction::I32GtU),
        "i32.ge_s" => insts.push(Instruction::I32GeS),
        "i32.ge_u" => insts.push(Instruction::I32GeU),
        "i32.load" => insts.push(Instruction::I32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i32.store" => insts.push(Instruction::I32Store(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i32.load8_s" => insts.push(Instruction::I32Load8S(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.load8_u" => insts.push(Instruction::I32Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.load16_s" => insts.push(Instruction::I32Load16S(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.load16_u" => insts.push(Instruction::I32Load16U(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.store8" => insts.push(Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.store16" => insts.push(Instruction::I32Store16(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.extend8_s" => insts.push(Instruction::I32Extend8S),
        "i32.extend16_s" => insts.push(Instruction::I32Extend16S),
        "i64.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<i64>() {
                insts.push(Instruction::I64Const(v));
            } else {
                return Err(format!("invalid i64.const immediate: {}", parts[1]));
            }
        }
        "i64.add" => insts.push(Instruction::I64Add),
        "i64.sub" => insts.push(Instruction::I64Sub),
        "i64.mul" => insts.push(Instruction::I64Mul),
        "i64.div_s" => insts.push(Instruction::I64DivS),
        "i64.div_u" => insts.push(Instruction::I64DivU),
        "i64.rem_s" => insts.push(Instruction::I64RemS),
        "i64.rem_u" => insts.push(Instruction::I64RemU),
        "i64.and" => insts.push(Instruction::I64And),
        "i64.or" => insts.push(Instruction::I64Or),
        "i64.xor" => insts.push(Instruction::I64Xor),
        "i64.shl" => insts.push(Instruction::I64Shl),
        "i64.shr_s" => insts.push(Instruction::I64ShrS),
        "i64.shr_u" => insts.push(Instruction::I64ShrU),
        "i64.rotl" => insts.push(Instruction::I64Rotl),
        "i64.rotr" => insts.push(Instruction::I64Rotr),
        "i64.clz" => insts.push(Instruction::I64Clz),
        "i64.ctz" => insts.push(Instruction::I64Ctz),
        "i64.popcnt" => insts.push(Instruction::I64Popcnt),
        "i64.eqz" => insts.push(Instruction::I64Eqz),
        "i64.eq" => insts.push(Instruction::I64Eq),
        "i64.ne" => insts.push(Instruction::I64Ne),
        "i64.lt_s" => insts.push(Instruction::I64LtS),
        "i64.lt_u" => insts.push(Instruction::I64LtU),
        "i64.le_s" => insts.push(Instruction::I64LeS),
        "i64.le_u" => insts.push(Instruction::I64LeU),
        "i64.gt_s" => insts.push(Instruction::I64GtS),
        "i64.gt_u" => insts.push(Instruction::I64GtU),
        "i64.ge_s" => insts.push(Instruction::I64GeS),
        "i64.ge_u" => insts.push(Instruction::I64GeU),
        "i64.load" => insts.push(Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "i64.store" => insts.push(Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "i64.load8_s" => insts.push(Instruction::I64Load8S(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.load8_u" => insts.push(Instruction::I64Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.load16_s" => insts.push(Instruction::I64Load16S(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.load16_u" => insts.push(Instruction::I64Load16U(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.load32_s" => insts.push(Instruction::I64Load32S(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i64.load32_u" => insts.push(Instruction::I64Load32U(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i64.store8" => insts.push(Instruction::I64Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.store16" => insts.push(Instruction::I64Store16(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.store32" => insts.push(Instruction::I64Store32(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "f32.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<f32>() {
                insts.push(Instruction::F32Const(v.into()));
            } else {
                return Err(format!("invalid f32.const immediate: {}", parts[1]));
            }
        }
        "f32.add" => insts.push(Instruction::F32Add),
        "f32.sub" => insts.push(Instruction::F32Sub),
        "f32.mul" => insts.push(Instruction::F32Mul),
        "f32.div" => insts.push(Instruction::F32Div),
        "f32.abs" => insts.push(Instruction::F32Abs),
        "f32.neg" => insts.push(Instruction::F32Neg),
        "f32.ceil" => insts.push(Instruction::F32Ceil),
        "f32.floor" => insts.push(Instruction::F32Floor),
        "f32.trunc" => insts.push(Instruction::F32Trunc),
        "f32.nearest" => insts.push(Instruction::F32Nearest),
        "f32.sqrt" => insts.push(Instruction::F32Sqrt),
        "f32.min" => insts.push(Instruction::F32Min),
        "f32.max" => insts.push(Instruction::F32Max),
        "f32.copysign" => insts.push(Instruction::F32Copysign),
        "f32.eq" => insts.push(Instruction::F32Eq),
        "f32.ne" => insts.push(Instruction::F32Ne),
        "f32.lt" => insts.push(Instruction::F32Lt),
        "f32.le" => insts.push(Instruction::F32Le),
        "f32.gt" => insts.push(Instruction::F32Gt),
        "f32.ge" => insts.push(Instruction::F32Ge),
        "f32.load" => insts.push(Instruction::F32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "f32.store" => insts.push(Instruction::F32Store(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "f64.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<f64>() {
                insts.push(Instruction::F64Const(v.into()));
            } else {
                return Err(format!("invalid f64.const immediate: {}", parts[1]));
            }
        }
        "f64.add" => insts.push(Instruction::F64Add),
        "f64.sub" => insts.push(Instruction::F64Sub),
        "f64.mul" => insts.push(Instruction::F64Mul),
        "f64.div" => insts.push(Instruction::F64Div),
        "f64.abs" => insts.push(Instruction::F64Abs),
        "f64.neg" => insts.push(Instruction::F64Neg),
        "f64.ceil" => insts.push(Instruction::F64Ceil),
        "f64.floor" => insts.push(Instruction::F64Floor),
        "f64.trunc" => insts.push(Instruction::F64Trunc),
        "f64.nearest" => insts.push(Instruction::F64Nearest),
        "f64.sqrt" => insts.push(Instruction::F64Sqrt),
        "f64.min" => insts.push(Instruction::F64Min),
        "f64.max" => insts.push(Instruction::F64Max),
        "f64.copysign" => insts.push(Instruction::F64Copysign),
        "f64.eq" => insts.push(Instruction::F64Eq),
        "f64.ne" => insts.push(Instruction::F64Ne),
        "f64.lt" => insts.push(Instruction::F64Lt),
        "f64.le" => insts.push(Instruction::F64Le),
        "f64.gt" => insts.push(Instruction::F64Gt),
        "f64.ge" => insts.push(Instruction::F64Ge),
        "f64.load" => insts.push(Instruction::F64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "f64.store" => insts.push(Instruction::F64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "i32.wrap_i64" => insts.push(Instruction::I32WrapI64),
        "i64.extend_i32_s" => insts.push(Instruction::I64ExtendI32S),
        "i64.extend_i32_u" => insts.push(Instruction::I64ExtendI32U),
        "i32.trunc_f32_s" => insts.push(Instruction::I32TruncF32S),
        "i32.trunc_f32_u" => insts.push(Instruction::I32TruncF32U),
        "i32.trunc_f64_s" => insts.push(Instruction::I32TruncF64S),
        "i32.trunc_f64_u" => insts.push(Instruction::I32TruncF64U),
        "i64.trunc_f32_s" => insts.push(Instruction::I64TruncF32S),
        "i64.trunc_f32_u" => insts.push(Instruction::I64TruncF32U),
        "i64.trunc_f64_s" => insts.push(Instruction::I64TruncF64S),
        "i64.trunc_f64_u" => insts.push(Instruction::I64TruncF64U),
        "f32.convert_i32_s" => insts.push(Instruction::F32ConvertI32S),
        "f32.convert_i32_u" => insts.push(Instruction::F32ConvertI32U),
        "f32.convert_i64_s" => insts.push(Instruction::F32ConvertI64S),
        "f32.convert_i64_u" => insts.push(Instruction::F32ConvertI64U),
        "f32.demote_f64" => insts.push(Instruction::F32DemoteF64),
        "f64.convert_i32_s" => insts.push(Instruction::F64ConvertI32S),
        "f64.convert_i32_u" => insts.push(Instruction::F64ConvertI32U),
        "f64.convert_i64_s" => insts.push(Instruction::F64ConvertI64S),
        "f64.convert_i64_u" => insts.push(Instruction::F64ConvertI64U),
        "f64.promote_f32" => insts.push(Instruction::F64PromoteF32),
        "i32.reinterpret_f32" => insts.push(Instruction::I32ReinterpretF32),
        "i64.reinterpret_f64" => insts.push(Instruction::I64ReinterpretF64),
        "f32.reinterpret_i32" => insts.push(Instruction::F32ReinterpretI32),
        "f64.reinterpret_i64" => insts.push(Instruction::F64ReinterpretI64),
        "i32.trunc_sat_f32_s" => insts.push(Instruction::I32TruncSatF32S),
        "i32.trunc_sat_f32_u" => insts.push(Instruction::I32TruncSatF32U),
        "i32.trunc_sat_f64_s" => insts.push(Instruction::I32TruncSatF64S),
        "i32.trunc_sat_f64_u" => insts.push(Instruction::I32TruncSatF64U),
        "i64.trunc_sat_f32_s" => insts.push(Instruction::I64TruncSatF32S),
        "i64.trunc_sat_f32_u" => insts.push(Instruction::I64TruncSatF32U),
        "i64.trunc_sat_f64_s" => insts.push(Instruction::I64TruncSatF64S),
        "i64.trunc_sat_f64_u" => insts.push(Instruction::I64TruncSatF64U),
        "memory.grow" => insts.push(Instruction::MemoryGrow(0)),
        "memory.size" => insts.push(Instruction::MemorySize(0)),
        "memory.copy" => insts.push(Instruction::MemoryCopy {
            dst_mem: 0,
            src_mem: 0,
        }),
        "drop" => insts.push(Instruction::Drop),
        other => return Err(format!("unsupported wasm instruction: {}", other)),
    }
    Ok(insts)
}

pub(crate) fn precheck_raw_wasm_body(_ctx: &TypeCtx, func: &HirFunction) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    if let HirBody::Wasm(wb) = &func.body {
        let mut param_map = BTreeMap::new();
        for (idx, p) in func.params.iter().enumerate() {
            param_map.insert(p.name.clone(), idx as u32);
        }
        for line in &wb.lines {
            let parsed = parse_wasm_line_with_lookup(line, |name| param_map.get(name).copied());
            if let Err(msg) = parsed {
                out.push(
                    Diagnostic::error(msg, func.span)
                        .with_id(DiagnosticId::CodegenWasmRawLineParseError),
                );
            }
        }
    }
    out
}

//! WASM backend for NEPLG2.

#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

use crate::builtins::BuiltinKind;
use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::types::{TypeCtx, TypeId, TypeKind};
use alloc::string::ToString;

#[derive(Debug)]
pub struct CodegenResult {
    pub bytes: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn generate_wasm(ctx: &TypeCtx, module: &HirModule) -> CodegenResult {
    let mut diags = Vec::new();

    // Build function list (builtins first)
    let mut functions: Vec<FuncLower> = Vec::new();

    // Builtins
    functions.push(FuncLower::builtin(
        "add",
        BuiltinKind::AddI32,
        vec![ValType::I32, ValType::I32],
        Some(ValType::I32),
    ));
    functions.push(FuncLower::builtin(
        "sub",
        BuiltinKind::SubI32,
        vec![ValType::I32, ValType::I32],
        Some(ValType::I32),
    ));
    functions.push(FuncLower::builtin(
        "lt",
        BuiltinKind::LtI32,
        vec![ValType::I32, ValType::I32],
        Some(ValType::I32),
    ));
    functions.push(FuncLower::builtin(
        "print_i32",
        BuiltinKind::PrintI32,
        vec![ValType::I32],
        None,
    ));

    // User functions
    for f in &module.functions {
        if let Some(sig) = wasm_sig(ctx, f.result, &f.params) {
            functions.push(FuncLower::user(f.clone(), sig));
        } else {
            diags.push(Diagnostic::error(
                "unsupported function signature for wasm",
                f.span,
            ));
        }
    }

    // Map names to indices
    let mut name_to_index = BTreeMap::new();
    for (idx, f) in functions.iter().enumerate() {
        name_to_index.insert(f.name.clone(), idx as u32);
    }

    // Type section dedup
    let mut type_section = TypeSection::new();
    let mut sig_map: BTreeMap<(Vec<ValType>, Vec<ValType>), u32> = BTreeMap::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        if !sig_map.contains_key(&key) {
            let idx = type_section.len();
            type_section.ty().function(f.params.clone(), f.results.clone());
            sig_map.insert(key.clone(), idx);
        }
    }

    let mut func_section = FunctionSection::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        let type_idx = *sig_map.get(&key).unwrap();
        func_section.function(type_idx);
    }

    let mut code_section = CodeSection::new();
    for f in &functions {
        match lower_body(ctx, f, &name_to_index) {
            Ok(body) => {
                code_section.function(&body);
            }
            Err(mut ds) => {
                diags.append(&mut ds);
            }
        }
    }

    let mut export_section = ExportSection::new();
    if let Some(entry) = &module.entry {
        if let Some(idx) = name_to_index.get(entry) {
            export_section.export("main", ExportKind::Func, *idx);
            export_section.export(entry, ExportKind::Func, *idx);
        }
    }

    if diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return CodegenResult {
            bytes: None,
            diagnostics: diags,
        };
    }

    let mut module_bytes = Module::new();
    module_bytes.section(&type_section);
    module_bytes.section(&func_section);
    module_bytes.section(&export_section);
    module_bytes.section(&code_section);

    CodegenResult {
        bytes: Some(module_bytes.finish()),
        diagnostics: diags,
    }
}

// ---------------------------------------------------------------------
// Function lowering
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FuncLower {
    name: String,
    params: Vec<ValType>,
    results: Vec<ValType>,
    body: FuncBodyLower,
}

#[derive(Debug, Clone)]
enum FuncBodyLower {
    Builtin(BuiltinKind),
    User(HirFunction),
}

impl FuncLower {
    fn builtin(name: &str, kind: BuiltinKind, params: Vec<ValType>, result: Option<ValType>) -> Self {
        Self {
            name: name.to_string(),
            params,
            results: result.into_iter().collect(),
            body: FuncBodyLower::Builtin(kind),
        }
    }

    fn user(func: HirFunction, sig: (Vec<ValType>, Vec<ValType>)) -> Self {
        Self {
            name: func.name.clone(),
            params: sig.0,
            results: sig.1,
            body: FuncBodyLower::User(func),
        }
    }
}

fn wasm_sig(ctx: &TypeCtx, result: TypeId, params: &[HirParam]) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(p.ty);
        if let Some(v) = valtype(&vk) {
            param_types.push(v);
        } else {
            return None;
        }
    }
    let res_kind = ctx.get(result);
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        Vec::new()
    };
    Some((param_types, res))
}

fn valtype(kind: &TypeKind) -> Option<ValType> {
    match kind {
        TypeKind::Unit => None,
        TypeKind::I32 | TypeKind::Bool => Some(ValType::I32),
        TypeKind::F32 => Some(ValType::F32),
        _ => None,
    }
}

fn lower_body(
    ctx: &TypeCtx,
    func: &FuncLower,
    name_map: &BTreeMap<String, u32>,
) -> Result<Function, Vec<Diagnostic>> {
    match &func.body {
        FuncBodyLower::Builtin(kind) => Ok(lower_builtin(kind)),
        FuncBodyLower::User(f) => lower_user(ctx, f, name_map),
    }
}

fn lower_builtin(kind: &BuiltinKind) -> Function {
    let mut func = Function::new(Vec::new());
    match kind {
        BuiltinKind::AddI32 => {
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::I32Add);
        }
        BuiltinKind::SubI32 => {
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::I32Sub);
        }
        BuiltinKind::LtI32 => {
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::LocalGet(1));
            func.instruction(&Instruction::I32LtS);
        }
        BuiltinKind::PrintI32 => {
            // no-op
        }
    }
    func.instruction(&Instruction::End);
    func
}

// ---------------------------------------------------------------------
// User function lowering
// ---------------------------------------------------------------------

fn lower_user(
    ctx: &TypeCtx,
    func: &HirFunction,
    name_map: &BTreeMap<String, u32>,
) -> Result<Function, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let mut locals = LocalMap::new(func.params.len());
    for p in &func.params {
        locals.register_param(p.name.clone(), p.ty);
    }

    let mut insts: Vec<Instruction<'static>> = Vec::new();

    match &func.body {
        HirBody::Block(block) => {
            let produced = gen_block(ctx, block, name_map, &mut locals, &mut insts, &mut diags);
            let expected = valtype(&ctx.get(func.result));
            if expected.is_some() && produced.flatten().is_none() {
                diags.push(Diagnostic::error(
                    "function expected to return value",
                    func.span,
                ));
            }
        }
        HirBody::Wasm(wb) => {
            for line in &wb.lines {
                for inst in parse_wasm_line(line, &locals) {
                    insts.push(inst);
                }
            }
        }
    }

    let mut wasm_func = Function::new(locals.local_decls());
    for inst in insts {
        wasm_func.instruction(&inst);
    }
    wasm_func.instruction(&Instruction::End);
    if diags.is_empty() {
        Ok(wasm_func)
    } else {
        Err(diags)
    }
}

fn gen_block(
    ctx: &TypeCtx,
    block: &HirBlock,
    name_map: &BTreeMap<String, u32>,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
    diags: &mut Vec<Diagnostic>,
) -> Option<Option<ValType>> {
    let mut last_val: Option<ValType> = None;
    for line in &block.lines {
        let val = gen_expr(ctx, &line.expr, name_map, locals, insts, diags);
        if line.drop_result {
            if val.is_some() {
                insts.push(Instruction::Drop);
            }
            last_val = None;
        } else {
            last_val = val;
        }
    }
    Some(last_val)
}

fn gen_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
    diags: &mut Vec<Diagnostic>,
) -> Option<ValType> {
    match &expr.kind {
        HirExprKind::LiteralI32(v) => {
            insts.push(Instruction::I32Const(*v));
            Some(ValType::I32)
        }
        HirExprKind::LiteralF32(v) => {
            insts.push(Instruction::F32Const((*v).into()));
            Some(ValType::F32)
        }
        HirExprKind::LiteralBool(b) => {
            insts.push(Instruction::I32Const(if *b { 1 } else { 0 }));
            Some(ValType::I32)
        }
        HirExprKind::Unit => None,
        HirExprKind::Var(name) => {
            if let Some(idx) = locals.lookup(name) {
                insts.push(Instruction::LocalGet(idx));
                valtype(&ctx.get(expr.ty))
            } else if let Some(fidx) = name_map.get(name) {
                insts.push(Instruction::Call(*fidx));
                valtype(&ctx.get(expr.ty))
            } else {
                diags.push(Diagnostic::error("unknown variable", expr.span));
                None
            }
        }
        HirExprKind::Call { callee, args } => {
            for arg in args {
                gen_expr(ctx, arg, name_map, locals, insts, diags);
            }
            if let Some(idx) = match callee {
                FuncRef::Builtin(n) | FuncRef::User(n) => name_map.get(n),
            } {
                insts.push(Instruction::Call(*idx));
            } else {
                diags.push(Diagnostic::error("unknown function", expr.span));
            }
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::If { cond, then_branch, else_branch } => {
            gen_expr(ctx, cond, name_map, locals, insts, diags);
            let result_ty = valtype(&ctx.get(expr.ty));
            match result_ty {
                Some(vt) => insts.push(Instruction::If(wasm_encoder::BlockType::Result(vt))),
                None => insts.push(Instruction::If(wasm_encoder::BlockType::Empty)),
            }
            gen_expr(ctx, then_branch, name_map, locals, insts, diags);
            insts.push(Instruction::Else);
            gen_expr(ctx, else_branch, name_map, locals, insts, diags);
            insts.push(Instruction::End);
            result_ty
        }
        HirExprKind::While { cond, body } => {
            insts.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
            gen_expr(ctx, cond, name_map, locals, insts, diags);
            insts.push(Instruction::I32Eqz);
            insts.push(Instruction::BrIf(1)); // exit loop
            gen_expr(ctx, body, name_map, locals, insts, diags);
            insts.push(Instruction::Br(0));
            insts.push(Instruction::End);
            None
        }
        HirExprKind::Block(b) => gen_block(ctx, b, name_map, locals, insts, diags).flatten(),
        HirExprKind::Let { name, value, .. } => {
            let idx = locals.ensure_local(name.clone(), value.ty, ctx);
            gen_expr(ctx, value, name_map, locals, insts, diags);
            insts.push(Instruction::LocalSet(idx));
            None
        }
        HirExprKind::Set { name, value } => {
            if let Some(idx) = locals.lookup(name) {
                gen_expr(ctx, value, name_map, locals, insts, diags);
                insts.push(Instruction::LocalSet(idx));
            } else {
                diags.push(Diagnostic::error("unknown variable", expr.span));
            }
            None
        }
    }
}

// ---------------------------------------------------------------------
// Locals
// ---------------------------------------------------------------------

#[derive(Debug)]
struct LocalInfo {
    name: String,
    idx: u32,
    ty: TypeId,
    is_param: bool,
}

#[derive(Debug)]
struct LocalMap {
    locals: Vec<LocalInfo>,
    next_idx: u32,
    decls: Vec<ValType>,
}

impl LocalMap {
    fn new(param_count: usize) -> Self {
        Self {
            locals: Vec::new(),
            next_idx: param_count as u32,
            decls: Vec::new(),
        }
    }

    fn register_param(&mut self, name: String, ty: TypeId) {
        let idx = self.locals.len() as u32;
        self.locals.push(LocalInfo {
            name,
            idx,
            ty,
            is_param: true,
        });
    }

    fn ensure_local(&mut self, name: String, ty: TypeId, ctx: &TypeCtx) -> u32 {
        if let Some(idx) = self.lookup(&name) {
            idx
        } else {
            let idx = self.next_idx;
            self.next_idx += 1;
            self.locals.push(LocalInfo {
                name,
                idx,
                ty,
                is_param: false,
            });
            if let Some(vt) = valtype(&ctx.get(ty)) {
                self.decls.push(vt);
            }
            idx
        }
    }

    fn lookup(&self, name: &str) -> Option<u32> {
        for l in &self.locals {
            if l.name == name {
                return Some(l.idx);
            }
        }
        None
    }

    fn local_decls(&self) -> Vec<(u32, ValType)> {
        self.decls.iter().map(|v| (1u32, *v)).collect()
    }
}

// ---------------------------------------------------------------------
// Minimal wasm text parser for #wasm blocks
// ---------------------------------------------------------------------

fn parse_wasm_line(line: &str, locals: &LocalMap) -> Vec<Instruction<'static>> {
    let mut insts = Vec::new();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return insts;
    }
    match parts[0] {
        "local.get" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
                insts.push(Instruction::LocalGet(idx));
            }
        }
        "local.set" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
                insts.push(Instruction::LocalSet(idx));
            }
        }
        "i32.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<i32>() {
                insts.push(Instruction::I32Const(v));
            }
        }
        "i32.add" => insts.push(Instruction::I32Add),
        "i32.sub" => insts.push(Instruction::I32Sub),
        "i32.lt_s" => insts.push(Instruction::I32LtS),
        _ => {}
    }
    insts
}

fn parse_local(text: &str, locals: &LocalMap) -> Option<u32> {
    if let Some(stripped) = text.strip_prefix('$') {
        if let Ok(idx) = stripped.parse::<u32>() {
            Some(idx)
        } else {
            locals.lookup(stripped)
        }
    } else {
        text.parse::<u32>().ok()
    }
}

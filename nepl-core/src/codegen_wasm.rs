//! WASM backend for NEPLG2.

#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};

use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug)]
pub struct CodegenResult {
    pub bytes: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct StringLower {
    offsets: Vec<u32>,
    segments: Vec<(u32, Vec<u8>)>,
    min_pages: u32,
}

impl StringLower {
    fn offset(&self, idx: u32) -> Option<u32> {
        self.offsets.get(idx as usize).copied()
    }

    fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
}

fn lower_strings(strings: &[String]) -> StringLower {
    let mut offsets = Vec::new();
    let mut segments = Vec::new();
    let mut cursor: u32 = 0;
    for s in strings {
        cursor = align_to(cursor, 4);
        offsets.push(cursor);
        let mut data = Vec::new();
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(bytes);
        segments.push((cursor, data));
        cursor = cursor.saturating_add(4 + len);
    }
    let min_pages = ((cursor + 0xFFFF) / 0x10000).max(1);
    StringLower {
        offsets,
        segments,
        min_pages,
    }
}

fn align_to(x: u32, align: u32) -> u32 {
    let mask = align - 1;
    (x + mask) & !mask
}

pub fn generate_wasm(ctx: &TypeCtx, module: &HirModule) -> CodegenResult {
    let mut diags = Vec::new();
    let strings = lower_strings(&module.string_literals);

    // Build imports / function list (builtins first)
    let mut imports: Vec<ImportLower> = Vec::new();
    let mut functions: Vec<FuncLower> = Vec::new();

    // Extern imports
    for ext in &module.externs {
        if let Some(sig) = wasm_sig_ids(ctx, ext.result, &ext.params) {
            imports.push(ImportLower::function(
                ext.module.clone(),
                ext.name.clone(),
                ext.local_name.clone(),
                sig.0,
                sig.1,
            ));
        } else {
            diags.push(Diagnostic::error(
                "unsupported extern signature for wasm",
                ext.span,
            ));
        }
    }

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
    let mut next_index: u32 = 0;
    for imp in &imports {
        name_to_index.insert(imp.name.clone(), next_index);
        next_index += 1;
    }
    for (idx, f) in functions.iter().enumerate() {
        name_to_index.insert(f.name.clone(), next_index + idx as u32);
    }

    // Type section dedup
    let mut type_section = TypeSection::new();
    let mut sig_map: BTreeMap<(Vec<ValType>, Vec<ValType>), u32> = BTreeMap::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        if !sig_map.contains_key(&key) {
            let idx = type_section.len();
            type_section
                .ty()
                .function(f.params.clone(), f.results.clone());
            sig_map.insert(key.clone(), idx);
        }
    }
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        if !sig_map.contains_key(&key) {
            let idx = type_section.len();
            type_section
                .ty()
                .function(imp.params.clone(), imp.results.clone());
            sig_map.insert(key.clone(), idx);
        }
    }

    let mut import_section = ImportSection::new();
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        let type_idx = *sig_map.get(&key).unwrap();
        import_section.import(&imp.module, &imp.field, EntityType::Function(type_idx));
    }

    let mut func_section = FunctionSection::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        let type_idx = *sig_map.get(&key).unwrap();
        func_section.function(type_idx);
    }

    let mut code_section = CodeSection::new();
    for f in &functions {
        match lower_body(ctx, f, &name_to_index, &strings) {
            Ok(body) => {
                code_section.function(&body);
            }
            Err(mut ds) => {
                diags.append(&mut ds);
            }
        }
    }

    let mut memory_section = MemorySection::new();
    memory_section.memory(MemoryType {
        minimum: strings.min_pages as u64,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });

    let mut export_section = ExportSection::new();
    export_section.export("memory", ExportKind::Memory, 0);
    if let Some(entry) = &module.entry {
        if let Some(idx) = name_to_index.get(entry) {
            export_section.export("main", ExportKind::Func, *idx);
            if entry != "main" {
                export_section.export(entry, ExportKind::Func, *idx);
            }
        }
    }

    let mut data_section = DataSection::new();
    for (offset, bytes) in &strings.segments {
        data_section.active(0, &ConstExpr::i32_const(*offset as i32), bytes.clone());
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
    if !imports.is_empty() {
        module_bytes.section(&import_section);
    }
    module_bytes.section(&func_section);
    module_bytes.section(&memory_section);
    module_bytes.section(&export_section);
    module_bytes.section(&code_section);
    if !strings.segments.is_empty() {
        module_bytes.section(&data_section);
    }

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
struct ImportLower {
    module: String,
    field: String,
    name: String,
    params: Vec<ValType>,
    results: Vec<ValType>,
}

#[derive(Debug, Clone)]
enum FuncBodyLower {
    User(HirFunction),
}

impl FuncLower {
    fn user(func: HirFunction, sig: (Vec<ValType>, Vec<ValType>)) -> Self {
        Self {
            name: func.name.clone(),
            params: sig.0,
            results: sig.1,
            body: FuncBodyLower::User(func),
        }
    }
}

impl ImportLower {
    fn function(
        module: String,
        field: String,
        local_name: String,
        params: Vec<ValType>,
        results: Vec<ValType>,
    ) -> Self {
        Self {
            module,
            field,
            name: local_name,
            params,
            results,
        }
    }
}

fn wasm_sig(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[HirParam],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
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

fn wasm_sig_ids(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[TypeId],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(*p);
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
        TypeKind::I32 | TypeKind::Bool | TypeKind::Str => Some(ValType::I32),
        TypeKind::F32 => Some(ValType::F32),
        _ => None,
    }
}

fn lower_body(
    ctx: &TypeCtx,
    func: &FuncLower,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
) -> Result<Function, Vec<Diagnostic>> {
    match &func.body {
        FuncBodyLower::User(f) => lower_user(ctx, f, name_map, strings),
    }
}

// ---------------------------------------------------------------------
// User function lowering
// ---------------------------------------------------------------------

fn lower_user(
    ctx: &TypeCtx,
    func: &HirFunction,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
) -> Result<Function, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let mut locals = LocalMap::new(func.params.len());
    for p in &func.params {
        locals.register_param(p.name.clone(), p.ty);
    }

    let mut insts: Vec<Instruction<'static>> = Vec::new();

    match &func.body {
        HirBody::Block(block) => {
            let produced = gen_block(
                ctx,
                block,
                name_map,
                strings,
                &mut locals,
                &mut insts,
                &mut diags,
            );
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
                match parse_wasm_line(line, &locals) {
                    Ok(mut v) => insts.append(&mut v),
                    Err(msg) => diags.push(Diagnostic::error(msg, func.span)),
                }
            }
            if diags.is_empty() {
                if let Err(d) = validate_wasm_stack(ctx, func, &locals, &insts) {
                    diags.push(d);
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
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
    diags: &mut Vec<Diagnostic>,
) -> Option<Option<ValType>> {
    let mut last_val: Option<ValType> = None;
    for line in &block.lines {
        let val = gen_expr(ctx, &line.expr, name_map, strings, locals, insts, diags);
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
    strings: &StringLower,
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
        HirExprKind::LiteralStr(id) => {
            if let Some(off) = strings.offset(*id) {
                insts.push(Instruction::I32Const(off as i32));
                Some(ValType::I32)
            } else {
                diags.push(Diagnostic::error(
                    "string literal not found during codegen",
                    expr.span,
                ));
                None
            }
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
                diags.push(Diagnostic::error(
                    format!("unknown variable {}", name),
                    expr.span,
                ));
                None
            }
        }
        HirExprKind::Call { callee, args } => {
            for arg in args {
                gen_expr(ctx, arg, name_map, strings, locals, insts, diags);
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
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            gen_expr(ctx, cond, name_map, strings, locals, insts, diags);
            let result_ty = valtype(&ctx.get(expr.ty));
            match result_ty {
                Some(vt) => insts.push(Instruction::If(wasm_encoder::BlockType::Result(vt))),
                None => insts.push(Instruction::If(wasm_encoder::BlockType::Empty)),
            }
            gen_expr(ctx, then_branch, name_map, strings, locals, insts, diags);
            insts.push(Instruction::Else);
            gen_expr(ctx, else_branch, name_map, strings, locals, insts, diags);
            insts.push(Instruction::End);
            result_ty
        }
        HirExprKind::While { cond, body } => {
            // while cond body:
            // block  ;; break target depth=1
            //   loop ;; continue target depth=0
            //     cond
            //     i32.eqz
            //     br_if 1  ;; break
            //     body
            //     br 0     ;; continue
            //   end
            // end
            insts.push(Instruction::Block(wasm_encoder::BlockType::Empty));
            insts.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
            gen_expr(ctx, cond, name_map, strings, locals, insts, diags);
            insts.push(Instruction::I32Eqz);
            insts.push(Instruction::BrIf(1));
            gen_expr(ctx, body, name_map, strings, locals, insts, diags);
            insts.push(Instruction::Br(0));
            insts.push(Instruction::End);
            insts.push(Instruction::End);
            None
        }
        HirExprKind::Block(b) => {
            gen_block(ctx, b, name_map, strings, locals, insts, diags).flatten()
        }
        HirExprKind::Let { name, value, .. } => {
            let idx = locals.ensure_local(name.clone(), value.ty, ctx);
            gen_expr(ctx, value, name_map, strings, locals, insts, diags);
            insts.push(Instruction::LocalSet(idx));
            None
        }
        HirExprKind::Set { name, value } => {
            if let Some(idx) = locals.lookup(name) {
                gen_expr(ctx, value, name_map, strings, locals, insts, diags);
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

    fn valtype_of(&self, idx: u32, ctx: &TypeCtx) -> Option<ValType> {
        self.locals
            .iter()
            .find(|l| l.idx == idx)
            .and_then(|l| valtype(&ctx.get(l.ty)))
    }
}

// ---------------------------------------------------------------------
// Minimal wasm text parser for #wasm blocks
// ---------------------------------------------------------------------

fn parse_wasm_line(line: &str, locals: &LocalMap) -> Result<Vec<Instruction<'static>>, String> {
    let mut insts = Vec::new();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(insts);
    }
    match parts[0] {
        "local.get" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
                insts.push(Instruction::LocalGet(idx));
            } else {
                return Err(format!("unknown local in #wasm: {}", parts[1]));
            }
        }
        "local.set" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
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
        "i32.lt_s" => insts.push(Instruction::I32LtS),
        other => return Err(format!("unsupported wasm instruction: {}", other)),
    }
    Ok(insts)
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

fn validate_wasm_stack(
    ctx: &TypeCtx,
    func: &HirFunction,
    locals: &LocalMap,
    insts: &[Instruction<'static>],
) -> Result<(), Diagnostic> {
    let mut stack: Vec<ValType> = Vec::new();
    for inst in insts {
        match inst {
            Instruction::LocalGet(idx) => {
                if let Some(vt) = locals.valtype_of(*idx, ctx) {
                    stack.push(vt);
                } else {
                    return Err(Diagnostic::error("unknown local in #wasm", func.span));
                }
            }
            Instruction::LocalSet(idx) => {
                let expected = locals
                    .valtype_of(*idx, ctx)
                    .ok_or_else(|| Diagnostic::error("unknown local in #wasm", func.span))?;
                match stack.pop() {
                    Some(top) if top == expected => {}
                    Some(_) => {
                        return Err(Diagnostic::error(
                            "type mismatch for local.set in #wasm",
                            func.span,
                        ));
                    }
                    None => {
                        return Err(Diagnostic::error(
                            "stack underflow in #wasm local.set",
                            func.span,
                        ));
                    }
                }
            }
            Instruction::I32Const(_) => stack.push(ValType::I32),
            Instruction::I32Add | Instruction::I32Sub | Instruction::I32LtS => {
                let a = stack.pop();
                let b = stack.pop();
                if a == Some(ValType::I32) && b == Some(ValType::I32) {
                    stack.push(ValType::I32);
                } else {
                    return Err(Diagnostic::error(
                        "i32 arithmetic expects two i32 values on stack",
                        func.span,
                    ));
                }
            }
            other => {
                return Err(Diagnostic::error(
                    alloc::format!("unsupported wasm instruction in #wasm: {:?}", other),
                    func.span,
                ));
            }
        }
    }

    let expected = valtype(&ctx.get(func.result));
    match expected {
        Some(vt) => {
            if stack.len() == 1 && stack[0] == vt {
                Ok(())
            } else {
                Err(Diagnostic::error(
                    "wasm body result does not match function signature",
                    func.span,
                ))
            }
        }
        None => {
            if stack.is_empty() {
                Ok(())
            } else {
                Err(Diagnostic::error(
                    "wasm body leaves values on stack for unit return",
                    func.span,
                ))
            }
        }
    }
}

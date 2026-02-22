//! LLVM IR 生成（core 側）
//!
//! このモジュールは AST から LLVM IR テキストを生成する責務のみを持つ。
//! clang 実行などのホスト依存処理は `nepl-cli` 側で扱う。

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::{Block, FnBody, Ident, Literal, Module, PrefixExpr, PrefixItem, Stmt, TypeExpr};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::ast::Directive;
use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirModule};
use crate::types::{TypeCtx, TypeId, TypeKind};

/// LLVM IR 生成時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmCodegenError {
    MissingLlvmIrBlock,
    UnsupportedParsedFunctionBody { function: String },
    UnsupportedWasmBody { function: String },
    ConflictingRawBodies { function: String },
    TypecheckFailed { reason: String },
    MissingEntryFunction { function: String },
    UnsupportedHirLowering { function: String, reason: String },
}

impl core::fmt::Display for LlvmCodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LlvmCodegenError::MissingLlvmIrBlock => {
                write!(
                    f,
                    "llvm target requires at least one #llvmir block in module/function body"
                )
            }
            LlvmCodegenError::UnsupportedParsedFunctionBody { function } => write!(
                f,
                "llvm target currently supports only subset lowering for parsed functions; function '{}' is not in supported subset",
                function
            ),
            LlvmCodegenError::UnsupportedWasmBody { function } => write!(
                f,
                "llvm target cannot lower #wasm function body; function '{}'",
                function
            ),
            LlvmCodegenError::ConflictingRawBodies { function } => write!(
                f,
                "function '{}' has multiple active raw bodies after #if gate evaluation",
                function
            ),
            LlvmCodegenError::TypecheckFailed { reason } => {
                write!(f, "failed to typecheck module for llvm lowering: {}", reason)
            }
            LlvmCodegenError::MissingEntryFunction { function } => write!(
                f,
                "entry function '{}' was not found in lowered module",
                function
            ),
            LlvmCodegenError::UnsupportedHirLowering { function, reason } => write!(
                f,
                "failed to lower function '{}' to llvm: {}",
                function,
                reason
            ),
        }
    }
}

enum RawBodySelection<'a> {
    None,
    Llvm(&'a crate::ast::LlvmIrBlock),
    Wasm,
    Conflict,
}

/// `#llvmir` ブロックを連結して LLVM IR テキストを生成する。
///
/// 現段階では手書き `#llvmir` を主経路とし、Parsed 関数は最小 subset のみ lower する。
pub fn emit_ll_from_module(module: &Module) -> Result<String, LlvmCodegenError> {
    emit_ll_from_module_for_target(module, CompileTarget::Llvm, BuildProfile::Debug)
}

/// `target/profile` 条件を評価しながら LLVM IR を生成する。
pub fn emit_ll_from_module_for_target(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<String, LlvmCodegenError> {
    let mut out = String::new();
    let entry_names = collect_active_entry_names(module, target, profile);
    let mut emitted_functions: Vec<String> = Vec::new();
    let mut pending_if: Option<bool> = None;

    for stmt in &module.root.items {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }

        match stmt {
            Stmt::LlvmIr(block) => {
                collect_defined_functions_from_llvmir_block(block, &mut emitted_functions);
                append_llvmir_block(&mut out, block);
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    collect_defined_functions_from_llvmir_block(block, &mut emitted_functions);
                    append_llvmir_block(&mut out, block);
                }
                FnBody::Parsed(block) => {
                    match select_raw_body_from_parsed_block(block, target, profile) {
                        RawBodySelection::Llvm(raw) => {
                            append_llvmir_block(&mut out, raw);
                        }
                        RawBodySelection::Wasm => {
                            return Err(LlvmCodegenError::UnsupportedWasmBody {
                                function: def.name.name.clone(),
                            });
                        }
                        RawBodySelection::Conflict => {
                            return Err(LlvmCodegenError::ConflictingRawBodies {
                                function: def.name.name.clone(),
                            });
                        }
                        RawBodySelection::None => {
                            if let Some(lowered) = lower_parsed_fn_with_gates(
                                def.name.name.as_str(),
                                &def.signature,
                                &def.params,
                                block,
                                target,
                                profile,
                            ) {
                                emitted_functions.push(def.name.name.clone());
                                out.push_str(&lowered);
                                out.push('\n');
                            }
                        }
                    }
                }
                FnBody::Wasm(_) => {
                    // `#wasm` は明示的な wasm backend 専用実装。
                    // 非 entry 関数は移行期間のためスキップするが、
                    // entry が #wasm のみの場合は LLVM 実行可能なモジュールを作れないためエラーとする。
                    if entry_names.iter().any(|n| n == &def.name.name) {
                        return Err(LlvmCodegenError::UnsupportedWasmBody {
                            function: def.name.name.clone(),
                        });
                    }
                }
            },
            _ => {}
        }
    }

    if let Some(entry) = entry_names.last() {
        if !emitted_functions.iter().any(|n| n == entry) {
            let _ = try_lower_entry_from_hir(
                module,
                target,
                profile,
                entry.as_str(),
                &mut out,
                &mut emitted_functions,
            );
        }
        if emitted_functions.iter().any(|n| n == entry)
            && entry != "main"
            && !emitted_functions.iter().any(|n| n == "main")
        {
            out.push_str(&format!(
                "define i32 @main() {{\nentry:\n  %0 = call i32 @{}()\n  ret i32 %0\n}}\n\n",
                entry
            ));
        }
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlTy {
    Void,
    I32,
    I64,
    F32,
    F64,
}

impl LlTy {
    fn ir(self) -> &'static str {
        match self {
            LlTy::Void => "void",
            LlTy::I32 => "i32",
            LlTy::I64 => "i64",
            LlTy::F32 => "float",
            LlTy::F64 => "double",
        }
    }
}

#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<LlTy>,
    ret: LlTy,
}

#[derive(Debug, Clone)]
struct LocalBinding {
    ptr: String,
    ty: LlTy,
}

#[derive(Debug, Clone)]
struct LlValue {
    ty: LlTy,
    repr: String,
}

struct LowerCtx<'a> {
    function_name: &'a str,
    sigs: &'a BTreeMap<String, FnSig>,
    strings: &'a [String],
    out: String,
    tmp_seq: usize,
    label_seq: usize,
    scopes: Vec<BTreeMap<String, LocalBinding>>,
}

impl<'a> LowerCtx<'a> {
    fn new(function_name: &'a str, sigs: &'a BTreeMap<String, FnSig>, strings: &'a [String]) -> Self {
        Self {
            function_name,
            sigs,
            strings,
            out: String::new(),
            tmp_seq: 0,
            label_seq: 0,
            scopes: Vec::new(),
        }
    }

    fn push_line(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn next_tmp(&mut self) -> String {
        let name = format!("%t{}", self.tmp_seq);
        self.tmp_seq += 1;
        name
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_seq);
        self.label_seq += 1;
        label
    }

    fn begin_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind_local(&mut self, name: &str, ptr: String, ty: LlTy) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), LocalBinding { ptr, ty });
        }
    }

    fn lookup_local(&self, name: &str) -> Option<&LocalBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
}

fn try_lower_entry_from_hir(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry: &str,
    out: &mut String,
    emitted_functions: &mut Vec<String>,
) -> Result<(), LlvmCodegenError> {
    let (mut types, mut hir) = build_hir_for_llvm_lowering(module, target, profile)?;
    crate::passes::insert_drops(&mut hir, types.unit());

    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &hir.functions {
        function_map.insert(f.name.clone(), f);
    }
    if !function_map.contains_key(entry) {
        return Err(LlvmCodegenError::MissingEntryFunction {
            function: entry.to_string(),
        });
    }

    let mut sigs = collect_hir_signatures(&types, &hir);
    let reachable = collect_reachable_functions(&hir, entry);

    for ex in &hir.externs {
        if reachable.iter().any(|n| n == &ex.local_name) {
            let name = ll_symbol(ex.local_name.as_str());
            let params = ex
                .params
                .iter()
                .map(|t| llty_for_type(&types, *t).ir())
                .collect::<Vec<_>>()
                .join(", ");
            let ret = llty_for_type(&types, ex.result).ir();
            out.push_str(&format!("declare {} {}({})\n", ret, name, params));
            if !emitted_functions.iter().any(|n| n == &ex.local_name) {
                emitted_functions.push(ex.local_name.clone());
            }
        }
    }
    if !reachable.is_empty() {
        out.push('\n');
    }

    for name in reachable {
        if emitted_functions.iter().any(|n| n == &name) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        match &func.body {
            HirBody::LlvmIr(raw) => {
                append_llvmir_block(out, raw);
                emitted_functions.push(name);
            }
            HirBody::Wasm(_) => {
                return Err(LlvmCodegenError::UnsupportedWasmBody {
                    function: func.name.clone(),
                });
            }
            HirBody::Block(block) => {
                let lowered = lower_hir_function(&types, &hir, &sigs, func, block)?;
                out.push_str(&lowered);
                out.push('\n');
                emitted_functions.push(name);
            }
        }
    }

    if entry == "main" && emitted_functions.iter().any(|n| n == "__nepl_entry_main") {
        out.push_str("define i32 @main() {\nentry:\n  call void @__nepl_entry_main()\n  ret i32 0\n}\n\n");
        emitted_functions.push(String::from("main"));
    }

    // suppress unused warning when future passes extend signature synthesis
    sigs.clear();
    Ok(())
}

fn build_hir_for_llvm_lowering(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<(TypeCtx, HirModule), LlvmCodegenError> {
    match try_build_hir_with_target(module, target, profile) {
        Ok(hir) => return Ok(hir),
        Err(primary_reason) => {
            if matches!(target, CompileTarget::Llvm) {
                match try_build_hir_with_target(module, CompileTarget::Wasi, profile) {
                    Ok(hir) => return Ok(hir),
                    Err(fallback_reason) => {
                        return Err(LlvmCodegenError::TypecheckFailed {
                            reason: format!(
                                "llvm check: {} / std-compatible fallback: {}",
                                primary_reason, fallback_reason
                            ),
                        });
                    }
                }
            }
            return Err(LlvmCodegenError::TypecheckFailed {
                reason: primary_reason,
            });
        }
    }
}

fn try_build_hir_with_target(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<(TypeCtx, HirModule), String> {
    let typed = crate::typecheck::typecheck(module, target, profile);
    let Some(typed_module) = typed.module else {
        return Err(summarize_diagnostics_for_message(&typed.diagnostics));
    };
    let mut types = typed.types;
    let hir = crate::monomorphize::monomorphize(&mut types, typed_module);
    Ok((types, hir))
}

fn collect_hir_signatures(types: &TypeCtx, module: &HirModule) -> BTreeMap<String, FnSig> {
    let mut out = BTreeMap::new();
    for f in &module.functions {
        let params = f.params.iter().map(|p| llty_for_type(types, p.ty)).collect::<Vec<_>>();
        let ret = llty_for_type(types, f.result);
        out.insert(f.name.clone(), FnSig { params, ret });
    }
    for ex in &module.externs {
        let params = ex
            .params
            .iter()
            .map(|p| llty_for_type(types, *p))
            .collect::<Vec<_>>();
        let ret = llty_for_type(types, ex.result);
        out.insert(ex.local_name.clone(), FnSig { params, ret });
    }
    out
}

fn collect_reachable_functions(module: &HirModule, entry: &str) -> Vec<String> {
    let mut function_map: BTreeMap<String, &HirFunction> = BTreeMap::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut stack = Vec::new();
    stack.push(entry.to_string());
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        let mut callees = BTreeSet::new();
        collect_callees_in_body(&func.body, &mut callees);
        for c in callees {
            if !visited.contains(c.as_str()) {
                stack.push(c);
            }
        }
    }
    visited.into_iter().collect::<Vec<_>>()
}

fn collect_callees_in_body(body: &HirBody, out: &mut BTreeSet<String>) {
    if let HirBody::Block(block) = body {
        collect_callees_in_block(block, out);
    }
}

fn collect_callees_in_block(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        collect_callees_in_expr(&line.expr, out);
    }
}

fn collect_callees_in_expr(expr: &HirExpr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _) => {
                    out.insert(name.clone());
                }
                FuncRef::Trait { .. } => {}
            }
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_callees_in_expr(cond, out);
            collect_callees_in_expr(then_branch, out);
            collect_callees_in_expr(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_callees_in_expr(cond, out);
            collect_callees_in_expr(body, out);
        }
        HirExprKind::Block(b) => collect_callees_in_block(b, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_callees_in_expr(value, out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => collect_callees_in_expr(inner, out),
        HirExprKind::Match { scrutinee, arms } => {
            collect_callees_in_expr(scrutinee, out);
            for arm in arms {
                collect_callees_in_expr(&arm.body, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_callees_in_expr(payload, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } | HirExprKind::TupleConstruct { items: fields } => {
            for f in fields {
                collect_callees_in_expr(f, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_callees_in_expr(callee, out);
            for a in args {
                collect_callees_in_expr(a, out);
            }
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_)
        | HirExprKind::Drop { .. } => {}
    }
}

fn lower_hir_function(
    types: &TypeCtx,
    module: &HirModule,
    sigs: &BTreeMap<String, FnSig>,
    func: &HirFunction,
    block: &HirBlock,
) -> Result<String, LlvmCodegenError> {
    let mut exported_name = func.name.clone();
    let mut ret_ty = llty_for_type(types, func.result);
    if func.name == "main" && matches!(ret_ty, LlTy::Void) {
        exported_name = String::from("__nepl_entry_main");
        ret_ty = LlTy::Void;
    }

    let mut ctx = LowerCtx::new(func.name.as_str(), sigs, &module.string_literals);
    let mut params = Vec::new();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        params.push(format!("{} %p{}", pty.ir(), idx));
    }
    ctx.push_line(&format!(
        "define {} {}({}) {{",
        ret_ty.ir(),
        ll_symbol(exported_name.as_str()),
        params.join(", ")
    ));
    ctx.push_line("entry:");

    ctx.begin_scope();
    for (idx, p) in func.params.iter().enumerate() {
        let pty = llty_for_type(types, p.ty);
        let ptr = ctx.next_tmp();
        ctx.push_line(&format!("  {} = alloca {}", ptr, pty.ir()));
        ctx.push_line(&format!(
            "  store {} %p{}, {}* {}",
            pty.ir(),
            idx,
            pty.ir(),
            ptr
        ));
        ctx.bind_local(p.name.as_str(), ptr, pty);
    }

    let ret_val = lower_hir_block(types, &mut ctx, block)?;
    match ret_ty {
        LlTy::Void => {
            ctx.push_line("  ret void");
        }
        _ => {
            if let Some(v) = ret_val {
                if v.ty == ret_ty {
                    ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), v.repr));
                } else {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: func.name.clone(),
                        reason: format!("return type mismatch {:?} -> {:?}", v.ty, ret_ty),
                    });
                }
            } else {
                let zero = match ret_ty {
                    LlTy::I32 => "0",
                    LlTy::I64 => "0",
                    LlTy::F32 => "0.0",
                    LlTy::F64 => "0.0",
                    LlTy::Void => "",
                };
                ctx.push_line(&format!("  ret {} {}", ret_ty.ir(), zero));
            }
        }
    }
    ctx.end_scope();
    ctx.push_line("}");
    Ok(ctx.out)
}

fn lower_hir_block(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    block: &HirBlock,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    ctx.begin_scope();
    let mut last = None;
    for line in &block.lines {
        let v = lower_hir_expr(types, ctx, &line.expr)?;
        if !line.drop_result {
            last = v;
        }
    }
    ctx.end_scope();
    Ok(last)
}

fn lower_hir_expr(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    expr: &HirExpr,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    match &expr.kind {
        HirExprKind::LiteralI32(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: format!("{}", v),
        })),
        HirExprKind::LiteralF32(v) => Ok(Some(LlValue {
            ty: LlTy::F32,
            repr: format!("{}", v),
        })),
        HirExprKind::LiteralBool(v) => Ok(Some(LlValue {
            ty: LlTy::I32,
            repr: if *v { String::from("1") } else { String::from("0") },
        })),
        HirExprKind::LiteralStr(id) => lower_hir_string_literal(types, ctx, *id as usize),
        HirExprKind::Unit => Ok(None),
        HirExprKind::Var(name) => {
            let Some(binding) = ctx.lookup_local(name.as_str()) else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("unknown variable '{}'", name),
                });
            };
            let bty = binding.ty;
            let bptr = binding.ptr.clone();
            let tmp = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = load {}, {}* {}",
                tmp,
                bty.ir(),
                bty.ir(),
                bptr
            ));
            Ok(Some(LlValue {
                ty: bty,
                repr: tmp,
            }))
        }
        HirExprKind::Let { name, value, .. } => {
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                return Ok(None);
            };
            let ptr = ctx.next_tmp();
            ctx.push_line(&format!("  {} = alloca {}", ptr, v.ty.ir()));
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                v.ty.ir(),
                ptr
            ));
            ctx.bind_local(name.as_str(), ptr, v.ty);
            Ok(None)
        }
        HirExprKind::Set { name, value } => {
            let Some(binding) = ctx.lookup_local(name.as_str()).cloned() else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("set on unknown variable '{}'", name),
                });
            };
            let Some(v) = lower_hir_expr(types, ctx, value)? else {
                return Ok(None);
            };
            if v.ty != binding.ty {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: format!("set type mismatch {:?} -> {:?}", v.ty, binding.ty),
                });
            }
            ctx.push_line(&format!(
                "  store {} {}, {}* {}",
                v.ty.ir(),
                v.repr,
                binding.ty.ir(),
                binding.ptr
            ));
            Ok(None)
        }
        HirExprKind::Call { callee, args } => {
            let callee_name = match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _) => name.as_str(),
                FuncRef::Trait { trait_name, method, .. } => {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!("trait call {}::{} is not yet supported", trait_name, method),
                    });
                }
            };
            let mut lowered_args = Vec::new();
            for a in args {
                if let Some(v) = lower_hir_expr(types, ctx, a)? {
                    lowered_args.push(v);
                }
            }
            let sig = ctx.sigs.get(callee_name).ok_or_else(|| LlvmCodegenError::UnsupportedHirLowering {
                function: ctx.function_name.to_string(),
                reason: format!("missing function signature for '{}'", callee_name),
            })?;
            let mut args_ir = Vec::new();
            for (idx, v) in lowered_args.iter().enumerate() {
                let ty = sig.params.get(idx).copied().unwrap_or(v.ty);
                if ty != v.ty {
                    return Err(LlvmCodegenError::UnsupportedHirLowering {
                        function: ctx.function_name.to_string(),
                        reason: format!(
                            "call argument type mismatch on '{}': expected {:?}, got {:?}",
                            callee_name, ty, v.ty
                        ),
                    });
                }
                args_ir.push(format!("{} {}", ty.ir(), v.repr));
            }
            match sig.ret {
                LlTy::Void => {
                    ctx.push_line(&format!(
                        "  call {} {}({})",
                        sig.ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(None)
                }
                ret => {
                    let tmp = ctx.next_tmp();
                    ctx.push_line(&format!(
                        "  {} = call {} {}({})",
                        tmp,
                        ret.ir(),
                        ll_symbol(callee_name),
                        args_ir.join(", ")
                    ));
                    Ok(Some(LlValue { ty: ret, repr: tmp }))
                }
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("if condition must produce a value"),
                });
            };
            if cond_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("if condition must be i32/bool-compatible"),
                });
            }
            let cond_i1 = ctx.next_tmp();
            ctx.push_line(&format!(
                "  {} = icmp ne i32 {}, 0",
                cond_i1, cond_v.repr
            ));
            let then_label = ctx.next_label("if_then");
            let else_label = ctx.next_label("if_else");
            let end_label = ctx.next_label("if_end");
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cond_i1, then_label, else_label
            ));

            let result_ty = llty_for_type(types, expr.ty);
            let result_slot = if result_ty != LlTy::Void {
                let slot = ctx.next_tmp();
                ctx.push_line(&format!("  {} = alloca {}", slot, result_ty.ir()));
                Some(slot)
            } else {
                None
            };

            ctx.push_line(&format!("{}:", then_label));
            if let Some(tv) = lower_hir_expr(types, ctx, then_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if tv.ty != result_ty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("then branch result type mismatch"),
                        });
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        tv.ty.ir(),
                        tv.repr,
                        tv.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));

            ctx.push_line(&format!("{}:", else_label));
            if let Some(ev) = lower_hir_expr(types, ctx, else_branch)? {
                if let Some(slot) = result_slot.as_ref() {
                    if ev.ty != result_ty {
                        return Err(LlvmCodegenError::UnsupportedHirLowering {
                            function: ctx.function_name.to_string(),
                            reason: String::from("else branch result type mismatch"),
                        });
                    }
                    ctx.push_line(&format!(
                        "  store {} {}, {}* {}",
                        ev.ty.ir(),
                        ev.repr,
                        ev.ty.ir(),
                        slot
                    ));
                }
            }
            ctx.push_line(&format!("  br label %{}", end_label));
            ctx.push_line(&format!("{}:", end_label));
            if let Some(slot) = result_slot {
                let tmp = ctx.next_tmp();
                ctx.push_line(&format!(
                    "  {} = load {}, {}* {}",
                    tmp,
                    result_ty.ir(),
                    result_ty.ir(),
                    slot
                ));
                Ok(Some(LlValue {
                    ty: result_ty,
                    repr: tmp,
                }))
            } else {
                Ok(None)
            }
        }
        HirExprKind::While { cond, body } => {
            let cond_label = ctx.next_label("while_cond");
            let body_label = ctx.next_label("while_body");
            let end_label = ctx.next_label("while_end");
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", cond_label));
            let Some(cond_v) = lower_hir_expr(types, ctx, cond)? else {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("while condition must produce a value"),
                });
            };
            if cond_v.ty != LlTy::I32 {
                return Err(LlvmCodegenError::UnsupportedHirLowering {
                    function: ctx.function_name.to_string(),
                    reason: String::from("while condition must be i32/bool-compatible"),
                });
            }
            let cmp = ctx.next_tmp();
            ctx.push_line(&format!("  {} = icmp ne i32 {}, 0", cmp, cond_v.repr));
            ctx.push_line(&format!(
                "  br i1 {}, label %{}, label %{}",
                cmp, body_label, end_label
            ));
            ctx.push_line(&format!("{}:", body_label));
            let _ = lower_hir_expr(types, ctx, body)?;
            ctx.push_line(&format!("  br label %{}", cond_label));
            ctx.push_line(&format!("{}:", end_label));
            Ok(None)
        }
        HirExprKind::Block(block) => lower_hir_block(types, ctx, block),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args: _,
        } => {
            if name == "size_of" || name == "align_of" {
                if let Some(ty) = type_args.first() {
                    let size = match types.get(types.resolve_id(*ty)) {
                        TypeKind::U8 => 1,
                        TypeKind::Named(ref n) if n == "i64" || n == "f64" => 8,
                        TypeKind::Unit => 0,
                        _ => 4,
                    };
                    return Ok(Some(LlValue {
                        ty: LlTy::I32,
                        repr: format!("{}", size),
                    }));
                }
            }
            if name == "unreachable" {
                ctx.push_line("  unreachable");
                return Ok(None);
            }
            Err(LlvmCodegenError::UnsupportedHirLowering {
                function: ctx.function_name.to_string(),
                reason: format!("unsupported intrinsic '{}'", name),
            })
        }
        HirExprKind::Drop { .. } => Ok(None),
        other => Err(LlvmCodegenError::UnsupportedHirLowering {
            function: ctx.function_name.to_string(),
            reason: format!("unsupported expression kind {:?}", other),
        }),
    }
}

fn lower_hir_string_literal(
    _types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    id: usize,
) -> Result<Option<LlValue>, LlvmCodegenError> {
    let Some(s) = ctx.strings.get(id) else {
        return Err(LlvmCodegenError::UnsupportedHirLowering {
            function: ctx.function_name.to_string(),
            reason: format!("string literal id {} was out of bounds", id),
        });
    };
    let bytes = s.as_bytes();
    let alloc_sig = ctx.sigs.get("alloc").ok_or_else(|| LlvmCodegenError::UnsupportedHirLowering {
        function: ctx.function_name.to_string(),
        reason: String::from("alloc function is required to materialize string literals"),
    })?;
    if alloc_sig.params.len() != 1 || alloc_sig.ret != LlTy::I32 {
        return Err(LlvmCodegenError::UnsupportedHirLowering {
            function: ctx.function_name.to_string(),
            reason: String::from("alloc signature is incompatible"),
        });
    }
    let ptr_tmp = ctx.next_tmp();
    let total_len = (bytes.len() + 4) as i32;
    ctx.push_line(&format!(
        "  {} = call i32 {}(i32 {})",
        ptr_tmp,
        ll_symbol("alloc"),
        total_len
    ));
    ctx.push_line(&format!(
        "  call void {}(i32 {}, i32 {})",
        ll_symbol("store_i32"),
        ptr_tmp,
        bytes.len()
    ));
    for (idx, b) in bytes.iter().enumerate() {
        let off = ctx.next_tmp();
        ctx.push_line(&format!("  {} = add i32 {}, {}", off, ptr_tmp, idx + 4));
        ctx.push_line(&format!(
            "  call void {}(i32 {}, i32 {})",
            ll_symbol("store_u8"),
            off,
            *b as i32
        ));
    }
    Ok(Some(LlValue {
        ty: LlTy::I32,
        repr: ptr_tmp,
    }))
}

fn llty_for_type(types: &TypeCtx, ty: TypeId) -> LlTy {
    match types.get(types.resolve_id(ty)) {
        TypeKind::Unit | TypeKind::Never => LlTy::Void,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Str => LlTy::I32,
        TypeKind::F32 => LlTy::F32,
        TypeKind::Named(name) if name == "i64" => LlTy::I64,
        TypeKind::Named(name) if name == "f64" => LlTy::F64,
        TypeKind::Reference(_, _) => LlTy::I32,
        TypeKind::Box(_) => LlTy::I32,
        TypeKind::Tuple { .. } => LlTy::I32,
        TypeKind::Struct { .. } => LlTy::I32,
        TypeKind::Enum { .. } => LlTy::I32,
        TypeKind::Apply { .. } => LlTy::I32,
        TypeKind::Function { .. } => LlTy::I32,
        TypeKind::Var(_) => LlTy::I32,
        TypeKind::Named(_) => LlTy::I32,
    }
}

fn ll_symbol(name: &str) -> String {
    let escaped = name
        .replace('\\', "\\5C")
        .replace('"', "\\22");
    format!("@\"{}\"", escaped)
}

fn summarize_diagnostics_for_message(diags: &[crate::diagnostic::Diagnostic]) -> String {
    let mut uniq = BTreeSet::new();
    for d in diags {
        if matches!(d.severity, crate::diagnostic::Severity::Error) {
            uniq.insert(d.message.clone());
        }
    }
    if uniq.is_empty() {
        return String::from("no diagnostic details");
    }
    let total = uniq.len();
    let mut parts = uniq.into_iter().take(3).collect::<Vec<_>>();
    if total > 3 {
        parts.push(format!("... and {} more", total - 3));
    }
    parts.join(" / ")
}

fn collect_defined_functions_from_llvmir_block(
    block: &crate::ast::LlvmIrBlock,
    out: &mut Vec<String>,
) {
    for line in &block.lines {
        if let Some(name) = parse_defined_function_name(line) {
            if !out.iter().any(|n| n == name) {
                out.push(String::from(name));
            }
        }
    }
}

fn parse_defined_function_name(line: &str) -> Option<&str> {
    // 例: define i32 @foo(i32 %x) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("define ") {
        return None;
    }
    let at = trimmed.find('@')?;
    let rest = &trimmed[(at + 1)..];
    let end = rest.find('(')?;
    let name = &rest[..end];
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn collect_active_entry_names(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Vec<String> {
    let mut pending_if: Option<bool> = None;
    let mut out = Vec::new();
    for stmt in &module.root.items {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::Directive(Directive::Entry { name }) = stmt {
            out.push(name.name.clone());
        }
    }
    out
}

fn gate_allows(d: &Directive, target: CompileTarget, profile: BuildProfile) -> Option<bool> {
    match d {
        Directive::IfTarget { target: gate, .. } => Some(target.allows(gate.as_str())),
        Directive::IfProfile { profile: p, .. } => Some(profile_allows(p.as_str(), profile)),
        _ => None,
    }
}

fn profile_allows(profile: &str, active: BuildProfile) -> bool {
    match profile {
        "debug" => matches!(active, BuildProfile::Debug),
        "release" => matches!(active, BuildProfile::Release),
        _ => false,
    }
}

fn append_llvmir_block(out: &mut String, block: &crate::ast::LlvmIrBlock) {
    for line in &block.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
}

fn active_stmt_indices(block: &Block, target: CompileTarget, profile: BuildProfile) -> Vec<usize> {
    let mut pending_if: Option<bool> = None;
    let mut out = Vec::new();
    for (idx, stmt) in block.items.iter().enumerate() {
        if let Stmt::Directive(d) = stmt {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if allowed {
            out.push(idx);
        }
    }
    out
}

fn select_raw_body_from_parsed_block<'a>(
    block: &'a Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> RawBodySelection<'a> {
    let mut selected: Option<RawBodySelection<'a>> = None;
    for idx in active_stmt_indices(block, target, profile) {
        match &block.items[idx] {
            Stmt::LlvmIr(raw) => {
                if selected.is_some() {
                    return RawBodySelection::Conflict;
                }
                selected = Some(RawBodySelection::Llvm(raw));
            }
            Stmt::Wasm(_) => {
                if selected.is_some() {
                    return RawBodySelection::Conflict;
                }
                selected = Some(RawBodySelection::Wasm);
            }
            Stmt::Directive(_) => {}
            _ => return RawBodySelection::None,
        }
    }
    selected.unwrap_or(RawBodySelection::None)
}

fn lower_parsed_fn_with_gates(
    name: &str,
    signature: &TypeExpr,
    params: &[Ident],
    body: &Block,
    target: CompileTarget,
    profile: BuildProfile,
) -> Option<String> {
    if !params.is_empty() {
        return None;
    }

    let result_ty = match signature {
        TypeExpr::Function { result, .. } => result.as_ref(),
        _ => return None,
    };
    if !matches!(result_ty, TypeExpr::I32) {
        return None;
    }

    let active = active_stmt_indices(body, target, profile);
    if active.len() != 1 {
        return None;
    }
    let ret_value = match &body.items[active[0]] {
        Stmt::Expr(expr) => lower_i32_literal_expr(expr)?,
        _ => return None,
    };

    Some(format!(
        "define i32 @{}() {{\nentry:\n  ret i32 {}\n}}",
        name, ret_value
    ))
}

fn lower_i32_literal_expr(expr: &PrefixExpr) -> Option<i32> {
    if expr.items.len() != 1 {
        return None;
    }
    match &expr.items[0] {
        PrefixItem::Literal(Literal::Int(text), _) => parse_i32_literal(text),
        _ => None,
    }
}

fn parse_i32_literal(text: &str) -> Option<i32> {
    if let Some(hex) = text.strip_prefix("0x") {
        i32::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = text.strip_prefix("-0x") {
        i32::from_str_radix(hex, 16).ok().map(|v| -v)
    } else {
        text.parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;
    use crate::lexer;
    use crate::parser;
    use crate::span::FileId;

    fn parse_module(src: &str) -> Module {
        let file_id = FileId(0);
        let lexed = lexer::lex(file_id, src);
        let parsed = parser::parse_tokens(file_id, lexed);
        let has_error = parsed
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        assert!(!has_error, "parse diagnostics: {:?}", parsed.diagnostics);
        parsed.module.expect("module should parse")
    }

    #[test]
    fn emit_ll_collects_top_and_fn_blocks() {
        let src = r#"
#indent 4
#target llvm

#llvmir:
    ; module header
    target triple = "x86_64-pc-linux-gnu"

fn body <()->i32> ():
    #llvmir:
        define i32 @body() {
        entry:
            ret i32 7
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("llvm ir should be emitted");
        assert!(ll.contains("; module header"));
        assert!(ll.contains("define i32 @body()"));
        assert!(ll.contains("    ret i32 7"));
    }

    #[test]
    fn emit_ll_skips_unsupported_parsed_function_body() {
        let src = r#"
#target llvm
fn body <()->i32> ():
    add 1 2
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("unsupported parsed function should be skipped");
        assert!(!ll.contains("define i32 @body()"));
    }

    #[test]
    fn emit_ll_supports_parsed_const_i32_function() {
        let src = r#"
#target llvm
fn c <()->i32> ():
    123
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("const i32 function should be lowered");
        assert!(ll.contains("define i32 @c()"));
        assert!(ll.contains("ret i32 123"));
    }

    #[test]
    fn emit_ll_respects_if_target_gate() {
        let src = r#"
#target llvm
#if[target=wasm]
fn w <()->i32> ():
    #wasm:
        i32.const 1

#if[target=llvm]
fn l <()->i32> ():
    #llvmir:
        define i32 @l() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(&module, CompileTarget::Llvm, BuildProfile::Debug)
            .expect("llvm-gated items should compile");
        assert!(ll.contains("define i32 @l()"));
        assert!(!ll.contains("define i32 @w()"));
    }

    #[test]
    fn emit_ll_supports_function_body_if_target_raw() {
        let src = r#"
#target llvm
fn f <()->i32> ():
    #if[target=wasm]
    #wasm:
        i32.const 1
    #if[target=llvm]
    #llvmir:
        define i32 @f() {
        entry:
            ret i32 42
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module_for_target(&module, CompileTarget::Llvm, BuildProfile::Debug)
            .expect("llvm raw function body should be selected");
        assert!(ll.contains("define i32 @f()"));
        assert!(ll.contains("ret i32 42"));
    }

    #[test]
    fn emit_ll_rejects_entry_with_wasm_body() {
        let src = r#"
#target llvm
#entry main
fn main <()->i32> ():
    #wasm:
        i32.const 1
"#;
        let module = parse_module(src);
        let err = emit_ll_from_module(&module).expect_err("entry with #wasm body must fail");
        assert_eq!(
            err,
            LlvmCodegenError::UnsupportedWasmBody {
                function: "main".to_string()
            }
        );
    }

    #[test]
    fn emit_ll_generates_main_bridge_from_entry() {
        let src = r#"
#target llvm
#entry boot
fn boot <()->i32> ():
    #llvmir:
        define i32 @boot() {
        entry:
            ret i32 9
        }
"#;
        let module = parse_module(src);
        let ll = emit_ll_from_module(&module).expect("entry bridge should be emitted");
        assert!(ll.contains("define i32 @boot()"));
        assert!(ll.contains("define i32 @main()"));
        assert!(ll.contains("call i32 @boot()"));
    }

}

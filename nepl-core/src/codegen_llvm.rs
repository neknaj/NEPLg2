//! LLVM IR 生成（core 側）
//!
//! このモジュールは AST から LLVM IR テキストを生成する責務のみを持つ。
//! clang 実行などのホスト依存処理は `nepl-cli` 側で扱う。

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Block, FnBody, Ident, Literal, Module, PrefixExpr, PrefixItem, Stmt, TypeExpr};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::ast::Directive;

/// LLVM IR 生成時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmCodegenError {
    MissingLlvmIrBlock,
    UnsupportedParsedFunctionBody { function: String },
    UnsupportedWasmBody { function: String },
    ConflictingRawBodies { function: String },
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
                append_llvmir_block(&mut out, block);
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
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

    Ok(out)
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
}

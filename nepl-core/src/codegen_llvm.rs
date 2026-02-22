//! LLVM IR 生成（core 側）
//!
//! このモジュールは AST から LLVM IR テキストを生成する責務のみを持つ。
//! clang 実行などのホスト依存処理は `nepl-cli` 側で扱う。

extern crate alloc;

use alloc::format;
use alloc::string::String;

use crate::ast::{Block, FnBody, Ident, Literal, Module, PrefixExpr, PrefixItem, Stmt, TypeExpr};

/// LLVM IR 生成時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmCodegenError {
    MissingLlvmIrBlock,
    UnsupportedParsedFunctionBody { function: String },
    UnsupportedWasmBody { function: String },
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
        }
    }
}

/// `#llvmir` ブロックを連結して LLVM IR テキストを生成する。
///
/// 現段階では手書き `#llvmir` を主経路とし、Parsed 関数は最小 subset のみ lower する。
pub fn emit_ll_from_module(module: &Module) -> Result<String, LlvmCodegenError> {
    let mut out = String::new();
    let mut saw_llvmir = false;

    for stmt in &module.root.items {
        match stmt {
            Stmt::LlvmIr(block) => {
                append_llvmir_block(&mut out, block);
                saw_llvmir = true;
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    append_llvmir_block(&mut out, block);
                    saw_llvmir = true;
                }
                FnBody::Parsed(block) => {
                    let lowered =
                        lower_parsed_fn(def.name.name.as_str(), &def.signature, &def.params, block)
                            .ok_or_else(|| LlvmCodegenError::UnsupportedParsedFunctionBody {
                                function: def.name.name.clone(),
                            })?;
                    out.push_str(&lowered);
                    out.push('\n');
                    saw_llvmir = true;
                }
                FnBody::Wasm(_) => {
                    return Err(LlvmCodegenError::UnsupportedWasmBody {
                        function: def.name.name.clone(),
                    });
                }
            },
            _ => {}
        }
    }

    if !saw_llvmir {
        return Err(LlvmCodegenError::MissingLlvmIrBlock);
    }

    Ok(out)
}

fn append_llvmir_block(out: &mut String, block: &crate::ast::LlvmIrBlock) {
    for line in &block.lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
}

fn lower_parsed_fn(name: &str, signature: &TypeExpr, params: &[Ident], body: &Block) -> Option<String> {
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

    if body.items.len() != 1 {
        return None;
    }
    let ret_value = match &body.items[0] {
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
    fn emit_ll_rejects_unsupported_parsed_function_body() {
        let src = r#"
#target llvm
fn body <()->i32> ():
    add 1 2
"#;
        let module = parse_module(src);
        let err = emit_ll_from_module(&module).expect_err("must reject parsed function body");
        let msg = format!("{err}");
        assert!(msg.contains("supported subset"));
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
}

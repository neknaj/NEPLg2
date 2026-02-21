use std::collections::BTreeMap;
use std::path::PathBuf;

use js_sys::{Reflect, Uint8Array};
use nepl_core::ast::{Block, Directive, FnBody, MatchArm, PrefixExpr, PrefixItem, Stmt, Symbol};
use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::error::CoreError;
use nepl_core::hir::{HirBlock, HirExpr, HirExprKind, HirLine};
use nepl_core::lexer::{lex, Token, TokenKind};
use nepl_core::loader::{Loader, SourceMap};
use nepl_core::parser::parse_tokens;
use nepl_core::span::{FileId, Span};
use nepl_core::typecheck::typecheck;
use nepl_core::{compile_module, BuildProfile, CompileOptions, CompileTarget};
use wasmprinter::print_bytes;
use wasm_bindgen::prelude::*;

const NEPLG2_REPO_URL: &str = "https://github.com/neknaj/NEPLg2/";
const NEPLG2_COMMIT_BASE_URL: &str = "https://github.com/neknaj/NEPLg2/commit/";

// build.rs などで NEPLG2_COMPILER_COMMIT が設定されていればそれを使う。
// wasm 実行時には git コマンド等を呼べないため、ビルド時埋め込みが前提。
const NEPLG2_COMPILER_COMMIT: &str = match option_env!("NEPLG2_COMPILER_COMMIT") {
    Some(v) if !v.is_empty() => v,
    _ => "unknown",
};

fn build_wat_header_comments() -> String {
    // WAT の行コメントは `;;` で始まる。([spec] コメントは字句要素として扱われる)
    // ここでは確実にコメント化できるよう、行ごとに `;; ` を付ける。
    let mut out = String::new();
    out.push_str(";; compiler: NEPLg2 ");
    out.push_str(NEPLG2_REPO_URL);
    out.push('\n');

    out.push_str(";; compiler commit: ");
    out.push_str(NEPLG2_COMPILER_COMMIT);
    out.push('\n');

    out.push_str(";; compiler commit url: ");
    if NEPLG2_COMPILER_COMMIT != "unknown" {
        out.push_str(NEPLG2_COMMIT_BASE_URL);
        out.push_str(NEPLG2_COMPILER_COMMIT);
    } else {
        out.push_str("(unknown)");
    }
    out.push_str("\n\n");
    out
}

fn build_attached_source_comment(entry_path: &str, source: &str) -> String {
    // 入力ソースを WAT コメントとして先頭に埋め込む（行コメントで安全に固定する）
    let mut out = String::new();
    out.push_str(";; ---- BEGIN ATTACHED SOURCE ----\n");
    out.push_str(";; path: ");
    out.push_str(entry_path);
    out.push('\n');

    for (i, line) in source.lines().enumerate() {
        // 例: ";; 0001: let x = 1"
        out.push_str(";; ");
        out.push_str(&format!("{:04}: ", i + 1));
        out.push_str(line);
        out.push('\n');
    }

    // source が末尾改行で終わっていても lines() は最後の空行を落とすため、
    // 入力の雰囲気を残したいならここで明示的に 1 行足しておく。
    if source.ends_with('\n') {
        out.push_str(";; 0000: \n");
    }

    out.push_str(";; ---- END ATTACHED SOURCE ----\n\n");
    out
}

fn build_nepl_wat_debug_comment(debug_text: &str) -> String {
    if debug_text.trim().is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(";; ---- BEGIN NEPL WAT DEBUG ----\n");
    for line in debug_text.lines() {
        out.push_str(";; ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(";; ---- END NEPL WAT DEBUG ----\n\n");
    out
}

fn decorate_wat(wat: String, attach_source: bool, entry_path: &str, source: &str, wat_debug: &str) -> String {
    // WAT/wasmprinter の本文の前に、コンパイラ情報＋（必要なら）入力ソースを差し込む
    let mut out = String::new();
    out.push_str(&build_wat_header_comments());
    out.push_str(&build_nepl_wat_debug_comment(wat_debug));
    if attach_source {
        out.push_str(&build_attached_source_comment(entry_path, source));
    }
    out.push_str(&wat);
    out
}

fn make_wat(
    wasm: &[u8],
    attach_source: bool,
    entry_path: &str,
    source: &str,
    wat_debug: &str,
) -> Result<String, JsValue> {
    let wat = print_bytes(wasm).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(decorate_wat(wat, attach_source, entry_path, source, wat_debug))
}

fn make_wat_min(wasm: &[u8], attach_source: bool, entry_path: &str, source: &str) -> Result<String, JsValue> {
    // wat-min では圧縮後に、既存の compiler/source コメントのみ付加する。
    // NEPL 詳細注釈は付与しない。
    let wat = print_bytes(wasm).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let min = minify_wat_text(&wat);
    Ok(decorate_wat(min, attach_source, entry_path, source, ""))
}

// main.rs の wat-min と同等の単純 minify：
// - 文字列リテラル（"..."）内はそのまま
// - 行コメント `;; ...` とブロックコメント `(; ... ;)` を除去
// - 空白を 1 個に圧縮し、括弧の前後の空白を削る
fn minify_wat_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut comment_depth = 0usize;
    let mut prev_space = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                // エスケープシーケンス（\" など）を 1 文字進めて保持する
                if let Some(next) = chars.next() {
                    out.push(next);
                }
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }

        if comment_depth > 0 {
            // ネストしたブロックコメント `(; ... ;)` に対応
            if c == '(' && chars.peek() == Some(&';') {
                chars.next();
                comment_depth += 1;
                continue;
            }
            if c == ';' && chars.peek() == Some(&')') {
                chars.next();
                comment_depth = comment_depth.saturating_sub(1);
                if comment_depth == 0 && !prev_space && !out.is_empty() {
                    out.push(' ');
                    prev_space = true;
                }
                continue;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            prev_space = false;
            continue;
        }

        // 行コメント `;; ...`
        if c == ';' && chars.peek() == Some(&';') {
            chars.next();
            while let Some(next) = chars.next() {
                if next == '\n' {
                    break;
                }
            }
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }

        // ブロックコメント `(; ... ;)`
        if c == '(' && chars.peek() == Some(&';') {
            chars.next();
            comment_depth = 1;
            continue;
        }

        // 空白の圧縮
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }

        // 括弧の直前の空白を削る
        if c == '(' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push('(');
            prev_space = false;
            continue;
        }
        if c == ')' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push(')');
            prev_space = false;
            continue;
        }

        out.push(c);
        prev_space = false;
    }

    out.trim().to_string()
}

fn parse_emit_list(emit: JsValue) -> Result<Vec<String>, JsValue> {
    // emit は "wasm"/"wat"/"wat-min" の文字列、またはそれらの配列を想定する
    if emit.is_null() || emit.is_undefined() {
        return Ok(vec!["wasm".to_string()]);
    }
    if let Some(s) = emit.as_string() {
        return Ok(vec![s]);
    }
    if js_sys::Array::is_array(&emit) {
        let arr = js_sys::Array::from(&emit);
        let mut out = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            if let Some(s) = v.as_string() {
                out.push(s);
            }
        }
        if out.is_empty() {
            return Ok(vec!["wasm".to_string()]);
        }
        return Ok(out);
    }
    Err(JsValue::from_str("emit must be a string or an array of strings"))
}

fn line_col_of(source: &str, byte_pos: u32) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut seen = 0u32;
    for ch in source.chars() {
        if seen >= byte_pos {
            break;
        }
        let len = ch.len_utf8() as u32;
        if seen + len > byte_pos {
            break;
        }
        seen += len;
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn span_to_js(source: &str, span: Span) -> JsValue {
    let obj = js_sys::Object::new();
    let (start_line, start_col) = line_col_of(source, span.start);
    let (end_line, end_col) = line_col_of(source, span.end);
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("file_id"),
        &JsValue::from_f64(span.file_id.0 as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("start"),
        &JsValue::from_f64(span.start as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("end"),
        &JsValue::from_f64(span.end as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("start_line"),
        &JsValue::from_f64(start_line as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("start_col"),
        &JsValue::from_f64(start_col as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("end_line"),
        &JsValue::from_f64(end_line as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("end_col"),
        &JsValue::from_f64(end_col as f64),
    );
    obj.into()
}

fn token_kind_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Indent => "Indent",
        TokenKind::Dedent => "Dedent",
        TokenKind::Newline => "Newline",
        TokenKind::Eof => "Eof",
        TokenKind::Colon => "Colon",
        TokenKind::Semicolon => "Semicolon",
        TokenKind::Pipe => "Pipe",
        TokenKind::LParen => "LParen",
        TokenKind::RParen => "RParen",
        TokenKind::Comma => "Comma",
        TokenKind::LAngle => "LAngle",
        TokenKind::RAngle => "RAngle",
        TokenKind::Arrow(_) => "Arrow",
        TokenKind::PathSep => "PathSep",
        TokenKind::At => "At",
        TokenKind::Dot => "Dot",
        TokenKind::Ampersand => "Ampersand",
        TokenKind::Star => "Star",
        TokenKind::Minus => "Minus",
        TokenKind::Equals => "Equals",
        TokenKind::Ident(_) => "Ident",
        TokenKind::IntLiteral(_) => "IntLiteral",
        TokenKind::FloatLiteral(_) => "FloatLiteral",
        TokenKind::BoolLiteral(_) => "BoolLiteral",
        TokenKind::StringLiteral(_) => "StringLiteral",
        TokenKind::UnitLiteral => "UnitLiteral",
        TokenKind::KwFn => "KwFn",
        TokenKind::KwLet => "KwLet",
        TokenKind::KwMut => "KwMut",
        TokenKind::KwNoShadow => "KwNoShadow",
        TokenKind::KwSet => "KwSet",
        TokenKind::KwIf => "KwIf",
        TokenKind::KwWhile => "KwWhile",
        TokenKind::KwCond => "KwCond",
        TokenKind::KwThen => "KwThen",
        TokenKind::KwElse => "KwElse",
        TokenKind::KwDo => "KwDo",
        TokenKind::KwStruct => "KwStruct",
        TokenKind::KwEnum => "KwEnum",
        TokenKind::KwMatch => "KwMatch",
        TokenKind::KwTrait => "KwTrait",
        TokenKind::KwImpl => "KwImpl",
        TokenKind::KwFor => "KwFor",
        TokenKind::KwPub => "KwPub",
        TokenKind::KwBlock => "KwBlock",
        TokenKind::KwTuple => "KwTuple",
        TokenKind::KwMlstr => "KwMlstr",
        TokenKind::DirEntry(_) => "DirEntry",
        TokenKind::DirTarget(_) => "DirTarget",
        TokenKind::DirImport(_) => "DirImport",
        TokenKind::DirUse(_) => "DirUse",
        TokenKind::DirIfTarget(_) => "DirIfTarget",
        TokenKind::DirIfProfile(_) => "DirIfProfile",
        TokenKind::DirWasm => "DirWasm",
        TokenKind::DirIndentWidth(_) => "DirIndentWidth",
        TokenKind::DirInclude(_) => "DirInclude",
        TokenKind::DirExtern { .. } => "DirExtern",
        TokenKind::DirIntrinsic => "DirIntrinsic",
        TokenKind::DirPrelude(_) => "DirPrelude",
        TokenKind::DirNoPrelude => "DirNoPrelude",
        TokenKind::WasmText(_) => "WasmText",
        TokenKind::MlstrLine(_) => "MlstrLine",
    }
}

fn token_extra(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Arrow(e) => Some(format!("{:?}", e)),
        TokenKind::Ident(v)
        | TokenKind::IntLiteral(v)
        | TokenKind::FloatLiteral(v)
        | TokenKind::StringLiteral(v)
        | TokenKind::DirEntry(v)
        | TokenKind::DirTarget(v)
        | TokenKind::DirImport(v)
        | TokenKind::DirUse(v)
        | TokenKind::DirIfTarget(v)
        | TokenKind::DirIfProfile(v)
        | TokenKind::DirInclude(v)
        | TokenKind::DirPrelude(v)
        | TokenKind::WasmText(v)
        | TokenKind::MlstrLine(v) => Some(v.clone()),
        TokenKind::BoolLiteral(v) => Some(v.to_string()),
        TokenKind::DirIndentWidth(v) => Some(v.to_string()),
        TokenKind::DirExtern {
            module,
            name,
            func,
            signature,
        } => Some(format!(
            "module={module}, name={name}, func={func}, signature={signature}"
        )),
        _ => None,
    }
}

fn token_to_js(source: &str, token: &Token) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("kind"),
        &JsValue::from_str(token_kind_name(&token.kind)),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("debug"),
        &JsValue::from_str(&format!("{:?}", token.kind)),
    );
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, token.span));
    if let Some(extra) = token_extra(&token.kind) {
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("value"),
            &JsValue::from_str(&extra),
        );
    }
    obj.into()
}

fn diagnostics_to_js(source: &str, diagnostics: &[Diagnostic]) -> JsValue {
    let arr = js_sys::Array::new();
    for d in diagnostics {
        let obj = js_sys::Object::new();
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("severity"),
            &JsValue::from_str(severity),
        );
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("code"),
            &d.code
                .map(JsValue::from_str)
                .unwrap_or(JsValue::NULL),
        );
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("message"),
            &JsValue::from_str(&d.message),
        );
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("primary"),
            &span_to_js(source, d.primary.span),
        );

        let secondary = js_sys::Array::new();
        for s in &d.secondary {
            let sub = js_sys::Object::new();
            let _ = Reflect::set(
                &sub,
                &JsValue::from_str("span"),
                &span_to_js(source, s.span),
            );
            let _ = Reflect::set(
                &sub,
                &JsValue::from_str("message"),
                &s.message
                    .as_ref()
                    .map(|m| JsValue::from_str(m))
                    .unwrap_or(JsValue::NULL),
            );
            secondary.push(&sub);
        }
        let _ = Reflect::set(&obj, &JsValue::from_str("secondary"), &secondary);
        arr.push(&obj);
    }
    arr.into()
}

fn expr_to_js(source: &str, expr: &PrefixExpr) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("PrefixExpr"));
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, expr.span));
    let items = js_sys::Array::new();
    for item in &expr.items {
        items.push(&prefix_item_to_js(source, item));
    }
    let _ = Reflect::set(&obj, &JsValue::from_str("items"), &items);
    obj.into()
}

fn block_to_js(source: &str, block: &Block) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Block"));
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, block.span));
    let items = js_sys::Array::new();
    for stmt in &block.items {
        items.push(&stmt_to_js(source, stmt));
    }
    let _ = Reflect::set(&obj, &JsValue::from_str("items"), &items);
    obj.into()
}

fn directive_name(d: &Directive) -> &'static str {
    match d {
        Directive::Entry { .. } => "Entry",
        Directive::Target { .. } => "Target",
        Directive::Import { .. } => "Import",
        Directive::Use { .. } => "Use",
        Directive::IfTarget { .. } => "IfTarget",
        Directive::IfProfile { .. } => "IfProfile",
        Directive::IndentWidth { .. } => "IndentWidth",
        Directive::Extern { .. } => "Extern",
        Directive::Include { .. } => "Include",
        Directive::Prelude { .. } => "Prelude",
        Directive::NoPrelude { .. } => "NoPrelude",
    }
}

fn stmt_to_js(source: &str, stmt: &Stmt) -> JsValue {
    let obj = js_sys::Object::new();
    match stmt {
        Stmt::Directive(d) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("Directive"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(directive_name(d)),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", d)),
            );
        }
        Stmt::FnDef(def) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("FnDef"));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name.name),
            );
            match &def.body {
                FnBody::Parsed(block) => {
                    let _ = Reflect::set(&obj, &JsValue::from_str("body"), &block_to_js(source, block));
                }
                FnBody::Wasm(block) => {
                    let _ = Reflect::set(
                        &obj,
                        &JsValue::from_str("body"),
                        &JsValue::from_str(&format!("{:?}", block)),
                    );
                }
            }
        }
        Stmt::FnAlias(alias) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("FnAlias"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&alias.name.name),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("target"),
                &JsValue::from_str(&alias.target.name),
            );
        }
        Stmt::StructDef(def) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("StructDef"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name.name),
            );
        }
        Stmt::EnumDef(def) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("EnumDef"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name.name),
            );
        }
        Stmt::Wasm(block) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Wasm"));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", block)),
            );
        }
        Stmt::Trait(def) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Trait"));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name.name),
            );
        }
        Stmt::Impl(def) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Impl"));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", def)),
            );
        }
        Stmt::Expr(expr) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Expr"));
            let _ = Reflect::set(&obj, &JsValue::from_str("expr"), &expr_to_js(source, expr));
        }
        Stmt::ExprSemi(expr, span) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("ExprSemi"),
            );
            let _ = Reflect::set(&obj, &JsValue::from_str("expr"), &expr_to_js(source, expr));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("semi_span"),
                &span
                    .map(|s| span_to_js(source, s))
                    .unwrap_or(JsValue::NULL),
            );
        }
    }
    obj.into()
}

fn prefix_item_to_js(source: &str, item: &PrefixItem) -> JsValue {
    let obj = js_sys::Object::new();
    match item {
        PrefixItem::Symbol(sym) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("Symbol"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", sym)),
            );
        }
        PrefixItem::Literal(lit, span) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("Literal"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", lit)),
            );
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
        }
        PrefixItem::TypeAnnotation(ty, span) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("TypeAnnotation"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", ty)),
            );
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
        }
        PrefixItem::Block(block, span) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Block"));
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
            let _ = Reflect::set(&obj, &JsValue::from_str("block"), &block_to_js(source, block));
        }
        PrefixItem::Match(m, span) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Match"));
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", m)),
            );
        }
        PrefixItem::Pipe(span) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Pipe"));
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
        }
        PrefixItem::Tuple(values, span) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Tuple"));
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
            let arr = js_sys::Array::new();
            for e in values {
                arr.push(&expr_to_js(source, e));
            }
            let _ = Reflect::set(&obj, &JsValue::from_str("items"), &arr);
        }
        PrefixItem::Group(expr, span) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("Group"));
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
            let _ = Reflect::set(&obj, &JsValue::from_str("expr"), &expr_to_js(source, expr));
        }
        PrefixItem::Intrinsic(expr, span) => {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("Intrinsic"),
            );
            let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, *span));
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("debug"),
                &JsValue::from_str(&format!("{:?}", expr)),
            );
        }
    }
    obj.into()
}

#[derive(Clone)]
struct NameDefTrace {
    id: usize,
    name: String,
    kind: &'static str,
    span: Span,
    scope_depth: usize,
}

#[derive(Clone)]
struct NameRefTrace {
    name: String,
    span: Span,
    scope_depth: usize,
    resolved_def_id: Option<usize>,
    candidate_def_ids: Vec<usize>,
}

#[derive(Clone)]
struct NameShadowTrace {
    name: String,
    event_kind: &'static str,
    span: Span,
    scope_depth: usize,
    selected_def_id: Option<usize>,
    shadowed_def_ids: Vec<usize>,
    severity: &'static str,
    message: String,
}

#[derive(Clone)]
struct SemanticExprTrace {
    id: usize,
    function_name: String,
    kind: &'static str,
    span: Span,
    ty: String,
    parent_id: Option<usize>,
    arg_spans: Vec<Span>,
}

#[derive(Clone)]
struct SemanticTokenTrace {
    token_index: usize,
    inferred_expr_id: Option<usize>,
    inferred_type: Option<String>,
    expr_span: Option<Span>,
    arg_index: Option<usize>,
    arg_span: Option<Span>,
}

#[derive(Default)]
struct NameResolutionTrace {
    defs: Vec<NameDefTrace>,
    refs: Vec<NameRefTrace>,
    shadows: Vec<NameShadowTrace>,
    scopes: Vec<BTreeMap<String, Vec<usize>>>,
    warn_important_shadow: bool,
}

impl NameResolutionTrace {
    fn new() -> Self {
        Self::new_with_options(true)
    }

    fn new_with_options(warn_important_shadow: bool) -> Self {
        Self {
            defs: Vec::new(),
            refs: Vec::new(),
            shadows: Vec::new(),
            scopes: vec![BTreeMap::new()],
            warn_important_shadow,
        }
    }

    fn current_depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define(&mut self, name: String, kind: &'static str, span: Span) -> usize {
        let existing_candidates = self.lookup_candidates(&name);
        let id = self.defs.len();
        let depth = self.current_depth();
        self.defs.push(NameDefTrace {
            id,
            name: name.clone(),
            kind,
            span,
            scope_depth: depth,
        });

        if !existing_candidates.is_empty() {
            let severity = if self.warn_important_shadow
                && is_important_shadow_symbol(&name)
                && is_variable_def_kind(kind)
            {
                "warning"
            } else {
                "info"
            };
            let message = if severity == "warning" {
                format!(
                    "important symbol '{}' is shadowed by {} definition",
                    name, kind
                )
            } else {
                format!("'{}' shadows an outer definition", name)
            };
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "definition_shadow",
                span,
                scope_depth: depth,
                selected_def_id: Some(id),
                shadowed_def_ids: existing_candidates,
                severity,
                message,
            });
        } else if self.warn_important_shadow
            && is_important_shadow_symbol(&name)
            && is_variable_def_kind(kind)
        {
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "important_name",
                span,
                scope_depth: depth,
                selected_def_id: Some(id),
                shadowed_def_ids: Vec::new(),
                severity: "warning",
                message: format!(
                    "definition '{}' may shadow important stdlib symbol",
                    name
                ),
            });
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.entry(name).or_default().push(id);
        }
        id
    }

    fn lookup_candidates(&self, name: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for scope in self.scopes.iter().rev() {
            if let Some(ids) = scope.get(name) {
                out.extend(ids.iter().rev().copied());
            }
        }
        out
    }

    fn reference(&mut self, name: String, span: Span) {
        let candidates = self.lookup_candidates(&name);
        let resolved = candidates.first().copied();
        if candidates.len() > 1 {
            self.shadows.push(NameShadowTrace {
                name: name.clone(),
                event_kind: "reference_shadow",
                span,
                scope_depth: self.current_depth(),
                selected_def_id: resolved,
                shadowed_def_ids: candidates[1..].to_vec(),
                severity: "info",
                message: format!(
                    "'{}' resolved to nearest definition with {} shadowed candidate(s)",
                    name,
                    candidates.len().saturating_sub(1)
                ),
            });
        }
        self.refs.push(NameRefTrace {
            name,
            span,
            scope_depth: self.current_depth(),
            resolved_def_id: resolved,
            candidate_def_ids: candidates,
        });
    }
}

fn is_layout_marker(name: &str) -> bool {
    matches!(name, "cond" | "then" | "else" | "do" | "block")
}

fn is_important_shadow_symbol(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "print_i32"
            | "println_i32"
            | "add"
            | "sub"
            | "mul"
            | "div"
            | "eq"
            | "lt"
            | "le"
            | "gt"
            | "ge"
            | "map"
            | "len"
    )
}

fn is_variable_def_kind(kind: &str) -> bool {
    matches!(kind, "let_hoisted" | "let_mut" | "param" | "match_bind")
}

fn hoist_block_defs(trace: &mut NameResolutionTrace, block: &Block) {
    for stmt in &block.items {
        match stmt {
            Stmt::FnDef(def) => {
                trace.define(def.name.name.clone(), "fn", def.name.span);
            }
            Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                if let Some(PrefixItem::Symbol(Symbol::Let { name, mutable, .. })) = expr.items.first() {
                    if !*mutable {
                        trace.define(name.name.clone(), "let_hoisted", name.span);
                    }
                }
            }
            _ => {}
        }
    }
}

fn trace_match_arm(trace: &mut NameResolutionTrace, arm: &MatchArm) {
    trace.push_scope();
    if let Some(bind) = &arm.bind {
        trace.define(bind.name.clone(), "match_bind", bind.span);
    }
    trace_block(trace, &arm.body);
    trace.pop_scope();
}

fn trace_prefix_expr(trace: &mut NameResolutionTrace, expr: &PrefixExpr) {
    for (idx, item) in expr.items.iter().enumerate() {
        match item {
            PrefixItem::Symbol(Symbol::Let { name, mutable, .. }) => {
                if *mutable {
                    trace.define(name.name.clone(), "let_mut", name.span);
                }
                if idx != 0 {
                    trace.reference(name.name.clone(), name.span);
                }
            }
            PrefixItem::Symbol(Symbol::Set { name }) => {
                trace.reference(name.name.clone(), name.span);
            }
            PrefixItem::Symbol(Symbol::Ident(id, _, _)) => {
                if !is_layout_marker(&id.name) {
                    trace.reference(id.name.clone(), id.span);
                }
            }
            PrefixItem::Block(block, _) => {
                trace.push_scope();
                trace_block(trace, block);
                trace.pop_scope();
            }
            PrefixItem::Match(m, _) => {
                trace_prefix_expr(trace, &m.scrutinee);
                for arm in &m.arms {
                    trace_match_arm(trace, arm);
                }
            }
            PrefixItem::Tuple(items, _) => {
                for item_expr in items {
                    trace_prefix_expr(trace, item_expr);
                }
            }
            PrefixItem::Group(inner, _) => {
                trace_prefix_expr(trace, inner);
            }
            PrefixItem::Intrinsic(intr, _) => {
                for arg in &intr.args {
                    trace_prefix_expr(trace, arg);
                }
            }
            PrefixItem::Literal(_, _) | PrefixItem::TypeAnnotation(_, _) | PrefixItem::Pipe(_) => {}
            PrefixItem::Symbol(Symbol::If(_))
            | PrefixItem::Symbol(Symbol::While(_))
            | PrefixItem::Symbol(Symbol::AddrOf(_))
            | PrefixItem::Symbol(Symbol::Deref(_)) => {}
        }
    }
}

fn trace_stmt(trace: &mut NameResolutionTrace, stmt: &Stmt) {
    match stmt {
        Stmt::FnDef(def) => match &def.body {
            FnBody::Parsed(body) => {
                trace.push_scope();
                for param in &def.params {
                    trace.define(param.name.clone(), "param", param.span);
                }
                trace_block(trace, body);
                trace.pop_scope();
            }
            FnBody::Wasm(_) => {}
        },
        Stmt::FnAlias(alias) => {
            trace.reference(alias.target.name.clone(), alias.target.span);
            trace.define(alias.name.name.clone(), "fn_alias", alias.name.span);
        }
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
            trace_prefix_expr(trace, expr);
        }
        _ => {}
    }
}

fn trace_block(trace: &mut NameResolutionTrace, block: &Block) {
    hoist_block_defs(trace, block);
    for stmt in &block.items {
        trace_stmt(trace, stmt);
    }
}

fn def_trace_to_js(source: &str, def: &NameDefTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(def.id as f64));
    let _ = Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&def.name));
    let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str(def.kind));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("scope_depth"),
        &JsValue::from_f64(def.scope_depth as f64),
    );
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, def.span));
    obj.into()
}

fn ref_trace_to_js(source: &str, rf: &NameRefTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("scope_depth"),
        &JsValue::from_f64(rf.scope_depth as f64),
    );
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, rf.span));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("resolved_def_id"),
        &rf.resolved_def_id
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    let cand = js_sys::Array::new();
    for id in &rf.candidate_def_ids {
        cand.push(&JsValue::from_f64(*id as f64));
    }
    let _ = Reflect::set(&obj, &JsValue::from_str("candidate_def_ids"), &cand);
    obj.into()
}

fn shadow_trace_to_js(source: &str, sh: &NameShadowTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&sh.name));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("event_kind"),
        &JsValue::from_str(sh.event_kind),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("scope_depth"),
        &JsValue::from_f64(sh.scope_depth as f64),
    );
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, sh.span));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("selected_def_id"),
        &sh.selected_def_id
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    let hidden = js_sys::Array::new();
    for id in &sh.shadowed_def_ids {
        hidden.push(&JsValue::from_f64(*id as f64));
    }
    let _ = Reflect::set(&obj, &JsValue::from_str("shadowed_def_ids"), &hidden);
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("severity"),
        &JsValue::from_str(sh.severity),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("message"),
        &JsValue::from_str(&sh.message),
    );
    obj.into()
}

fn name_resolution_payload_to_js(source: &str, trace: &NameResolutionTrace) -> JsValue {
    let payload = js_sys::Object::new();

    let defs = js_sys::Array::new();
    for def in &trace.defs {
        defs.push(&def_trace_to_js(source, def));
    }
    let refs = js_sys::Array::new();
    for rf in &trace.refs {
        refs.push(&ref_trace_to_js(source, rf));
    }
    let shadows = js_sys::Array::new();
    for sh in &trace.shadows {
        shadows.push(&shadow_trace_to_js(source, sh));
    }
    let shadow_diagnostics = js_sys::Array::new();
    for sh in &trace.shadows {
        if matches!(sh.severity, "warning" | "info") {
            shadow_diagnostics.push(&shadow_trace_to_js(source, sh));
        }
    }

    let by_name = js_sys::Object::new();
    let mut names = BTreeMap::<String, (Vec<usize>, Vec<usize>)>::new();
    for d in &trace.defs {
        names.entry(d.name.clone()).or_default().0.push(d.id);
    }
    for (idx, r) in trace.refs.iter().enumerate() {
        names.entry(r.name.clone()).or_default().1.push(idx);
    }
    for (name, (def_ids, ref_ids)) in names {
        let name_obj = js_sys::Object::new();
        let d_arr = js_sys::Array::new();
        for id in def_ids {
            d_arr.push(&JsValue::from_f64(id as f64));
        }
        let r_arr = js_sys::Array::new();
        for id in ref_ids {
            r_arr.push(&JsValue::from_f64(id as f64));
        }
        let _ = Reflect::set(&name_obj, &JsValue::from_str("definitions"), &d_arr);
        let _ = Reflect::set(&name_obj, &JsValue::from_str("references"), &r_arr);
        let _ = Reflect::set(&by_name, &JsValue::from_str(&name), &name_obj);
    }

    let policy = js_sys::Object::new();
    let _ = Reflect::set(
        &policy,
        &JsValue::from_str("selection"),
        &JsValue::from_str("nearest_scope_first"),
    );
    let _ = Reflect::set(
        &policy,
        &JsValue::from_str("hoist"),
        &JsValue::from_str("fn and non-mut let"),
    );
    let _ = Reflect::set(
        &policy,
        &JsValue::from_str("warn_important_shadow"),
        &JsValue::from_bool(trace.warn_important_shadow),
    );

    let _ = Reflect::set(&payload, &JsValue::from_str("definitions"), &defs);
    let _ = Reflect::set(&payload, &JsValue::from_str("references"), &refs);
    let _ = Reflect::set(&payload, &JsValue::from_str("shadows"), &shadows);
    let _ = Reflect::set(
        &payload,
        &JsValue::from_str("shadow_diagnostics"),
        &shadow_diagnostics,
    );
    let _ = Reflect::set(&payload, &JsValue::from_str("by_name"), &by_name);
    let _ = Reflect::set(&payload, &JsValue::from_str("policy"), &policy);
    payload.into()
}

fn semantic_expr_to_js(source: &str, se: &SemanticExprTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(se.id as f64));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("function_name"),
        &JsValue::from_str(&se.function_name),
    );
    let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str(se.kind));
    let _ = Reflect::set(&obj, &JsValue::from_str("span"), &span_to_js(source, se.span));
    let _ = Reflect::set(&obj, &JsValue::from_str("inferred_type"), &JsValue::from_str(&se.ty));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("parent_id"),
        &se.parent_id
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    let arg_arr = js_sys::Array::new();
    for sp in &se.arg_spans {
        arg_arr.push(&span_to_js(source, *sp));
    }
    let _ = Reflect::set(&obj, &JsValue::from_str("argument_ranges"), &arg_arr);
    obj.into()
}

fn semantic_token_to_js(source: &str, st: &SemanticTokenTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("token_index"),
        &JsValue::from_f64(st.token_index as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("inferred_expr_id"),
        &st.inferred_expr_id
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("inferred_type"),
        &st.inferred_type
            .as_ref()
            .map(|s| JsValue::from_str(s))
            .unwrap_or(JsValue::NULL),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("expression_range"),
        &st.expr_span
            .map(|s| span_to_js(source, s))
            .unwrap_or(JsValue::NULL),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("arg_index"),
        &st.arg_index
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("arg_range"),
        &st.arg_span
            .map(|s| span_to_js(source, s))
            .unwrap_or(JsValue::NULL),
    );
    obj.into()
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.file_id == inner.file_id && outer.start <= inner.start && inner.end <= outer.end
}

fn span_width(span: Span) -> usize {
    span.end.saturating_sub(span.start) as usize
}

fn hir_kind_name(kind: &HirExprKind) -> &'static str {
    match kind {
        HirExprKind::LiteralI32(_) => "LiteralI32",
        HirExprKind::LiteralF32(_) => "LiteralF32",
        HirExprKind::LiteralBool(_) => "LiteralBool",
        HirExprKind::LiteralStr(_) => "LiteralStr",
        HirExprKind::Unit => "Unit",
        HirExprKind::Var(_) => "Var",
        HirExprKind::Call { .. } => "Call",
        HirExprKind::CallIndirect { .. } => "CallIndirect",
        HirExprKind::If { .. } => "If",
        HirExprKind::While { .. } => "While",
        HirExprKind::Match { .. } => "Match",
        HirExprKind::EnumConstruct { .. } => "EnumConstruct",
        HirExprKind::StructConstruct { .. } => "StructConstruct",
        HirExprKind::TupleConstruct { .. } => "TupleConstruct",
        HirExprKind::Block(_) => "Block",
        HirExprKind::Let { .. } => "Let",
        HirExprKind::Set { .. } => "Set",
        HirExprKind::Intrinsic { .. } => "Intrinsic",
        HirExprKind::AddrOf(_) => "AddrOf",
        HirExprKind::Deref(_) => "Deref",
        HirExprKind::Drop { .. } => "Drop",
    }
}

fn collect_semantic_expr_from_line(
    line: &HirLine,
    function_name: &str,
    types: &nepl_core::types::TypeCtx,
    out: &mut Vec<SemanticExprTrace>,
) {
    collect_semantic_expr(&line.expr, function_name, types, None, out);
}

fn collect_semantic_expr_from_block(
    block: &HirBlock,
    function_name: &str,
    types: &nepl_core::types::TypeCtx,
    parent_id: Option<usize>,
    out: &mut Vec<SemanticExprTrace>,
) {
    for line in &block.lines {
        collect_semantic_expr(&line.expr, function_name, types, parent_id, out);
    }
}

fn collect_semantic_expr(
    expr: &HirExpr,
    function_name: &str,
    types: &nepl_core::types::TypeCtx,
    parent_id: Option<usize>,
    out: &mut Vec<SemanticExprTrace>,
) -> usize {
    let id = out.len();
    out.push(SemanticExprTrace {
        id,
        function_name: function_name.to_string(),
        kind: hir_kind_name(&expr.kind),
        span: expr.span,
        ty: types.type_to_string(expr.ty),
        parent_id,
        arg_spans: Vec::new(),
    });

    let mut arg_spans = Vec::new();
    match &expr.kind {
        HirExprKind::Call { args, .. } => {
            for a in args {
                arg_spans.push(a.span);
                collect_semantic_expr(a, function_name, types, Some(id), out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_semantic_expr(callee, function_name, types, Some(id), out);
            for a in args {
                arg_spans.push(a.span);
                collect_semantic_expr(a, function_name, types, Some(id), out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            arg_spans.push(cond.span);
            arg_spans.push(then_branch.span);
            arg_spans.push(else_branch.span);
            collect_semantic_expr(cond, function_name, types, Some(id), out);
            collect_semantic_expr(then_branch, function_name, types, Some(id), out);
            collect_semantic_expr(else_branch, function_name, types, Some(id), out);
        }
        HirExprKind::While { cond, body } => {
            arg_spans.push(cond.span);
            arg_spans.push(body.span);
            collect_semantic_expr(cond, function_name, types, Some(id), out);
            collect_semantic_expr(body, function_name, types, Some(id), out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            arg_spans.push(scrutinee.span);
            collect_semantic_expr(scrutinee, function_name, types, Some(id), out);
            for arm in arms {
                arg_spans.push(arm.body.span);
                collect_semantic_expr(&arm.body, function_name, types, Some(id), out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                arg_spans.push(p.span);
                collect_semantic_expr(p, function_name, types, Some(id), out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                arg_spans.push(f.span);
                collect_semantic_expr(f, function_name, types, Some(id), out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for e in items {
                arg_spans.push(e.span);
                collect_semantic_expr(e, function_name, types, Some(id), out);
            }
        }
        HirExprKind::Block(b) => {
            collect_semantic_expr_from_block(b, function_name, types, Some(id), out);
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            arg_spans.push(value.span);
            collect_semantic_expr(value, function_name, types, Some(id), out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                arg_spans.push(a.span);
                collect_semantic_expr(a, function_name, types, Some(id), out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            arg_spans.push(inner.span);
            collect_semantic_expr(inner, function_name, types, Some(id), out);
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::Drop { .. } => {}
    }

    out[id].arg_spans = arg_spans;
    id
}

fn resolve_target_for_analysis(module: &nepl_core::ast::Module) -> (CompileTarget, Vec<Diagnostic>) {
    let mut found: Option<(CompileTarget, Span)> = None;
    let mut diags = Vec::new();

    for d in &module.directives {
        if let Directive::Target { target, span } = d {
            let parsed = match target.as_str() {
                "wasm" => Some(CompileTarget::Wasm),
                "wasi" => Some(CompileTarget::Wasi),
                _ => None,
            };
            if let Some(t) = parsed {
                if let Some((_, prev_span)) = found {
                    diags.push(
                        Diagnostic::error("multiple #target directives are not allowed", *span)
                            .with_secondary_label(prev_span, Some("previous #target here".into())),
                    );
                } else {
                    found = Some((t, *span));
                }
            } else {
                diags.push(Diagnostic::error("unknown target in #target", *span));
            }
        }
    }

    if found.is_none() {
        for it in &module.root.items {
            if let Stmt::Directive(Directive::Target { target, span }) = it {
                let parsed = match target.as_str() {
                    "wasm" => Some(CompileTarget::Wasm),
                    "wasi" => Some(CompileTarget::Wasi),
                    _ => None,
                };
                if let Some(t) = parsed {
                    if let Some((_, prev_span)) = found {
                        diags.push(
                            Diagnostic::error("multiple #target directives are not allowed", *span)
                                .with_secondary_label(
                                    prev_span,
                                    Some("previous #target here".into()),
                                ),
                        );
                    } else {
                        found = Some((t, *span));
                    }
                } else {
                    diags.push(Diagnostic::error("unknown target in #target", *span));
                }
            }
        }
    }

    (found.map(|(t, _)| t).unwrap_or(CompileTarget::Wasm), diags)
}

/// 入力ソースを字句解析し、token 列と診断を JSON で返します。
///
/// VSCode 拡張や LSP 実装で、構文解析前の結果を可視化するための API です。
#[wasm_bindgen]
pub fn analyze_lex(source: &str) -> JsValue {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let out = js_sys::Object::new();
    let token_arr = js_sys::Array::new();
    for token in &lex_result.tokens {
        token_arr.push(&token_to_js(source, token));
    }
    let diagnostics = diagnostics_to_js(source, &lex_result.diagnostics);
    let has_error = lex_result
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error);
    let _ = Reflect::set(&out, &JsValue::from_str("stage"), &JsValue::from_str("lex"));
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("ok"),
        &JsValue::from_bool(!has_error),
    );
    let _ = Reflect::set(&out, &JsValue::from_str("tokens"), &token_arr);
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("indent_width"),
        &JsValue::from_f64(lex_result.indent_width as f64),
    );
    let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);
    out.into()
}

/// 入力ソースを構文解析し、token・AST 木構造・診断を JSON で返します。
///
/// lexer/parser の結果確認や、エディタ拡張での構文可視化に利用します。
#[wasm_bindgen]
pub fn analyze_parse(source: &str) -> JsValue {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let token_arr = js_sys::Array::new();
    for token in &lex_result.tokens {
        token_arr.push(&token_to_js(source, token));
    }
    let lex_diagnostics = diagnostics_to_js(source, &lex_result.diagnostics);
    let parse_result = parse_tokens(file_id, lex_result);
    let diagnostics = diagnostics_to_js(source, &parse_result.diagnostics);

    let out = js_sys::Object::new();
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("stage"),
        &JsValue::from_str("parse"),
    );
    let _ = Reflect::set(&out, &JsValue::from_str("tokens"), &token_arr);
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("lex_diagnostics"),
        &lex_diagnostics,
    );
    let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);

    if let Some(module) = parse_result.module {
        let module_obj = js_sys::Object::new();
        let _ = Reflect::set(
            &module_obj,
            &JsValue::from_str("indent_width"),
            &JsValue::from_f64(module.indent_width as f64),
        );
        let _ = Reflect::set(
            &module_obj,
            &JsValue::from_str("directives_count"),
            &JsValue::from_f64(module.directives.len() as f64),
        );
        let _ = Reflect::set(
            &module_obj,
            &JsValue::from_str("root"),
            &block_to_js(source, &module.root),
        );
        let _ = Reflect::set(
            &module_obj,
            &JsValue::from_str("debug"),
            &JsValue::from_str(&format!("{:#?}", module)),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(true));
        let _ = Reflect::set(&out, &JsValue::from_str("module"), &module_obj);
    } else {
        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
        let _ = Reflect::set(&out, &JsValue::from_str("module"), &JsValue::NULL);
    }

    out.into()
}

/// 同名識別子の解決結果を、LSP/エディタ向けに返します。
///
/// - `definitions`: 解析で見つかった定義点
/// - `references`: 各参照点の候補と最終選択（最内側優先）
/// - 巻き上げは現行仕様に合わせて `fn` と `let`(non-mut) を先行登録します
#[wasm_bindgen]
pub fn analyze_name_resolution(source: &str) -> JsValue {
    analyze_name_resolution_with_options(source, JsValue::UNDEFINED)
}

#[wasm_bindgen]
pub fn analyze_name_resolution_with_options(source: &str, options: JsValue) -> JsValue {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let parse_result = parse_tokens(file_id, lex_result);
    let diagnostics = diagnostics_to_js(source, &parse_result.diagnostics);
    let warn_important_shadow = Reflect::get(&options, &JsValue::from_str("warn_important_shadow"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let out = js_sys::Object::new();
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("stage"),
        &JsValue::from_str("name_resolution"),
    );
    let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);

    if let Some(module) = parse_result.module {
        let mut trace = NameResolutionTrace::new_with_options(warn_important_shadow);
        trace_block(&mut trace, &module.root);
        let payload = name_resolution_payload_to_js(source, &trace);
        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(true));
        let _ = Reflect::set(&out, &JsValue::from_str("definitions"), &Reflect::get(&payload, &JsValue::from_str("definitions")).unwrap_or(JsValue::NULL));
        let _ = Reflect::set(&out, &JsValue::from_str("references"), &Reflect::get(&payload, &JsValue::from_str("references")).unwrap_or(JsValue::NULL));
        let _ = Reflect::set(&out, &JsValue::from_str("shadows"), &Reflect::get(&payload, &JsValue::from_str("shadows")).unwrap_or(JsValue::NULL));
        let _ = Reflect::set(
            &out,
            &JsValue::from_str("shadow_diagnostics"),
            &Reflect::get(&payload, &JsValue::from_str("shadow_diagnostics")).unwrap_or(JsValue::NULL),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("by_name"), &Reflect::get(&payload, &JsValue::from_str("by_name")).unwrap_or(JsValue::NULL));
        let _ = Reflect::set(&out, &JsValue::from_str("policy"), &Reflect::get(&payload, &JsValue::from_str("policy")).unwrap_or(JsValue::NULL));
    } else {
        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
        let _ = Reflect::set(&out, &JsValue::from_str("definitions"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("references"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("shadows"), &js_sys::Array::new());
        let _ = Reflect::set(
            &out,
            &JsValue::from_str("shadow_diagnostics"),
            &js_sys::Array::new(),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("by_name"), &js_sys::Object::new());
        let policy = js_sys::Object::new();
        let _ = Reflect::set(
            &policy,
            &JsValue::from_str("warn_important_shadow"),
            &JsValue::from_bool(warn_important_shadow),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("policy"), &policy);
    }

    out.into()
}

/// 字句・構文・型検査の情報を統合し、LSP 向けの詳細解析結果を返します。
///
/// 返却する主な情報:
/// - `expressions`: 各式の範囲・推論型・親子関係・引数範囲
/// - `token_semantics`: token ごとの対応式と推論型、引数位置情報
/// - `functions`: 関数定義の範囲とシグネチャ
#[wasm_bindgen]
pub fn analyze_semantics(source: &str) -> JsValue {
    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let tokens = lex_result.tokens.clone();
    let token_arr = js_sys::Array::new();
    for token in &tokens {
        token_arr.push(&token_to_js(source, token));
    }

    let parse_result = parse_tokens(file_id, lex_result);
    let out = js_sys::Object::new();
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("stage"),
        &JsValue::from_str("semantics"),
    );
    let _ = Reflect::set(&out, &JsValue::from_str("tokens"), &token_arr);

    let mut all_diags = parse_result.diagnostics.clone();
    let mut has_error = all_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));

    if let Some(module) = &parse_result.module {
        let mut resolve_trace = NameResolutionTrace::new();
        trace_block(&mut resolve_trace, &module.root);
        let resolve_payload = name_resolution_payload_to_js(source, &resolve_trace);

        let (target, mut target_diags) = resolve_target_for_analysis(module);
        has_error |= target_diags
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        all_diags.append(&mut target_diags);

        let tc = typecheck(module, target, BuildProfile::Debug);
        has_error |= tc
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        all_diags.extend(tc.diagnostics.clone());

        let diagnostics = diagnostics_to_js(source, &all_diags);
        let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);

        if let Some(hir_module) = tc.module {
            let mut exprs = Vec::<SemanticExprTrace>::new();
            let function_arr = js_sys::Array::new();
            for f in &hir_module.functions {
                let f_obj = js_sys::Object::new();
                let _ = Reflect::set(&f_obj, &JsValue::from_str("name"), &JsValue::from_str(&f.name));
                let _ = Reflect::set(&f_obj, &JsValue::from_str("span"), &span_to_js(source, f.span));
                let _ = Reflect::set(
                    &f_obj,
                    &JsValue::from_str("signature"),
                    &JsValue::from_str(&tc.types.type_to_string(f.func_ty)),
                );
                function_arr.push(&f_obj);
                if let nepl_core::hir::HirBody::Block(b) = &f.body {
                    for line in &b.lines {
                        collect_semantic_expr_from_line(line, &f.name, &tc.types, &mut exprs);
                    }
                }
            }

            let expr_arr = js_sys::Array::new();
            for ex in &exprs {
                expr_arr.push(&semantic_expr_to_js(source, ex));
            }

            let token_res_arr = js_sys::Array::new();
            for (tok_idx, token) in tokens.iter().enumerate() {
                let mut best_ref: Option<&NameRefTrace> = None;
                for rf in &resolve_trace.refs {
                    if span_contains(rf.span, token.span) {
                        if let Some(prev) = best_ref {
                            if span_width(rf.span) < span_width(prev.span) {
                                best_ref = Some(rf);
                            }
                        } else {
                            best_ref = Some(rf);
                        }
                    }
                }
                let item = js_sys::Object::new();
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("token_index"),
                    &JsValue::from_f64(tok_idx as f64),
                );
                if let Some(rf) = best_ref {
                    let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
                    let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &span_to_js(source, rf.span));
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_def_id"),
                        &rf.resolved_def_id
                            .map(|v| JsValue::from_f64(v as f64))
                            .unwrap_or(JsValue::NULL),
                    );
                    let cand = js_sys::Array::new();
                    for id in &rf.candidate_def_ids {
                        cand.push(&JsValue::from_f64(*id as f64));
                    }
                    let _ = Reflect::set(&item, &JsValue::from_str("candidate_def_ids"), &cand);
                } else {
                    let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::NULL);
                    let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &JsValue::NULL);
                    let _ = Reflect::set(&item, &JsValue::from_str("resolved_def_id"), &JsValue::NULL);
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("candidate_def_ids"),
                        &js_sys::Array::new(),
                    );
                }
                token_res_arr.push(&item);
            }

            let mut token_semantics = Vec::<SemanticTokenTrace>::new();
            for (tok_idx, token) in tokens.iter().enumerate() {
                let mut best_expr: Option<&SemanticExprTrace> = None;
                for ex in &exprs {
                    if span_contains(ex.span, token.span) {
                        if let Some(prev) = best_expr {
                            if span_width(ex.span) < span_width(prev.span) {
                                best_expr = Some(ex);
                            }
                        } else {
                            best_expr = Some(ex);
                        }
                    }
                }
                let mut arg_hit: Option<(usize, Span)> = None;
                for ex in &exprs {
                    for (a_idx, a_sp) in ex.arg_spans.iter().enumerate() {
                        if span_contains(*a_sp, token.span) {
                            if let Some((_, prev_sp)) = arg_hit {
                                if span_width(*a_sp) < span_width(prev_sp) {
                                    arg_hit = Some((a_idx, *a_sp));
                                }
                            } else {
                                arg_hit = Some((a_idx, *a_sp));
                            }
                        }
                    }
                }
                token_semantics.push(SemanticTokenTrace {
                    token_index: tok_idx,
                    inferred_expr_id: best_expr.map(|x| x.id),
                    inferred_type: best_expr.map(|x| x.ty.clone()),
                    expr_span: best_expr.map(|x| x.span),
                    arg_index: arg_hit.map(|(idx, _)| idx),
                    arg_span: arg_hit.map(|(_, sp)| sp),
                });
            }
            let token_sem_arr = js_sys::Array::new();
            for ts in &token_semantics {
                token_sem_arr.push(&semantic_token_to_js(source, ts));
            }

            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(!has_error));
            let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &expr_arr);
            let _ = Reflect::set(&out, &JsValue::from_str("token_semantics"), &token_sem_arr);
            let _ = Reflect::set(&out, &JsValue::from_str("functions"), &function_arr);
            let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &resolve_payload);
            let _ = Reflect::set(&out, &JsValue::from_str("token_resolution"), &token_res_arr);
        } else {
            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
            let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &js_sys::Array::new());
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("token_semantics"),
                &js_sys::Array::new(),
            );
            let _ = Reflect::set(&out, &JsValue::from_str("functions"), &js_sys::Array::new());
            let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &resolve_payload);
            let _ = Reflect::set(&out, &JsValue::from_str("token_resolution"), &js_sys::Array::new());
        }
    } else {
        let diagnostics = diagnostics_to_js(source, &all_diags);
        let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);
        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
        let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &js_sys::Array::new());
        let _ = Reflect::set(
            &out,
            &JsValue::from_str("token_semantics"),
            &js_sys::Array::new(),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("functions"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &JsValue::NULL);
        let _ = Reflect::set(&out, &JsValue::from_str("token_resolution"), &js_sys::Array::new());
    }

    out.into()
}

fn compile_outputs_impl(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    emit: JsValue,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
    // 1) wasm を生成
    let compiled = compile_wasm_with_entry(entry_path, source, vfs)
        .map_err(|msg| JsValue::from_str(&msg))?;

    // 2) 依頼された形式に応じて結果を詰める
    let emit_list = parse_emit_list(emit)?;
    let obj = js_sys::Object::new();

    for e in emit_list {
        match e.as_str() {
            "wasm" => {
                let bytes = Uint8Array::from(compiled.wasm.as_slice());
                Reflect::set(&obj, &JsValue::from_str("wasm"), &bytes.into())?;
            }
            "wat" => {
                let wat = make_wat(
                    &compiled.wasm,
                    attach_source,
                    entry_path,
                    source,
                    &compiled.wat_comments,
                )?;
                Reflect::set(&obj, &JsValue::from_str("wat"), &JsValue::from_str(&wat))?;
            }
            "wat-min" => {
                let wat_min = make_wat_min(&compiled.wasm, attach_source, entry_path, source)?;
                Reflect::set(&obj, &JsValue::from_str("wat-min"), &JsValue::from_str(&wat_min))?;
            }
            other => {
                let msg = format!("unknown emit kind: {other} (expected wasm, wat, wat-min)");
                return Err(JsValue::from_str(&msg));
            }
        }
    }

    Ok(obj.into())
}

#[wasm_bindgen]
pub fn compile_source(source: &str) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_vfs(entry_path: &str, source: &str, vfs: JsValue) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry(entry_path, source, Some(vfs))
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_outputs(source: &str, emit: JsValue, attach_source: bool) -> Result<JsValue, JsValue> {
    // entry_path は CLI の -i 相当（lib 側では仮想パス）
    compile_outputs_impl("/virtual/entry.nepl", source, None, emit, attach_source)
}

#[wasm_bindgen]
pub fn compile_outputs_with_vfs(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    emit: JsValue,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
    compile_outputs_impl(entry_path, source, Some(vfs), emit, attach_source)
}

#[wasm_bindgen]
pub fn compile_to_wat_min(source: &str, attach_source: bool) -> Result<String, JsValue> {
    let compiled = compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map_err(|msg| JsValue::from_str(&msg))?;
    make_wat_min(&compiled.wasm, attach_source, "/virtual/entry.nepl", source)
}

#[wasm_bindgen]
pub fn compile_to_wat(source: &str) -> Result<String, JsValue> {
    let compiled = compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map_err(|msg| JsValue::from_str(&msg))?;
    make_wat(
        &compiled.wasm,
        false,
        "/virtual/entry.nepl",
        source,
        &compiled.wat_comments,
    )
}

#[wasm_bindgen]
pub fn list_tests() -> String {
    test_sources()
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join("\n")
}

#[wasm_bindgen]
pub fn get_stdlib_files() -> JsValue {
    let entries = stdlib_entries();
    let arr = js_sys::Array::new();
    for (path, content) in entries {
        let entry = js_sys::Array::new();
        entry.push(&JsValue::from_str(path));
        entry.push(&JsValue::from_str(content));
        arr.push(&entry);
    }
    arr.into()
}

#[wasm_bindgen]
pub fn get_bundled_stdlib_vfs() -> JsValue {
    let obj = js_sys::Object::new();
    for (path, content) in stdlib_entries() {
        let key = format!("/stdlib/{path}");
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str(&key),
            &JsValue::from_str(content),
        );
    }
    obj.into()
}

#[wasm_bindgen]
pub fn get_example_files() -> JsValue {
    let entries = example_entries();
    let arr = js_sys::Array::new();
    for (path, content) in entries {
        let entry = js_sys::Array::new();
        entry.push(&JsValue::from_str(path));
        entry.push(&JsValue::from_str(content));
        arr.push(&entry);
    }
    arr.into()
}

#[wasm_bindgen]
pub fn get_readme() -> String {
    readme_content().to_string()
}

#[wasm_bindgen]
pub fn compile_test(name: &str) -> Result<Vec<u8>, JsValue> {
    let src = test_sources()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, src)| *src)
        .ok_or_else(|| JsValue::from_str("unknown test"))?;
    compile_wasm_with_entry(&format!("/virtual/tests/{name}.nepl"), src, None)
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

struct CompiledWasm {
    wasm: Vec<u8>,
    wat_comments: String,
}

fn compile_wasm_with_entry(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
) -> Result<CompiledWasm, String> {
    compile_wasm_with_entry_and_profile_and_stdlib(entry_path, source, vfs, None, None)
}

fn parse_profile(profile: &str) -> Option<BuildProfile> {
    match profile {
        "debug" => Some(BuildProfile::Debug),
        "release" => Some(BuildProfile::Release),
        _ => None,
    }
}

fn compile_wasm_with_entry_and_profile(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    profile: Option<BuildProfile>,
) -> Result<CompiledWasm, String> {
    compile_wasm_with_entry_and_profile_and_stdlib(entry_path, source, vfs, None, profile)
}

fn merge_vfs_sources(
    sources: &mut BTreeMap<PathBuf, String>,
    vfs: Option<JsValue>,
) {
    if let Some(vfs_val) = vfs {
        if vfs_val.is_object() {
            let entries = js_sys::Object::entries(&vfs_val.into());
            for entry in entries.iter() {
                let pair = js_sys::Array::from(&entry);
                let path_str = pair.get(0).as_string().unwrap_or_default();
                let content = pair.get(1).as_string().unwrap_or_default();
                if !path_str.is_empty() {
                    sources.insert(PathBuf::from(path_str), content);
                }
            }
        }
    }
}

fn compile_wasm_with_entry_and_profile_and_stdlib(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    stdlib_vfs: Option<JsValue>,
    profile: Option<BuildProfile>,
) -> Result<CompiledWasm, String> {
    let stdlib_root = PathBuf::from("/stdlib");
    let mut sources = stdlib_sources(&stdlib_root);
    // stdlib 差し替えが指定された場合は、先に上書きで適用する
    merge_vfs_sources(&mut sources, stdlib_vfs);
    // 呼び出し元 VFS は最後に適用する
    merge_vfs_sources(&mut sources, vfs);

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Loader context contains {} files", sources.len()).into());
    
    let mut loader = Loader::new(stdlib_root);
    let mut provider = |path: &PathBuf| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| {
                let msg = format!(
                    "missing source: {}. Available sources: {:?}",
                    path.display(),
                    sources.keys().collect::<Vec<_>>()
                );
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&msg.clone().into());
                nepl_core::loader::LoaderError::Io(msg)
            })
    };
    let loaded = loader
        .load_inline_with_provider(PathBuf::from(entry_path), source.to_string(), &mut provider)
        .map_err(|e| e.to_string())?;
    let artifact = compile_module(
        loaded.module,
        CompileOptions {
            target: None,
            verbose: false,
            profile,
        },
    )
    .map_err(|e| render_core_error(e, &loaded.source_map))?;
    Ok(CompiledWasm {
        wasm: artifact.wasm,
        wat_comments: artifact.wat_comments,
    })
}

#[wasm_bindgen]
pub fn compile_source_with_vfs_and_stdlib(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    stdlib_vfs: JsValue,
) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry_and_profile_and_stdlib(
        entry_path,
        source,
        Some(vfs),
        Some(stdlib_vfs),
        None,
    )
    .map(|a| a.wasm)
    .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_profile(source: &str, profile: &str) -> Result<Vec<u8>, JsValue> {
    let parsed = parse_profile(profile)
        .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
    compile_wasm_with_entry_and_profile("/virtual/entry.nepl", source, None, Some(parsed))
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_vfs_and_profile(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    profile: &str,
) -> Result<Vec<u8>, JsValue> {
    let parsed = parse_profile(profile)
        .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
    compile_wasm_with_entry_and_profile(entry_path, source, Some(vfs), Some(parsed))
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_vfs_stdlib_and_profile(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    stdlib_vfs: JsValue,
    profile: &str,
) -> Result<Vec<u8>, JsValue> {
    let parsed = parse_profile(profile)
        .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
    compile_wasm_with_entry_and_profile_and_stdlib(
        entry_path,
        source,
        Some(vfs),
        Some(stdlib_vfs),
        Some(parsed),
    )
    .map(|a| a.wasm)
    .map_err(|msg| JsValue::from_str(&msg))
}

fn render_core_error(err: CoreError, sm: &SourceMap) -> String {
    match err {
        CoreError::Diagnostics(diags) => render_diagnostics(&diags, sm),
        other => other.to_string(),
    }
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) -> String {
    let mut out = String::new();
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const CYAN: &str = "\x1b[36m";
    const BLUE: &str = "\x1b[34m";

    for d in diags {
        let (severity_str, severity_color) = match d.severity {
            Severity::Error => ("error", RED),
            Severity::Warning => ("warning", YELLOW),
        };
        let code = d.code.unwrap_or("");
        let primary = &d.primary;
        let (line, col) = sm
            .line_col(primary.span.file_id, primary.span.start)
            .unwrap_or((0, 0));
        let path = sm
            .path(primary.span.file_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());
        let code_display = if code.is_empty() {
            String::new()
        } else {
            format!("[{code}]")
        };
        
        // エラーヘッダ
        out.push_str(&format!(
            "{color}{bold}{sev}{code_disp}{reset}: {bold}{message}{reset}\n",
            color = severity_color,
            bold = BOLD,
            sev = severity_str,
            code_disp = code_display,
            reset = RESET,
            message = d.message
        ));
        
        // 位置ポインタ
        out.push_str(&format!(
            " {blue}-->{reset} {path}:{line}:{col}\n",
            blue = BLUE,
            reset = RESET,
            path = path,
            line = line + 1,
            col = col + 1
        ));

        if let Some(line_str) = sm.line_str(primary.span.file_id, line) {
            out.push_str(&format!(
                "  {blue}{line_num:>4} |{reset} {text}\n",
                blue = BLUE,
                reset = RESET,
                line_num = line + 1,
                text = line_str
            ));
            let caret_pos = col;
            out.push_str(&format!(
                "       {blue}|{reset} {spaces}{color}{bold}{carets}{reset}\n",
                blue = BLUE,
                reset = RESET,
                spaces = " ".repeat(caret_pos),
                color = severity_color,
                bold = BOLD,
                carets = "^".repeat(primary.span.len().max(1) as usize)
            ));
        }
        for label in &d.secondary {
            let (l, c) = sm
                .line_col(label.span.file_id, label.span.start)
                .unwrap_or((0, 0));
            let p = sm
                .path(label.span.file_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let msg = label.message.as_ref().map(|m| m.as_str()).unwrap_or("");
            out.push_str(&format!(
                " {blue}note:{reset} {p}:{line}:{col}: {msg}\n",
                blue = BLUE,
                reset = RESET,
                line = l + 1,
                col = c + 1
            ));
        }
        out.push('\n');
    }
    out
}

fn stdlib_sources(root: &PathBuf) -> BTreeMap<PathBuf, String> {
    let mut map = BTreeMap::new();
    for (path, src) in stdlib_entries() {
        map.insert(root.join(path), src.to_string());
    }
    map
}

include!(concat!(env!("OUT_DIR"), "/stdlib_entries.rs"));

fn stdlib_entries() -> &'static [(&'static str, &'static str)] {
    STD_LIB_ENTRIES
}

fn example_entries() -> &'static [(&'static str, &'static str)] {
    EXAMPLE_ENTRIES
}

fn readme_content() -> &'static str {
    README_CONTENT
}

fn test_sources() -> &'static [(&'static str, &'static str)] {
    TEST_ENTRIES
}

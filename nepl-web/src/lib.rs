use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use js_sys::{Function, Reflect, Uint8Array};
use nepl_core::ast::{
    Block, Directive, FnBody, MatchArm, MatchPattern, PrefixExpr, PrefixItem, Stmt, Symbol,
};
use nepl_core::compiler::{
    compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_and_neplproof,
    compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_neplproof_and_stage_timings,
    CompileStageTimings,
};
use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::diagnostic_codes::{DiagnosticCode, LoaderDiagnosticCode};
use nepl_core::error::CoreError;
use nepl_core::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirLine};
use nepl_core::artifact::{
    nepl_meta_artifact_pre_typecheck_envelope_for_module_surface,
    nepl_meta_artifact_pre_typecheck_envelope_for_module_surface_with_source_identity,
    NeplMetaArtifact, NeplMetaArtifactStore,
};
use nepl_core::lexer::{lex, Token, TokenKind};
use nepl_core::loader::{
    Loader, LoaderError, LoaderSessionCache, NeplMetaDependencyEdgePreTypecheckProbe, SourceMap,
};
use nepl_core::parser::parse_tokens;
use nepl_core::resource::{ResourceSummaryProofArtifact, ResourceSummaryValueCache};
use nepl_core::resolve::DefId;
use nepl_core::source_cache_key::compiled_source_cache_key_part;
use nepl_core::span::{FileId, Span};
use nepl_core::typecheck::typecheck;
use nepl_core::{
    BuildProfile, CompilationArtifactOptions, CompileOptions, CompileTarget,
    ResourceSummaryProofArtifactCacheOptions,
};
use wasmprinter::print_bytes;
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

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

fn span_to_js_with_map(source: &str, span: Span, source_map: Option<&SourceMap>) -> JsValue {
    let span_source = source_map
        .and_then(|sm| sm.get(span.file_id))
        .unwrap_or(source);
    let obj = js_sys::Object::new();
    let (start_line, start_col) = line_col_of(span_source, span.start);
    let (end_line, end_col) = line_col_of(span_source, span.end);
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
    if let Some(path) = source_map.and_then(|sm| sm.path(span.file_id)) {
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("file_path"),
            &JsValue::from_str(&path.to_string_lossy()),
        );
    }
    obj.into()
}

fn span_to_js(source: &str, span: Span) -> JsValue {
    span_to_js_with_map(source, span, None)
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
        TokenKind::Percent => "Percent",
        TokenKind::Backslash => "Backslash",
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
        TokenKind::CharLiteral(_) => "CharLiteral",
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
        TokenKind::DirCapability(_) => "DirCapability",
        TokenKind::DirWasm => "DirWasm",
        TokenKind::DirLlvmIr => "DirLlvmIr",
        TokenKind::DirIndentWidth(_) => "DirIndentWidth",
        TokenKind::DirInclude(_) => "DirInclude",
        TokenKind::DirExtern { .. } => "DirExtern",
        TokenKind::DirIntrinsic => "DirIntrinsic",
        TokenKind::DirPrelude(_) => "DirPrelude",
        TokenKind::DirNoPrelude => "DirNoPrelude",
        TokenKind::WasmText(_) => "WasmText",
        TokenKind::LlvmIrText(_) => "LlvmIrText",
        TokenKind::MlstrLine(_) => "MlstrLine",
        TokenKind::DocComment(_) => "DocComment",
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
        | TokenKind::DirCapability(v)
        | TokenKind::DirInclude(v)
        | TokenKind::DirPrelude(v)
        | TokenKind::WasmText(v)
        | TokenKind::LlvmIrText(v)
        | TokenKind::MlstrLine(v)
        | TokenKind::DocComment(v) => Some(v.clone()),
        TokenKind::BoolLiteral(v) => Some(v.to_string()),
        TokenKind::CharLiteral(v) => Some(v.to_string()),
        TokenKind::DirIndentWidth(v) => Some(v.to_string()),
        TokenKind::DirExtern {
            vis,
            module,
            name,
            func,
            signature,
        } => Some(format!(
            "vis={vis:?}, module={module}, name={name}, func={func}, signature={signature}"
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
            &JsValue::from_str(d.code.as_str()),
        );
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("code_message"),
            &JsValue::from_str(d.code.message()),
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

        let notes = js_sys::Array::new();
        for note in &d.notes {
            notes.push(&JsValue::from_str(note));
        }
        let _ = Reflect::set(&obj, &JsValue::from_str("notes"), &notes);

        let helps = js_sys::Array::new();
        for help in &d.helps {
            helps.push(&JsValue::from_str(help));
        }
        let _ = Reflect::set(&obj, &JsValue::from_str("helps"), &helps);
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
                FnBody::LlvmIr(block) => {
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
        Stmt::LlvmIr(block) => {
            let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("LlvmIr"));
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
    def_id: Option<DefId>,
    scope_depth: usize,
    doc: Option<String>,
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
    callee_def_id: Option<DefId>,
}

#[derive(Clone)]
struct SemanticTokenTrace {
    token_index: usize,
    inferred_expr_id: Option<usize>,
    inferred_type: Option<String>,
    expr_span: Option<Span>,
    arg_index: Option<usize>,
    arg_span: Option<Span>,
    selected_resolved_def_id: Option<usize>,
}

#[derive(Default)]
struct NameResolutionTrace {
    defs: Vec<NameDefTrace>,
    refs: Vec<NameRefTrace>,
    shadows: Vec<NameShadowTrace>,
    scopes: Vec<BTreeMap<String, Vec<usize>>>,
    warn_shadow: bool,
}

impl NameResolutionTrace {
    fn new() -> Self {
        Self::new_with_options(true)
    }

    fn new_with_options(warn_shadow: bool) -> Self {
        Self {
            defs: Vec::new(),
            refs: Vec::new(),
            shadows: Vec::new(),
            scopes: vec![BTreeMap::new()],
            warn_shadow,
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

    fn define(&mut self, name: String, kind: &'static str, span: Span, doc: Option<String>) -> usize {
        let existing_candidates = self.lookup_candidates(&name);
        let id = self.defs.len();
        let depth = self.current_depth();
        self.defs.push(NameDefTrace {
            id,
            name: name.clone(),
            kind,
            span,
            def_id: DefId::from_span(span),
            scope_depth: depth,
            doc,
        });

        if !existing_candidates.is_empty() {
            let has_outer_definition = existing_candidates.iter().copied().any(|candidate_id| {
                self.defs
                    .get(candidate_id)
                    .map_or(false, |definition| definition.scope_depth < depth)
            });
            let severity = if self.warn_shadow && has_outer_definition && is_variable_def_kind(kind)
            {
                "warning"
            } else {
                "info"
            };
            let message = if severity == "warning" {
                format!("symbol '{}' shadows an outer {} definition", name, kind)
            } else if has_outer_definition {
                format!("'{}' shadows an outer definition", name)
            } else {
                format!("'{}' redefines an existing definition in the same scope", name)
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

fn is_variable_def_kind(kind: &str) -> bool {
    matches!(kind, "let_hoisted" | "let_mut" | "param" | "match_bind")
}

fn hoist_block_defs(trace: &mut NameResolutionTrace, block: &Block) {
    for stmt in &block.items {
        match stmt {
            Stmt::FnDef(def) => {
                trace.define(def.name.name.clone(), "fn", def.name.span, def.doc.clone());
            }
            Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                if let Some(PrefixItem::Symbol(Symbol::Let { name, mutable, .. })) = expr.items.first() {
                    if !*mutable {
                        trace.define(name.name.clone(), "let_hoisted", name.span, None);
                    }
                }
            }
            Stmt::StructDef(def) => {
                trace.define(def.name.name.clone(), "struct", def.name.span, def.doc.clone());
            }
            Stmt::EnumDef(def) => {
                trace.define(def.name.name.clone(), "enum", def.name.span, def.doc.clone());
            }
            Stmt::Trait(def) => {
                trace.define(def.name.name.clone(), "trait", def.name.span, def.doc.clone());
            }
            _ => {}
        }
    }
}

fn trace_match_arm(trace: &mut NameResolutionTrace, arm: &MatchArm) {
    trace.push_scope();
    if let MatchPattern::Variant { bind: Some(bind), .. } = &arm.pattern {
        trace.define(bind.name.clone(), "match_bind", bind.span, None);
    }
    trace_block(trace, &arm.body);
    trace.pop_scope();
}

fn trace_prefix_expr(trace: &mut NameResolutionTrace, expr: &PrefixExpr) {
    for (idx, item) in expr.items.iter().enumerate() {
        match item {
            PrefixItem::Symbol(Symbol::Let { name, mutable, .. }) => {
                if *mutable {
                    trace.define(name.name.clone(), "let_mut", name.span, None);
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
            | PrefixItem::Symbol(Symbol::AddrOf { .. })
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
                    trace.define(param.name.clone(), "param", param.span, None);
                }
                trace_block(trace, body);
                trace.pop_scope();
            }
            FnBody::Wasm(_) => {}
            FnBody::LlvmIr(_) => {}
        },
        Stmt::FnAlias(alias) => {
            trace.reference(alias.target.name.clone(), alias.target.span);
            trace.define(alias.name.name.clone(), "fn_alias", alias.name.span, alias.doc.clone());
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

fn def_trace_to_js(source: &str, source_map: Option<&SourceMap>, def: &NameDefTrace) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(def.id as f64));
    let _ = Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&def.name));
    let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str(def.kind));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("scope_depth"),
        &JsValue::from_f64(def.scope_depth as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("span"),
        &span_to_js_with_map(source, def.span, source_map),
    );
    if let Some(doc) = &def.doc {
        let _ = Reflect::set(&obj, &JsValue::from_str("doc"), &JsValue::from_str(doc));
    }
    obj.into()
}

fn ref_trace_to_js(
    source: &str,
    source_map: Option<&SourceMap>,
    rf: &NameRefTrace,
    defs: &[NameDefTrace],
) -> JsValue {
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("scope_depth"),
        &JsValue::from_f64(rf.scope_depth as f64),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("span"),
        &span_to_js_with_map(source, rf.span, source_map),
    );
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

    if let Some(id) = rf.resolved_def_id {
        if let Some(def) = defs.get(id) {
            let resolved = js_sys::Object::new();
            let _ = Reflect::set(
                &resolved,
                &JsValue::from_str("id"),
                &JsValue::from_f64(def.id as f64),
            );
            let _ = Reflect::set(
                &resolved,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name),
            );
            let _ = Reflect::set(
                &resolved,
                &JsValue::from_str("kind"),
                &JsValue::from_str(def.kind),
            );
            let _ = Reflect::set(
                &resolved,
                &JsValue::from_str("scope_depth"),
                &JsValue::from_f64(def.scope_depth as f64),
            );
            let _ = Reflect::set(
                &resolved,
                &JsValue::from_str("span"),
                &span_to_js_with_map(source, def.span, source_map),
            );
            let _ = Reflect::set(&obj, &JsValue::from_str("resolved_def"), &resolved);
        } else {
            let _ = Reflect::set(&obj, &JsValue::from_str("resolved_def"), &JsValue::NULL);
        }
    } else {
        let _ = Reflect::set(&obj, &JsValue::from_str("resolved_def"), &JsValue::NULL);
    }

    let cand_defs = js_sys::Array::new();
    for id in &rf.candidate_def_ids {
        if let Some(def) = defs.get(*id) {
            let item = js_sys::Object::new();
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("id"),
                &JsValue::from_f64(def.id as f64),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("name"),
                &JsValue::from_str(&def.name),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("kind"),
                &JsValue::from_str(def.kind),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("scope_depth"),
                &JsValue::from_f64(def.scope_depth as f64),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("span"),
                &span_to_js_with_map(source, def.span, source_map),
            );
            cand_defs.push(&item);
        }
    }
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("candidate_definitions"),
        &cand_defs,
    );
    obj.into()
}

fn shadow_trace_to_js(source: &str, source_map: Option<&SourceMap>, sh: &NameShadowTrace) -> JsValue {
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
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("span"),
        &span_to_js_with_map(source, sh.span, source_map),
    );
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

fn name_resolution_payload_to_js(
    source: &str,
    source_map: Option<&SourceMap>,
    trace: &NameResolutionTrace,
) -> JsValue {
    let payload = js_sys::Object::new();

    let defs = js_sys::Array::new();
    for def in &trace.defs {
        defs.push(&def_trace_to_js(source, source_map, def));
    }
    let refs = js_sys::Array::new();
    for rf in &trace.refs {
        refs.push(&ref_trace_to_js(source, source_map, rf, &trace.defs));
    }
    let shadows = js_sys::Array::new();
    for sh in &trace.shadows {
        shadows.push(&shadow_trace_to_js(source, source_map, sh));
    }
    let shadow_diagnostics = js_sys::Array::new();
    for sh in &trace.shadows {
        if matches!(sh.severity, "warning" | "info") {
            shadow_diagnostics.push(&shadow_trace_to_js(source, source_map, sh));
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
        &JsValue::from_str("warn_shadow"),
        &JsValue::from_bool(trace.warn_shadow),
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
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("selected_resolved_def_id"),
        &st.selected_resolved_def_id
            .map(|v| JsValue::from_f64(v as f64))
            .unwrap_or(JsValue::NULL),
    );
    obj.into()
}

fn build_token_hints_to_js(
    source: &str,
    source_map: Option<&SourceMap>,
    tokens: &[Token],
    token_semantics: &[SemanticTokenTrace],
    resolve_trace: &NameResolutionTrace,
) -> js_sys::Array {
    let hints = js_sys::Array::new();
    for (tok_idx, token) in tokens.iter().enumerate() {
        let item = js_sys::Object::new();
        let _ = Reflect::set(
            &item,
            &JsValue::from_str("token_index"),
            &JsValue::from_f64(tok_idx as f64),
        );

        if let Some(ts) = token_semantics.get(tok_idx) {
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("inferred_expr_id"),
                &ts.inferred_expr_id
                    .map(|v| JsValue::from_f64(v as f64))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("inferred_type"),
                &ts.inferred_type
                    .as_ref()
                    .map(|s| JsValue::from_str(s))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("expression_range"),
                &ts.expr_span
                    .map(|s| span_to_js_with_map(source, s, source_map))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("arg_index"),
                &ts.arg_index
                    .map(|v| JsValue::from_f64(v as f64))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("arg_range"),
                &ts.arg_span
                    .map(|s| span_to_js_with_map(source, s, source_map))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("selected_resolved_def_id"),
                &ts.selected_resolved_def_id
                    .map(|v| JsValue::from_f64(v as f64))
                    .unwrap_or(JsValue::NULL),
            );
        } else {
            let _ = Reflect::set(&item, &JsValue::from_str("inferred_expr_id"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("inferred_type"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("expression_range"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("arg_index"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("arg_range"), &JsValue::NULL);
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("selected_resolved_def_id"),
                &JsValue::NULL,
            );
        }

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

        if let Some(rf) = best_ref {
            let resolved_def_id = token_semantics
                .get(tok_idx)
                .and_then(|ts| ts.selected_resolved_def_id)
                .or(rf.resolved_def_id);
            let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("ref_span"),
                &span_to_js_with_map(source, rf.span, source_map),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("resolved_def_id"),
                &resolved_def_id
                    .map(|v| JsValue::from_f64(v as f64))
                    .unwrap_or(JsValue::NULL),
            );
            let cand = js_sys::Array::new();
            for id in &rf.candidate_def_ids {
                cand.push(&JsValue::from_f64(*id as f64));
            }
            let _ = Reflect::set(&item, &JsValue::from_str("candidate_def_ids"), &cand);
            if let Some(id) = resolved_def_id {
                if let Some(def) = resolve_trace.defs.get(id) {
                    let resolved = js_sys::Object::new();
                    let _ = Reflect::set(
                        &resolved,
                        &JsValue::from_str("id"),
                        &JsValue::from_f64(def.id as f64),
                    );
                    let _ = Reflect::set(
                        &resolved,
                        &JsValue::from_str("name"),
                        &JsValue::from_str(&def.name),
                    );
                    let _ = Reflect::set(
                        &resolved,
                        &JsValue::from_str("kind"),
                        &JsValue::from_str(def.kind),
                    );
                    let _ = Reflect::set(
                        &resolved,
                        &JsValue::from_str("scope_depth"),
                        &JsValue::from_f64(def.scope_depth as f64),
                    );
                    let _ = Reflect::set(
                        &resolved,
                        &JsValue::from_str("span"),
                        &span_to_js_with_map(source, def.span, source_map),
                    );
                    if let Some(doc) = &def.doc {
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("doc"),
                            &JsValue::from_str(doc),
                        );
                    }
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_definition"),
                        &resolved,
                    );
                } else {
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_definition"),
                        &JsValue::NULL,
                    );
                }
            } else {
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("resolved_definition"),
                    &JsValue::NULL,
                );
            }

            let cand_defs = js_sys::Array::new();
            for id in &rf.candidate_def_ids {
                if let Some(def) = resolve_trace.defs.get(*id) {
                    let cand_def = js_sys::Object::new();
                    let _ = Reflect::set(
                        &cand_def,
                        &JsValue::from_str("id"),
                        &JsValue::from_f64(def.id as f64),
                    );
                    let _ = Reflect::set(
                        &cand_def,
                        &JsValue::from_str("name"),
                        &JsValue::from_str(&def.name),
                    );
                    let _ = Reflect::set(
                        &cand_def,
                        &JsValue::from_str("kind"),
                        &JsValue::from_str(def.kind),
                    );
                    let _ = Reflect::set(
                        &cand_def,
                        &JsValue::from_str("scope_depth"),
                        &JsValue::from_f64(def.scope_depth as f64),
                    );
                    let _ = Reflect::set(
                        &cand_def,
                        &JsValue::from_str("span"),
                        &span_to_js_with_map(source, def.span, source_map),
                    );
                    if let Some(doc) = &def.doc {
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("doc"),
                            &JsValue::from_str(doc),
                        );
                    }
                    cand_defs.push(&cand_def);
                }
            }
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("candidate_definitions"),
                &cand_defs,
            );
        } else {
            let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &JsValue::NULL);
            let _ = Reflect::set(&item, &JsValue::from_str("resolved_def_id"), &JsValue::NULL);
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("candidate_def_ids"),
                &js_sys::Array::new(),
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("resolved_definition"),
                &JsValue::NULL,
            );
            let _ = Reflect::set(
                &item,
                &JsValue::from_str("candidate_definitions"),
                &js_sys::Array::new(),
            );
        }

        hints.push(&item);
    }
    hints
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.file_id == inner.file_id && outer.start <= inner.start && inner.end <= outer.end
}

fn span_width(span: Span) -> usize {
    span.end.saturating_sub(span.start) as usize
}

fn callee_def_id(callee: &FuncRef) -> Option<DefId> {
    match callee {
        FuncRef::User(_, _, def_id) => *def_id,
        FuncRef::Builtin(_) | FuncRef::Trait { .. } => None,
    }
}

fn trace_def_id(resolve_trace: &NameResolutionTrace, def_id: DefId) -> Option<usize> {
    resolve_trace
        .defs
        .iter()
        .find(|def| def.def_id == Some(def_id))
        .map(|def| def.id)
}

fn best_semantic_expr_for_token<'a>(
    exprs: &'a [SemanticExprTrace],
    token: &Token,
) -> Option<&'a SemanticExprTrace> {
    let mut best_expr: Option<&SemanticExprTrace> = None;
    for ex in exprs {
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
    best_expr
}

fn selected_resolved_def_id_for_token(
    token: &Token,
    exprs: &[SemanticExprTrace],
    resolve_trace: &NameResolutionTrace,
) -> Option<usize> {
    best_semantic_expr_for_token(exprs, token)
        .and_then(|expr| expr.callee_def_id)
        .and_then(|def_id| trace_def_id(resolve_trace, def_id))
}

fn hir_kind_name(kind: &HirExprKind) -> &'static str {
    match kind {
        HirExprKind::LiteralI32(_) => "LiteralI32",
        HirExprKind::LiteralF32(_) => "LiteralF32",
        HirExprKind::LiteralBool(_) => "LiteralBool",
        HirExprKind::LiteralStr(_) => "LiteralStr",
        HirExprKind::Unit => "Unit",
        HirExprKind::Var(_) => "Var",
        HirExprKind::FnValue(_) => "FnValue",
        HirExprKind::MemoizedFunctionValue(_) => "MemoizedFunctionValue",
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
    let expr_callee_def_id = match &expr.kind {
        HirExprKind::Call { callee, .. } => callee_def_id(callee),
        _ => None,
    };
    out.push(SemanticExprTrace {
        id,
        function_name: function_name.to_string(),
        kind: hir_kind_name(&expr.kind),
        span: expr.span,
        ty: types.type_to_string(expr.ty),
        parent_id,
        arg_spans: Vec::new(),
        callee_def_id: expr_callee_def_id,
    });

    let mut arg_spans = Vec::new();
    match &expr.kind {
        HirExprKind::Call { args, .. } => {
            for a in args {
                arg_spans.push(a.span);
                collect_semantic_expr(a, function_name, types, Some(id), out);
            }
        }
        HirExprKind::FnValue(_) | HirExprKind::MemoizedFunctionValue(_) => {}
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
    let mut saw_target_directive = false;
    let mut diags = Vec::new();

    for d in &module.directives {
        if let Directive::Target { target, span } = d {
            saw_target_directive = true;
            let parsed = parse_target_name(target);
            if let Some(t) = parsed {
                if let Some((_, prev_span)) = found {
                    diags.push(
                        loader_error(
                            LoaderDiagnosticCode::TargetMultipleDirective,
                            "multiple #target directives are not allowed",
                            *span,
                        )
                        .with_secondary_label(prev_span, Some("previous #target here".into())),
                    );
                } else {
                    found = Some((t, *span));
                }
            } else {
                diags.push(loader_error(
                    LoaderDiagnosticCode::TargetUnknown,
                    "unknown target in #target",
                    *span,
                ));
            }
        }
    }

    if !saw_target_directive {
        for it in &module.root.items {
            if let Stmt::Directive(Directive::Target { target, span }) = it {
                let parsed = parse_target_name(target);
                if let Some(t) = parsed {
                    if let Some((_, prev_span)) = found {
                        diags.push(
                            loader_error(
                                LoaderDiagnosticCode::TargetMultipleDirective,
                                "multiple #target directives are not allowed",
                                *span,
                            )
                            .with_secondary_label(prev_span, Some("previous #target here".into())),
                        );
                    } else {
                        found = Some((t, *span));
                    }
                } else {
                    diags.push(loader_error(
                        LoaderDiagnosticCode::TargetUnknown,
                        "unknown target in #target",
                        *span,
                    ));
                }
            }
        }
    }

    (found.map(|(t, _)| t).unwrap_or(CompileTarget::Wasm), diags)
}

fn parse_target_name(target: &str) -> Option<CompileTarget> {
    match target {
        "wasm" | "core" => Some(CompileTarget::Wasm),
        "wasi" | "std" => Some(CompileTarget::Wasi),
        "llvm" => Some(CompileTarget::Llvm),
        _ => None,
    }
}

fn loader_error(code: LoaderDiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Loader(code), message, span)
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
    let warn_shadow = Reflect::get(&options, &JsValue::from_str("warn_shadow"))
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
        let mut trace = NameResolutionTrace::new_with_options(warn_shadow);
        trace_block(&mut trace, &module.root);
        let payload = name_resolution_payload_to_js(source, None, &trace);
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
            &JsValue::from_str("warn_shadow"),
            &JsValue::from_bool(warn_shadow),
        );
        let _ = Reflect::set(&out, &JsValue::from_str("policy"), &policy);
    }

    out.into()
}

/// VFS を使って import/alias/use を含む複数ファイルの名前解決情報を返します。
#[wasm_bindgen]
pub fn analyze_name_resolution_with_vfs(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    options: JsValue,
) -> JsValue {
    let warn_shadow = Reflect::get(&options, &JsValue::from_str("warn_shadow"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let stdlib_root = PathBuf::from("/stdlib");
    let bundled_sources = stdlib_sources(&stdlib_root);
    let mut overlay_sources = BTreeMap::new();
    merge_vfs_sources(&mut overlay_sources, Some(vfs));

    let mut loader = Loader::new(stdlib_root);
    let mut provider = |path: &PathBuf| {
        lookup_web_source(&bundled_sources, &overlay_sources, path).ok_or_else(|| {
            nepl_core::loader::LoaderError::Io(format!("missing source: {}", path.display()))
        })
    };
    let loaded = loader.load_inline_with_provider(
        PathBuf::from(entry_path),
        source.to_string(),
        &mut provider,
    );

    let out = js_sys::Object::new();
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("stage"),
        &JsValue::from_str("name_resolution"),
    );

    match loaded {
        Ok(loaded) => {
            let mut trace = NameResolutionTrace::new_with_options(warn_shadow);
            trace_block(&mut trace, &loaded.module.root);
            let payload = name_resolution_payload_to_js(source, Some(&loaded.source_map), &trace);
            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(true));
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("diagnostics"),
                &js_sys::Array::new(),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("definitions"),
                &Reflect::get(&payload, &JsValue::from_str("definitions")).unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("references"),
                &Reflect::get(&payload, &JsValue::from_str("references")).unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("shadows"),
                &Reflect::get(&payload, &JsValue::from_str("shadows")).unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("shadow_diagnostics"),
                &Reflect::get(&payload, &JsValue::from_str("shadow_diagnostics"))
                    .unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("by_name"),
                &Reflect::get(&payload, &JsValue::from_str("by_name")).unwrap_or(JsValue::NULL),
            );
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("policy"),
                &Reflect::get(&payload, &JsValue::from_str("policy")).unwrap_or(JsValue::NULL),
            );
        }
        Err(e) => {
            let mut ds = Vec::new();
            ds.push(loader_error(
                LoaderDiagnosticCode::SourceFailure,
                format!("loader error: {}", e),
                Span::dummy(),
            ));
            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("diagnostics"),
                &diagnostics_to_js(source, &ds),
            );
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
                &JsValue::from_str("warn_shadow"),
                &JsValue::from_bool(warn_shadow),
            );
            let _ = Reflect::set(&out, &JsValue::from_str("policy"), &policy);
        }
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
        let resolve_payload = name_resolution_payload_to_js(source, None, &resolve_trace);

        let (target, mut target_diags) = resolve_target_for_analysis(module);
        has_error |= target_diags
            .iter()
            .any(|d| matches!(d.severity, Severity::Error));
        all_diags.append(&mut target_diags);

        let tc = typecheck(module, target, BuildProfile::Debug, None);
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
                    let resolved_def_id =
                        selected_resolved_def_id_for_token(token, &exprs, &resolve_trace)
                            .or(rf.resolved_def_id);
                    let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
                    let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &span_to_js(source, rf.span));
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_def_id"),
                        &resolved_def_id
                            .map(|v| JsValue::from_f64(v as f64))
                            .unwrap_or(JsValue::NULL),
                    );
                    let cand = js_sys::Array::new();
                    for id in &rf.candidate_def_ids {
                        cand.push(&JsValue::from_f64(*id as f64));
                    }
                    let _ = Reflect::set(&item, &JsValue::from_str("candidate_def_ids"), &cand);
                    if let Some(id) = resolved_def_id {
                        if let Some(def) = resolve_trace.defs.get(id) {
                            let resolved = js_sys::Object::new();
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("id"),
                                &JsValue::from_f64(def.id as f64),
                            );
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("name"),
                                &JsValue::from_str(&def.name),
                            );
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("kind"),
                                &JsValue::from_str(def.kind),
                            );
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("scope_depth"),
                                &JsValue::from_f64(def.scope_depth as f64),
                            );
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("span"),
                            &span_to_js(source, def.span),
                        );
                        if let Some(doc) = &def.doc {
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("doc"),
                                &JsValue::from_str(doc),
                            );
                        }
                        let _ = Reflect::set(
                            &item,
                            &JsValue::from_str("resolved_definition"),
                            &resolved,
                        );
                        } else {
                            let _ = Reflect::set(
                                &item,
                                &JsValue::from_str("resolved_definition"),
                                &JsValue::NULL,
                            );
                        }
                    } else {
                        let _ = Reflect::set(
                            &item,
                            &JsValue::from_str("resolved_definition"),
                            &JsValue::NULL,
                        );
                    }
                    let cand_defs = js_sys::Array::new();
                    for id in &rf.candidate_def_ids {
                        if let Some(def) = resolve_trace.defs.get(*id) {
                            let cand_def = js_sys::Object::new();
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("id"),
                                &JsValue::from_f64(def.id as f64),
                            );
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("name"),
                                &JsValue::from_str(&def.name),
                            );
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("kind"),
                                &JsValue::from_str(def.kind),
                            );
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("scope_depth"),
                                &JsValue::from_f64(def.scope_depth as f64),
                            );
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("span"),
                            &span_to_js(source, def.span),
                        );
                        if let Some(doc) = &def.doc {
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("doc"),
                                &JsValue::from_str(doc),
                            );
                        }
                        cand_defs.push(&cand_def);
                    }
                }
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("candidate_definitions"),
                        &cand_defs,
                    );
                } else {
                    let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::NULL);
                    let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &JsValue::NULL);
                    let _ = Reflect::set(&item, &JsValue::from_str("resolved_def_id"), &JsValue::NULL);
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("candidate_def_ids"),
                        &js_sys::Array::new(),
                    );
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_definition"),
                        &JsValue::NULL,
                    );
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("candidate_definitions"),
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
                    selected_resolved_def_id: best_expr
                        .and_then(|x| x.callee_def_id)
                        .and_then(|def_id| trace_def_id(&resolve_trace, def_id)),
                });
            }
            let token_sem_arr = js_sys::Array::new();
            for ts in &token_semantics {
                token_sem_arr.push(&semantic_token_to_js(source, ts));
            }
            let token_hint_arr =
                build_token_hints_to_js(source, None, &tokens, &token_semantics, &resolve_trace);

            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(!has_error));
            let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &expr_arr);
            let _ = Reflect::set(&out, &JsValue::from_str("token_semantics"), &token_sem_arr);
            let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &token_hint_arr);
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
            let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &js_sys::Array::new());
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
        let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("functions"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &JsValue::NULL);
        let _ = Reflect::set(&out, &JsValue::from_str("token_resolution"), &js_sys::Array::new());
    }

    out.into()
}

/// VFS を用いた複数ファイル解析版。
/// import/alias/use で参照された定義について、token_resolution に file_path 付きの
/// resolved_definition / candidate_definitions を返す。
#[wasm_bindgen]
pub fn analyze_semantics_with_vfs(entry_path: &str, source: &str, vfs: JsValue) -> JsValue {
    let stdlib_root = PathBuf::from("/stdlib");
    let bundled_sources = stdlib_sources(&stdlib_root);
    let mut overlay_sources = BTreeMap::new();
    merge_vfs_sources(&mut overlay_sources, Some(vfs));

    let mut loader = Loader::new(stdlib_root);
    let mut provider = |path: &PathBuf| {
        lookup_web_source(&bundled_sources, &overlay_sources, path)
            .ok_or_else(|| nepl_core::loader::LoaderError::Io(format!("missing source: {}", path.display())))
    };

    let loaded = loader.load_inline_with_provider(
        PathBuf::from(entry_path),
        source.to_string(),
        &mut provider,
    );

    let out = js_sys::Object::new();
    let _ = Reflect::set(
        &out,
        &JsValue::from_str("stage"),
        &JsValue::from_str("semantics"),
    );

    let file_id = FileId(0);
    let lex_result = lex(file_id, source);
    let tokens = lex_result.tokens.clone();
    let token_arr = js_sys::Array::new();
    for token in &tokens {
        token_arr.push(&token_to_js(source, token));
    }
    let _ = Reflect::set(&out, &JsValue::from_str("tokens"), &token_arr);

    let mut all_diags = lex_result.diagnostics.clone();
    let mut has_error = all_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));

    let loaded = match loaded {
        Ok(v) => v,
        Err(e) => {
            let mut ds = all_diags;
            ds.push(loader_error(
                LoaderDiagnosticCode::SourceFailure,
                format!("loader error: {}", e),
                Span::dummy(),
            ));
            let diagnostics = diagnostics_to_js(source, &ds);
            let _ = Reflect::set(&out, &JsValue::from_str("diagnostics"), &diagnostics);
            let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(false));
            let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &js_sys::Array::new());
            let _ = Reflect::set(
                &out,
                &JsValue::from_str("token_semantics"),
                &js_sys::Array::new(),
            );
            let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &js_sys::Array::new());
            let _ = Reflect::set(&out, &JsValue::from_str("functions"), &js_sys::Array::new());
            let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &JsValue::NULL);
            let _ = Reflect::set(&out, &JsValue::from_str("token_resolution"), &js_sys::Array::new());
            return out.into();
        }
    };

    let module = loaded.module;
    let source_map = loaded.source_map;

    let mut resolve_trace = NameResolutionTrace::new();
    trace_block(&mut resolve_trace, &module.root);
    let resolve_payload = name_resolution_payload_to_js(source, Some(&source_map), &resolve_trace);

    let (target, mut target_diags) = resolve_target_for_analysis(&module);
    has_error |= target_diags
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    all_diags.append(&mut target_diags);

    let tc = typecheck(&module, target, BuildProfile::Debug, Some(&source_map));
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
            let _ = Reflect::set(
                &f_obj,
                &JsValue::from_str("span"),
                &span_to_js_with_map(source, f.span, Some(&source_map)),
            );
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
                let resolved_def_id =
                    selected_resolved_def_id_for_token(token, &exprs, &resolve_trace)
                        .or(rf.resolved_def_id);
                let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(&rf.name));
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("ref_span"),
                    &span_to_js_with_map(source, rf.span, Some(&source_map)),
                );
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("resolved_def_id"),
                    &resolved_def_id
                        .map(|v| JsValue::from_f64(v as f64))
                        .unwrap_or(JsValue::NULL),
                );
                let cand = js_sys::Array::new();
                for id in &rf.candidate_def_ids {
                    cand.push(&JsValue::from_f64(*id as f64));
                }
                let _ = Reflect::set(&item, &JsValue::from_str("candidate_def_ids"), &cand);
                if let Some(id) = resolved_def_id {
                    if let Some(def) = resolve_trace.defs.get(id) {
                        let resolved = js_sys::Object::new();
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("id"),
                            &JsValue::from_f64(def.id as f64),
                        );
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("name"),
                            &JsValue::from_str(&def.name),
                        );
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("kind"),
                            &JsValue::from_str(def.kind),
                        );
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("scope_depth"),
                            &JsValue::from_f64(def.scope_depth as f64),
                        );
                        let _ = Reflect::set(
                            &resolved,
                            &JsValue::from_str("span"),
                            &span_to_js_with_map(source, def.span, Some(&source_map)),
                        );
                        if let Some(doc) = &def.doc {
                            let _ = Reflect::set(
                                &resolved,
                                &JsValue::from_str("doc"),
                                &JsValue::from_str(doc),
                            );
                        }
                        let _ = Reflect::set(
                            &item,
                            &JsValue::from_str("resolved_definition"),
                            &resolved,
                        );
                    } else {
                        let _ = Reflect::set(
                            &item,
                            &JsValue::from_str("resolved_definition"),
                            &JsValue::NULL,
                        );
                    }
                } else {
                    let _ = Reflect::set(
                        &item,
                        &JsValue::from_str("resolved_definition"),
                        &JsValue::NULL,
                    );
                }
                let cand_defs = js_sys::Array::new();
                for id in &rf.candidate_def_ids {
                    if let Some(def) = resolve_trace.defs.get(*id) {
                        let cand_def = js_sys::Object::new();
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("id"),
                            &JsValue::from_f64(def.id as f64),
                        );
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("name"),
                            &JsValue::from_str(&def.name),
                        );
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("kind"),
                            &JsValue::from_str(def.kind),
                        );
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("scope_depth"),
                            &JsValue::from_f64(def.scope_depth as f64),
                        );
                        let _ = Reflect::set(
                            &cand_def,
                            &JsValue::from_str("span"),
                            &span_to_js_with_map(source, def.span, Some(&source_map)),
                        );
                        if let Some(doc) = &def.doc {
                            let _ = Reflect::set(
                                &cand_def,
                                &JsValue::from_str("doc"),
                                &JsValue::from_str(doc),
                            );
                        }
                        cand_defs.push(&cand_def);
                    }
                }
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("candidate_definitions"),
                    &cand_defs,
                );
            } else {
                let _ = Reflect::set(&item, &JsValue::from_str("name"), &JsValue::NULL);
                let _ = Reflect::set(&item, &JsValue::from_str("ref_span"), &JsValue::NULL);
                let _ = Reflect::set(&item, &JsValue::from_str("resolved_def_id"), &JsValue::NULL);
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("candidate_def_ids"),
                    &js_sys::Array::new(),
                );
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("resolved_definition"),
                    &JsValue::NULL,
                );
                let _ = Reflect::set(
                    &item,
                    &JsValue::from_str("candidate_definitions"),
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
                selected_resolved_def_id: best_expr
                    .and_then(|x| x.callee_def_id)
                    .and_then(|def_id| trace_def_id(&resolve_trace, def_id)),
            });
        }
        let token_sem_arr = js_sys::Array::new();
        for ts in &token_semantics {
            token_sem_arr.push(&semantic_token_to_js(source, ts));
        }
        let token_hint_arr = build_token_hints_to_js(
            source,
            Some(&source_map),
            &tokens,
            &token_semantics,
            &resolve_trace,
        );

        let _ = Reflect::set(&out, &JsValue::from_str("ok"), &JsValue::from_bool(!has_error));
        let _ = Reflect::set(&out, &JsValue::from_str("expressions"), &expr_arr);
        let _ = Reflect::set(&out, &JsValue::from_str("token_semantics"), &token_sem_arr);
        let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &token_hint_arr);
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
        let _ = Reflect::set(&out, &JsValue::from_str("token_hints"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("functions"), &js_sys::Array::new());
        let _ = Reflect::set(&out, &JsValue::from_str("name_resolution"), &resolve_payload);
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
    let stdlib_root = PathBuf::from("/stdlib");
    let bundled_sources = stdlib_sources(&stdlib_root);
    compile_outputs_with_bundled_sources(
        entry_path,
        source,
        &stdlib_root,
        &bundled_sources,
        vfs,
        emit,
        attach_source,
    )
}

fn compile_outputs_with_bundled_sources(
    entry_path: &str,
    source: &str,
    stdlib_root: &PathBuf,
    bundled_sources: &BTreeMap<PathBuf, &'static str>,
    vfs: Option<JsValue>,
    emit: JsValue,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
    compile_outputs_with_bundled_sources_and_cache(
        entry_path,
        source,
        stdlib_root,
        bundled_sources,
        vfs,
        emit,
        attach_source,
        None,
    )
}

fn compile_outputs_with_bundled_sources_and_cache(
    entry_path: &str,
    source: &str,
    stdlib_root: &PathBuf,
    bundled_sources: &BTreeMap<PathBuf, &'static str>,
    vfs: Option<JsValue>,
    emit: JsValue,
    attach_source: bool,
    loader_cache: Option<&mut LoaderSessionCache>,
) -> Result<JsValue, JsValue> {
    let emit_list = parse_emit_list(emit)?;
    let include_wat_comments = emit_list.iter().any(|kind| kind == "wat");
    let compiled = compile_wasm_with_bundled_sources_and_cache(
        entry_path,
        source,
        stdlib_root,
        bundled_sources,
        vfs,
        None,
        None,
        include_wat_comments,
        loader_cache,
        None,
        None,
        None,
        None,
    )
    .map_err(|msg| JsValue::from_str(&msg))?;
    compile_outputs_from_compiled(&compiled, entry_path, source, emit_list, attach_source)
}

fn compile_outputs_from_compiled(
    compiled: &CompiledWasm,
    entry_path: &str,
    source: &str,
    emit_list: Vec<String>,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
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
    compile_wasm_with_entry_and_comments("/virtual/entry.nepl", source, None, false)
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_vfs(entry_path: &str, source: &str, vfs: JsValue) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry_and_comments(entry_path, source, Some(vfs), false)
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
    let compiled = compile_wasm_with_entry_and_comments("/virtual/entry.nepl", source, None, false)
        .map_err(|msg| JsValue::from_str(&msg))?;
    make_wat_min(&compiled.wasm, attach_source, "/virtual/entry.nepl", source)
}

#[wasm_bindgen]
pub fn compile_to_wat(source: &str) -> Result<String, JsValue> {
    let compiled = compile_wasm_with_entry_and_comments("/virtual/entry.nepl", source, None, true)
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
pub fn get_bundled_stdlib_hash() -> String {
    stdlib_hash().to_string()
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
    compile_wasm_with_entry_and_comments(&format!("/virtual/tests/{name}.nepl"), src, None, false)
        .map(|a| a.wasm)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[derive(Clone)]
struct CompiledWasm {
    wasm: Vec<u8>,
    wat_comments: String,
    nepl_meta_artifact: NeplMetaArtifact,
    resource_summary_proof_artifact: Option<ResourceSummaryProofArtifact>,
    stdlib_overlay_used: bool,
}

fn compile_wasm_with_entry_and_comments(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    include_wat_comments: bool,
) -> Result<CompiledWasm, String> {
    compile_wasm_with_entry_and_profile_and_stdlib(
        entry_path,
        source,
        vfs,
        None,
        None,
        include_wat_comments,
    )
}

fn parse_profile(profile: &str) -> Option<BuildProfile> {
    match profile {
        "debug" => Some(BuildProfile::Debug),
        "release" => Some(BuildProfile::Release),
        _ => None,
    }
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

fn lookup_web_source(
    bundled_sources: &BTreeMap<PathBuf, &'static str>,
    overlay_sources: &BTreeMap<PathBuf, String>,
    path: &PathBuf,
) -> Option<String> {
    overlay_sources
        .get(path)
        .cloned()
        .or_else(|| bundled_sources.get(path).map(|source| (*source).to_string()))
}

fn compile_wasm_with_entry_and_profile_and_stdlib(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    stdlib_vfs: Option<JsValue>,
    profile: Option<BuildProfile>,
    include_wat_comments: bool,
) -> Result<CompiledWasm, String> {
    let stdlib_root = PathBuf::from("/stdlib");
    let bundled_sources = stdlib_sources(&stdlib_root);
    compile_wasm_with_bundled_sources(
        entry_path,
        source,
        &stdlib_root,
        &bundled_sources,
        vfs,
        stdlib_vfs,
        profile,
        include_wat_comments,
    )
}

fn compile_wasm_with_bundled_sources(
    entry_path: &str,
    source: &str,
    stdlib_root: &PathBuf,
    bundled_sources: &BTreeMap<PathBuf, &'static str>,
    vfs: Option<JsValue>,
    stdlib_vfs: Option<JsValue>,
    profile: Option<BuildProfile>,
    include_wat_comments: bool,
) -> Result<CompiledWasm, String> {
    compile_wasm_with_bundled_sources_and_cache(
        entry_path,
        source,
        stdlib_root,
        bundled_sources,
        vfs,
        stdlib_vfs,
        profile,
        include_wat_comments,
        None,
        None,
        None,
        None,
        None,
    )
}

fn probe_nepl_meta_dependency_edges_pre_typecheck(
    store: &mut NeplMetaArtifactStore,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    probes: &[NeplMetaDependencyEdgePreTypecheckProbe],
) {
    for probe in probes {
        let envelope =
            nepl_meta_artifact_pre_typecheck_envelope_for_module_surface_with_source_identity(
                CompileTarget::Wasm,
                profile,
                stdlib_content_hash,
                Some(probe.dependency_public_surface_hash),
                probe.target_source_key_hash,
                probe.target_source_capability_policy_set_hash,
                &probe.target_module_surface,
            );
        let _ = store.materializer_import_public_surface_pre_typecheck_edge_probe_mvp(
            probe.target_module_path.as_str(),
            envelope,
            probe.import_clause.as_ref(),
        );
    }
}

fn compile_wasm_with_bundled_sources_and_cache(
    entry_path: &str,
    source: &str,
    stdlib_root: &PathBuf,
    bundled_sources: &BTreeMap<PathBuf, &'static str>,
    vfs: Option<JsValue>,
    stdlib_vfs: Option<JsValue>,
    profile: Option<BuildProfile>,
    include_wat_comments: bool,
    loader_cache: Option<&mut LoaderSessionCache>,
    resource_summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    preseed_resource_summary_proof_artifact: Option<&ResourceSummaryProofArtifact>,
    stage_timings: Option<&mut CompileStageTimings>,
    nepl_meta_artifact_store: Option<&mut NeplMetaArtifactStore>,
) -> Result<CompiledWasm, String> {
    let mut overlay_sources = BTreeMap::new();
    // stdlib 差し替えが指定された場合は、先に上書きで適用する
    merge_vfs_sources(&mut overlay_sources, stdlib_vfs);
    // 呼び出し元 VFS は最後に適用する
    merge_vfs_sources(&mut overlay_sources, vfs);
    let overlay_overrides_stdlib = overlay_sources
        .keys()
        .any(|path| path.starts_with(stdlib_root));
    let mut loader_cache = if overlay_overrides_stdlib {
        if let Some(cache) = loader_cache {
            cache.record_stdlib_override_bypass();
        }
        None
    } else {
        loader_cache
    };
    let mut resource_summary_value_cache = if overlay_overrides_stdlib {
        None
    } else {
        resource_summary_value_cache
    };
    let preseed_resource_summary_proof_artifact = if overlay_overrides_stdlib {
        None
    } else {
        preseed_resource_summary_proof_artifact
    };

    let mut loader = Loader::new(stdlib_root.clone());
    let mut provider = |path: &PathBuf| {
        lookup_web_source(&bundled_sources, &overlay_sources, path).ok_or_else(|| {
            let msg = format!(
                "missing source: {}. Available sources: {:?}",
                path.display(),
                overlay_sources.keys().collect::<Vec<_>>()
            );
            nepl_core::loader::LoaderError::Io(msg)
        })
    };
    let collect_nepl_meta_edge_probes = match nepl_meta_artifact_store.as_ref() {
        Some(store) => !store.is_empty(),
        None => false,
    };
    let loaded = if let Some(cache) = loader_cache.as_deref_mut() {
        if collect_nepl_meta_edge_probes {
            loader.load_inline_with_provider_and_cache_collecting_nepl_meta_edge_probes(
                PathBuf::from(entry_path),
                source.to_string(),
                &mut provider,
                cache,
            )
        } else {
            loader.load_inline_with_provider_and_cache(
                PathBuf::from(entry_path),
                source.to_string(),
                &mut provider,
                cache,
            )
        }
    } else {
        loader.load_inline_with_provider(PathBuf::from(entry_path), source.to_string(), &mut provider)
    }
    .map_err(|e| render_loader_error(e, loader.source_map()))?;
    // dependency aggregate は compile path が実際に必要とする namespace key 入力である。
    // prewarm では計算せず、stdlib overlay で cache を bypass した場合も利用しない。
    let dependency_public_surface_hash = if let Some(cache) = loader_cache.as_deref_mut() {
        Some(
            loader
                .root_dependency_aggregate_public_surface_hash_for_source_with_cache(
                    PathBuf::from(entry_path),
                    source,
                    &mut provider,
                    cache,
                )
                .map_err(|e| render_loader_error(e, loader.source_map()))?,
        )
    } else {
        None
    };
    let options = CompileOptions {
        target: None,
        verbose: false,
        profile,
    };
    let artifact_options = CompilationArtifactOptions {
        include_wat_comments,
    };
    let module_surface = loaded.module_surface.clone();
    let stdlib_content_hash = bundled_stdlib_hash_u64();
    if let (Some(store), Some(surface)) = (nepl_meta_artifact_store, module_surface.as_ref()) {
        if !store.is_empty() {
            let active_profile = profile.unwrap_or(BuildProfile::default_source_profile());
            if let Ok(envelope) = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
                CompileTarget::Wasm,
                active_profile,
                stdlib_content_hash,
                dependency_public_surface_hash,
                Some(&loaded.source_map),
                surface,
            ) {
                // この probe は body skip ではなく、既存 artifact が現在の loader 境界で
                // 再投影可能かを測る観測点である。失敗しても通常 source fallback を保つ。
                let _ = store.materializer_import_public_surface_pre_typecheck_mvp(
                    surface.canonical_module_path.as_str(),
                    envelope,
                    None,
                );
            }
            probe_nepl_meta_dependency_edges_pre_typecheck(
                store,
                active_profile,
                stdlib_content_hash,
                &loaded.nepl_meta_edge_probes,
            );
        }
    }
    let proof_artifact_enabled =
        resource_summary_value_cache.is_some() && stdlib_content_hash.is_some();
    let resource_summary_proof_options = ResourceSummaryProofArtifactCacheOptions {
        preseed_artifact: if proof_artifact_enabled {
            preseed_resource_summary_proof_artifact
        } else {
            None
        },
        stdlib_content_hash: if proof_artifact_enabled {
            stdlib_content_hash
        } else {
            None
        },
    };
    let artifact = if let Some(stage_timings) = stage_timings {
        compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_neplproof_and_stage_timings(
            loaded.module,
            Some(&loaded.source_map),
            options,
            artifact_options,
            dependency_public_surface_hash,
            module_surface.as_ref(),
            resource_summary_value_cache.as_deref_mut(),
            resource_summary_proof_options,
            stage_timings,
            compile_stage_now_ms,
        )
    } else {
        compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_and_neplproof(
            loaded.module,
            Some(&loaded.source_map),
            options,
            artifact_options,
            dependency_public_surface_hash,
            module_surface.as_ref(),
            resource_summary_value_cache.as_deref_mut(),
            resource_summary_proof_options,
        )
    }
    .map_err(|e| render_core_error(e, &loaded.source_map))?;
    let resource_summary_proof_artifact = if proof_artifact_enabled {
        if let (Some(header), Some(cache)) = (
            artifact.resource_summary_proof_header,
            resource_summary_value_cache.as_deref(),
        ) {
            Some(cache.export_neplproof_artifact(header))
        } else {
            None
        }
    } else {
        None
    };
    Ok(CompiledWasm {
        wasm: artifact.wasm,
        wat_comments: artifact.wat_comments,
        nepl_meta_artifact: artifact.nepl_meta_artifact,
        resource_summary_proof_artifact,
        stdlib_overlay_used: overlay_overrides_stdlib,
    })
}

fn compile_stage_now_ms() -> f64 {
    let global = js_sys::global();
    if let Ok(performance) = Reflect::get(&global, &JsValue::from_str("performance")) {
        if let Ok(now) = Reflect::get(&performance, &JsValue::from_str("now")) {
            if let Ok(now) = now.dyn_into::<Function>() {
                if let Ok(value) = now.call0(&performance) {
                    if let Some(ms) = value.as_f64() {
                        return ms;
                    }
                }
            }
        }
    }
    js_sys::Date::now()
}

/// WASM instance 内で再利用するコンパイラセッション。
///
/// このセッションは、bundled stdlib の source table を保持し、compile 呼び出しごとの
/// table 再構築を避ける。parse/typecheck/Resource IR の query cache は次段階で追加するが、
/// 公開 API は最初から session 単位にしておくことで、Node runner や Web playground が
/// 後続の cache 改良を API 変更なしに受け取れるようにする。
#[wasm_bindgen]
pub struct CompilerSession {
    stdlib_root: PathBuf,
    bundled_sources: BTreeMap<PathBuf, &'static str>,
    loader_cache: RefCell<LoaderSessionCache>,
    resource_summary_value_cache: RefCell<ResourceSummaryValueCache>,
    nepl_meta_artifact: RefCell<Option<NeplMetaArtifact>>,
    nepl_meta_artifact_store: RefCell<NeplMetaArtifactStore>,
    resource_summary_proof_artifact: RefCell<Option<ResourceSummaryProofArtifact>>,
    compiled_output_cache: RefCell<Vec<CompiledOutputCacheEntry>>,
    prewarmed_import_surfaces: RefCell<BTreeMap<u64, usize>>,
    compiled_output_cache_hits: RefCell<usize>,
    compiled_output_cache_stores: RefCell<usize>,
    resource_summary_proof_artifact_preseed_candidates: RefCell<usize>,
    resource_summary_proof_artifact_stores: RefCell<usize>,
    prewarm_surface_hits: RefCell<usize>,
    prewarm_surface_stores: RefCell<usize>,
    last_compile_stage_timing_status: RefCell<&'static str>,
    last_compile_stage_timings: RefCell<Option<String>>,
}

#[derive(Clone)]
struct CompiledOutputCacheEntry {
    key: String,
    compiled: CompiledWasm,
}

const COMPILED_OUTPUT_CACHE_LIMIT: usize = 8;

#[wasm_bindgen]
impl CompilerSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CompilerSession {
        let stdlib_root = PathBuf::from("/stdlib");
        let bundled_sources = stdlib_sources(&stdlib_root);
        CompilerSession {
            stdlib_root,
            bundled_sources,
            loader_cache: RefCell::new(LoaderSessionCache::new(stdlib_hash())),
            resource_summary_value_cache: RefCell::new(ResourceSummaryValueCache::new()),
            nepl_meta_artifact: RefCell::new(None),
            nepl_meta_artifact_store: RefCell::new(NeplMetaArtifactStore::new()),
            resource_summary_proof_artifact: RefCell::new(None),
            compiled_output_cache: RefCell::new(Vec::new()),
            prewarmed_import_surfaces: RefCell::new(BTreeMap::new()),
            compiled_output_cache_hits: RefCell::new(0),
            compiled_output_cache_stores: RefCell::new(0),
            resource_summary_proof_artifact_preseed_candidates: RefCell::new(0),
            resource_summary_proof_artifact_stores: RefCell::new(0),
            prewarm_surface_hits: RefCell::new(0),
            prewarm_surface_stores: RefCell::new(0),
            last_compile_stage_timing_status: RefCell::new("not_started"),
            last_compile_stage_timings: RefCell::new(None),
        }
    }

    /// 現在の release artifact に埋め込まれている stdlib source 数を返す。
    ///
    /// Node 側の smoke test と診断表示で、session が bundled stdlib を保持していることを
    /// 確認するための軽量な観測点として使う。
    pub fn bundled_stdlib_file_count(&self) -> usize {
        self.bundled_sources.len()
    }

    /// 現在の release artifact に埋め込まれている stdlib content hash を返す。
    ///
    /// 長時間動く Node / Web session では mtime だけで鮮度を判定すると、同一時刻の
    /// 内容差し替えや process-local cache により stale artifact を見落とす可能性がある。
    /// 呼び出し元はこの hash と local stdlib tree の hash を比較して、bundled stdlib を
    /// 使ってよいかを決める。
    pub fn bundled_stdlib_hash(&self) -> String {
        stdlib_hash().to_string()
    }

    /// `CompilerSession` 内の loader cache 統計を JSON 文字列として返す。
    ///
    /// Web / Node 側では Rust の構造体を直接読めないため、warm compile が
    /// stdlib parsed module cache と arity surface cache を実際に踏んだかを
    /// 確認する観測点として使う。
    /// 値は累積統計であり、cache の正しさは path/hash key と loader 側の
    /// `FileId` 再投影によって担保する。
    pub fn loader_cache_stats_json(&self) -> String {
        let stats = self.loader_cache.borrow().stats();
        let resource_stats = self.resource_summary_value_cache.borrow().stats();
        let nepl_meta_artifact = self.nepl_meta_artifact.borrow();
        let nepl_meta_artifact_store = self.nepl_meta_artifact_store.borrow();
        let nepl_meta_artifact_store_stats = nepl_meta_artifact_store.stats();
        let nepl_meta_artifact_store_entries = nepl_meta_artifact_store.len();
        let proof_artifact = self.resource_summary_proof_artifact.borrow();
        let nepl_meta_header = nepl_meta_artifact
            .as_ref()
            .map(|artifact| artifact.header());
        let proof_counts = proof_artifact
            .as_ref()
            .map(|artifact| artifact.counts())
            .unwrap_or_default();
        let mut out = format!(
            "{{\"parsed_module_hits\":{},\"parsed_module_misses\":{},\"parsed_module_stores\":{},\"parsed_module_bypasses\":{},\"arity_surface_hits\":{},\"arity_surface_misses\":{},\"arity_surface_stores\":{},\"arity_surface_bypasses\":{},\"public_surface_hash_hits\":{},\"public_surface_hash_stores\":{},\"public_surface_hash_bypasses\":{},\"dependency_aggregate_public_surface_hash_hits\":{},\"dependency_aggregate_public_surface_hash_misses\":{},\"dependency_aggregate_public_surface_hash_stores\":{},\"dependency_aggregate_public_surface_hash_bypasses\":{},\"stdlib_override_bypasses\":{},\"compiled_output_cache_hits\":{},\"compiled_output_cache_stores\":{},\"prewarm_surface_hits\":{},\"prewarm_surface_stores\":{},\"resource_raw_alias_summary_recomputations\":{},\"resource_raw_alias_summary_count\":{},\"resource_i32_scalar_summary_recomputations\":{},\"resource_i32_scalar_summary_count\":{},\"resource_raw_init_summary_recomputations\":{},\"resource_raw_init_summary_count\":{},\"resource_collection_slot_summary_recomputations\":{},\"resource_collection_slot_summary_count\":{},\"resource_initialized_function_checks\":{},\"resource_initialized_function_check_ops\":{},\"resource_summary_value_hits\":{},\"resource_summary_value_misses\":{},\"resource_summary_value_stores\":{},\"resource_summary_value_bypasses\":{},\"resource_summary_value_replay_hits\":{},\"resource_summary_value_replay_bypasses\":{},\"resource_summary_value_replayed_ops\":{},\"resource_summary_value_lazy_pass_hits\":{},\"resource_summary_value_lazy_pass_ops\":{},\"resource_summary_value_recomputed_ops\":{},\"resource_summary_value_drop_traversal_forall_recomputed_ops\":{},\"resource_summary_value_raw_alias_return_entry_recomputed_ops\":{},\"resource_summary_value_i32_scalar_return_facts_recomputed_ops\":{},\"resource_summary_value_raw_init_param_facts_recomputed_ops\":{},\"resource_summary_value_drop_traversal_forall_hits\":{},\"resource_summary_value_drop_traversal_forall_stores\":{},\"resource_summary_value_drop_traversal_forall_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_hits\":{},\"resource_summary_value_raw_alias_return_entry_stores\":{},\"resource_summary_value_raw_alias_return_entry_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_dependency_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_missing_source_policy_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_unstable_key_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_context_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_index_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_projection_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_type_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_return_projection_bypasses\":{},\"resource_summary_value_raw_alias_return_entry_reprojection_value_return_type_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_hits\":{},\"resource_summary_value_i32_scalar_return_facts_stores\":{},\"resource_summary_value_i32_scalar_return_facts_misses\":{},\"resource_summary_value_i32_scalar_return_facts_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_dependency_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_missing_source_policy_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_unstable_key_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_unstable_entry_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_unstable_entry_return_projection_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_unstable_entry_parameter_projection_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_unstable_entry_scalar_type_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_context_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses\":{},\"resource_summary_value_i32_scalar_return_facts_replay_missing_source_policy_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_unstable_key_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_reprojection_context_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_return_projection_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_parameter_projection_functions\":{},\"resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_scalar_type_functions\":{},\"resource_summary_value_initialized_function_check_hits\":{},\"resource_summary_value_initialized_function_check_stores\":{},\"resource_summary_value_initialized_function_check_bypasses\":{},\"resource_summary_value_initialized_function_check_dependency_bypasses\":{},\"resource_summary_value_initialized_function_check_diagnostic_bypasses\":{},\"resource_summary_value_initialized_function_check_missing_source_policy_bypasses\":{},\"resource_summary_value_initialized_function_check_unstable_key_bypasses\":{},\"resource_summary_value_initialized_function_check_unstable_entry_bypasses\":{},\"resource_summary_value_initialized_function_check_unstable_entry_auto_drop_bypasses\":{},\"resource_summary_value_initialized_function_check_unstable_entry_place_bypasses\":{},\"resource_summary_value_initialized_function_check_unstable_entry_type_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_context_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_place_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_type_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_place_type_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_projection_result_type_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_cell_state_type_bypasses\":{},\"resource_summary_value_initialized_function_check_reprojection_value_collection_slot_state_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_hits\":{},\"resource_summary_value_raw_init_param_facts_stores\":{},\"resource_summary_value_raw_init_param_facts_bypasses\":{},\"resource_summary_value_raw_init_param_facts_incomplete_leaf_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_bypasses\":{},\"resource_summary_value_raw_init_param_facts_missing_source_policy_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_key_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_graph_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_identity_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_body_hash_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_source_policy_bypasses\":{},\"resource_summary_value_raw_init_param_facts_dependency_type_boundary_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_entry_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_entry_surface_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_projection_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_unstable_entry_param_release_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_context_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_empty_entry_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_projection_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_stable_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_result_type_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_release_projection_bypasses\":{},\"resource_summary_value_raw_init_param_facts_reprojection_value_param_release_type_bypasses\":{}}}",
            stats.parsed_module_hits,
            stats.parsed_module_misses,
            stats.parsed_module_stores,
            stats.parsed_module_bypasses,
            stats.arity_surface_hits,
            stats.arity_surface_misses,
            stats.arity_surface_stores,
            stats.arity_surface_bypasses,
            stats.public_surface_hash_hits,
            stats.public_surface_hash_stores,
            stats.public_surface_hash_bypasses,
            stats.dependency_aggregate_public_surface_hash_hits,
            stats.dependency_aggregate_public_surface_hash_misses,
            stats.dependency_aggregate_public_surface_hash_stores,
            stats.dependency_aggregate_public_surface_hash_bypasses,
            stats.stdlib_override_bypasses,
            *self.compiled_output_cache_hits.borrow(),
            *self.compiled_output_cache_stores.borrow(),
            *self.prewarm_surface_hits.borrow(),
            *self.prewarm_surface_stores.borrow(),
            resource_stats.resource_raw_alias_summary_recomputations,
            resource_stats.resource_raw_alias_summary_count,
            resource_stats.resource_i32_scalar_summary_recomputations,
            resource_stats.resource_i32_scalar_summary_count,
            resource_stats.resource_raw_init_summary_recomputations,
            resource_stats.resource_raw_init_summary_count,
            resource_stats.resource_collection_slot_summary_recomputations,
            resource_stats.resource_collection_slot_summary_count,
            resource_stats.resource_initialized_function_checks,
            resource_stats.resource_initialized_function_check_ops,
            resource_stats.resource_summary_value_hits,
            resource_stats.resource_summary_value_misses,
            resource_stats.resource_summary_value_stores,
            resource_stats.resource_summary_value_bypasses,
            resource_stats.resource_summary_value_replay_hits,
            resource_stats.resource_summary_value_replay_bypasses,
            resource_stats.resource_summary_value_replayed_ops,
            resource_stats.resource_summary_value_lazy_pass_hits,
            resource_stats.resource_summary_value_lazy_pass_ops,
            resource_stats.resource_summary_value_recomputed_ops,
            resource_stats.resource_summary_value_drop_traversal_forall_recomputed_ops,
            resource_stats.resource_summary_value_raw_alias_return_entry_recomputed_ops,
            resource_stats.resource_summary_value_i32_scalar_return_facts_recomputed_ops,
            resource_stats.resource_summary_value_raw_init_param_facts_recomputed_ops,
            resource_stats.resource_summary_value_drop_traversal_forall_hits,
            resource_stats.resource_summary_value_drop_traversal_forall_stores,
            resource_stats.resource_summary_value_drop_traversal_forall_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_hits,
            resource_stats.resource_summary_value_raw_alias_return_entry_stores,
            resource_stats.resource_summary_value_raw_alias_return_entry_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_dependency_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_missing_source_policy_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_unstable_key_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_context_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_index_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_projection_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_type_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_return_projection_bypasses,
            resource_stats.resource_summary_value_raw_alias_return_entry_reprojection_value_return_type_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_hits,
            resource_stats.resource_summary_value_i32_scalar_return_facts_stores,
            resource_stats.resource_summary_value_i32_scalar_return_facts_misses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_dependency_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_missing_source_policy_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_unstable_key_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_unstable_entry_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_unstable_entry_return_projection_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_unstable_entry_parameter_projection_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_unstable_entry_scalar_type_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_context_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_missing_source_policy_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_unstable_key_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_reprojection_context_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_return_projection_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_parameter_projection_functions,
            resource_stats.resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_scalar_type_functions,
            resource_stats.resource_summary_value_initialized_function_check_hits,
            resource_stats.resource_summary_value_initialized_function_check_stores,
            resource_stats.resource_summary_value_initialized_function_check_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_dependency_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_diagnostic_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_missing_source_policy_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_unstable_key_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_unstable_entry_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_unstable_entry_auto_drop_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_unstable_entry_place_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_unstable_entry_type_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_context_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_place_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_type_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_place_type_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_projection_result_type_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_cell_state_type_bypasses,
            resource_stats.resource_summary_value_initialized_function_check_reprojection_value_collection_slot_state_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_hits,
            resource_stats.resource_summary_value_raw_init_param_facts_stores,
            resource_stats.resource_summary_value_raw_init_param_facts_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_incomplete_leaf_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_missing_source_policy_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_key_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_graph_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_identity_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_body_hash_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_source_policy_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_dependency_type_boundary_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_entry_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_entry_surface_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_projection_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_unstable_entry_param_release_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_context_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_empty_entry_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_projection_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_stable_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_result_type_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_release_projection_bypasses,
            resource_stats.resource_summary_value_raw_init_param_facts_reprojection_value_param_release_type_bypasses,
        );
        out.pop();
        out.push_str(",\"resource_static_function_count\":");
        out.push_str(&resource_stats.resource_static_function_count.to_string());
        out.push_str(",\"resource_static_op_count\":");
        out.push_str(&resource_stats.resource_static_op_count.to_string());
        out.push_str(",\"resource_owner_obligation_function_checks\":");
        out.push_str(&resource_stats.resource_owner_obligation_function_checks.to_string());
        out.push_str(",\"resource_owner_obligation_function_check_ops\":");
        out.push_str(&resource_stats.resource_owner_obligation_function_check_ops.to_string());
        out.push_str(",\"resource_owner_return_summary_recomputations\":");
        out.push_str(&resource_stats.resource_owner_return_summary_recomputations.to_string());
        out.push_str(",\"resource_owner_return_summary_count\":");
        out.push_str(&resource_stats.resource_owner_return_summary_count.to_string());
        out.push_str(",\"resource_owner_return_summary_pass_cache_skip_functions\":");
        out.push_str(&resource_stats.resource_owner_return_summary_pass_cache_skip_functions.to_string());
        out.push_str(",\"resource_summary_value_drop_traversal_forall_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_drop_traversal_forall_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_drop_traversal_forall_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_drop_traversal_forall_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_drop_traversal_forall_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_drop_traversal_forall_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_alias_return_entry_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_alias_return_entry_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_alias_return_entry_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_alias_return_entry_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_alias_return_entry_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_alias_return_entry_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_alias_return_entry_plan_skip_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_alias_return_entry_plan_skip_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_alias_return_entry_plan_skip_ops\":");
        out.push_str(&resource_stats.resource_summary_value_raw_alias_return_entry_plan_skip_ops.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_plan_skip_functions\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_plan_skip_functions.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_plan_skip_ops\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_plan_skip_ops.to_string());
        out.push_str(",\"resource_summary_value_raw_init_param_facts_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_init_param_facts_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_init_param_facts_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_init_param_facts_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_init_param_facts_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_init_param_facts_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_init_param_facts_plan_skip_functions\":");
        out.push_str(&resource_stats.resource_summary_value_raw_init_param_facts_plan_skip_functions.to_string());
        out.push_str(",\"resource_summary_value_raw_init_param_facts_plan_skip_ops\":");
        out.push_str(&resource_stats.resource_summary_value_raw_init_param_facts_plan_skip_ops.to_string());
        out.push_str(",\"resource_summary_value_initialized_function_check_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_initialized_function_check_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_initialized_function_check_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_initialized_function_check_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_initialized_function_check_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_initialized_function_check_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_initialized_function_check_plan_skip_functions\":");
        out.push_str(&resource_stats.resource_summary_value_initialized_function_check_plan_skip_functions.to_string());
        out.push_str(",\"resource_summary_value_initialized_function_check_plan_skip_ops\":");
        out.push_str(&resource_stats.resource_summary_value_initialized_function_check_plan_skip_ops.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_hits\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_hits.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_stores\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_stores.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_bypasses.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_replay_probe_functions\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_replay_probe_functions.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_replay_hit_functions\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_replay_hit_functions.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_replay_miss_functions\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_replay_miss_functions.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_dependency_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_dependency_bypasses.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_diagnostic_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_diagnostic_bypasses.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_missing_source_policy_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_missing_source_policy_bypasses.to_string());
        out.push_str(",\"resource_summary_value_owner_obligation_check_unstable_key_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_owner_obligation_check_unstable_key_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses.to_string());
        out.push_str(",\"resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses\":");
        out.push_str(&resource_stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses.to_string());
        out.push_str(",\"nepl_meta_artifact_present\":");
        out.push_str(if nepl_meta_artifact.is_some() {
            "true"
        } else {
            "false"
        });
        out.push_str(",\"nepl_meta_artifact_public_entries\":");
        out.push_str(
            &nepl_meta_artifact
                .as_ref()
                .map(|artifact| artifact.entry_count())
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_typed_public_signature_hash\":");
        out.push_str(
            &nepl_meta_header
                .map(|header| header.typed_public_signature_hash)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_source_key_hash\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.source_key_hash)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_module_dependency_edges\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.module_dependency_edge_count)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_module_surface_hash\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.module_surface_hash)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_local_exports\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.local_export_count)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_reexport_projections\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.reexport_projection_count)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_export_surface_hash\":");
        out.push_str(
            &nepl_meta_header
                .and_then(|header| header.export_surface_hash)
                .unwrap_or(0)
                .to_string(),
        );
        let nepl_meta_materializer_mvp_reject = nepl_meta_artifact
            .as_ref()
            .and_then(|artifact| artifact.materializer_mvp_reject());
        out.push_str(",\"nepl_meta_artifact_materializer_mvp_ready\":");
        let materializer_mvp_ready =
            nepl_meta_artifact.is_some() && nepl_meta_materializer_mvp_reject.is_none();
        out.push_str(if materializer_mvp_ready {
            "true"
        } else {
            "false"
        });
        out.push_str(",\"nepl_meta_artifact_materializer_mvp_reject_code\":");
        out.push_str(
            &nepl_meta_materializer_mvp_reject
                .as_ref()
                .map(|reject| reject.code())
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_structured_public_surface_entries\":");
        out.push_str(
            &nepl_meta_header
                .map(|header| header.structured_public_surface_entry_count)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_structured_public_surface_hash\":");
        out.push_str(
            &nepl_meta_header
                .map(|header| header.structured_public_surface_hash)
                .unwrap_or(0)
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_payload_consistency_ok\":");
        out.push_str(
            if nepl_meta_artifact
                .as_ref()
                .map(|artifact| artifact.payload_consistency_reject().is_none())
                .unwrap_or(false)
            {
                "true"
            } else {
                "false"
            },
        );
        out.push_str(",\"nepl_meta_artifact_store_entries\":");
        out.push_str(&nepl_meta_artifact_store_entries.to_string());
        out.push_str(",\"nepl_meta_artifact_store_stores\":");
        out.push_str(&nepl_meta_artifact_store_stats.stores.to_string());
        out.push_str(",\"nepl_meta_artifact_store_rejects\":");
        out.push_str(&nepl_meta_artifact_store_stats.store_rejects.to_string());
        out.push_str(",\"nepl_meta_artifact_store_hits\":");
        out.push_str(&nepl_meta_artifact_store_stats.hits.to_string());
        out.push_str(",\"nepl_meta_artifact_store_misses\":");
        out.push_str(&nepl_meta_artifact_store_stats.misses.to_string());
        out.push_str(",\"nepl_meta_artifact_store_payload_rejects\":");
        out.push_str(&nepl_meta_artifact_store_stats.payload_rejects.to_string());
        out.push_str(",\"nepl_meta_artifact_store_compatibility_rejects\":");
        out.push_str(&nepl_meta_artifact_store_stats.compatibility_rejects.to_string());
        out.push_str(",\"nepl_meta_artifact_store_projection_rejects\":");
        out.push_str(&nepl_meta_artifact_store_stats.projection_rejects.to_string());
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_attempts\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_attempts
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_projected\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_projected
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_missing_artifacts\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_missing_artifacts
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_payload_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_payload_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_compatibility_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_compatibility_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_projection_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_projection_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_probe_projected_entries\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_probe_projected_entries
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_probe_reject_kind\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_probe_reject_kind
                .code()
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_probe_reject_code\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_probe_reject_code
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_probe_projected_entries\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_probe_projected_entries
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_attempts\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_attempts
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_projected\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_projected
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_missing_artifacts\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_missing_artifacts
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_payload_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_payload_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_compatibility_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_compatibility_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_projection_rejects\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_projection_rejects
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_pre_typecheck_edge_probe_projected_entries\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .pre_typecheck_edge_probe_projected_entries
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_kind\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_edge_probe_reject_kind
                .code()
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_code\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_edge_probe_reject_code
                .to_string(),
        );
        out.push_str(",\"nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projected_entries\":");
        out.push_str(
            &nepl_meta_artifact_store_stats
                .last_pre_typecheck_edge_probe_projected_entries
                .to_string(),
        );
        out.push_str(",\"resource_summary_proof_artifact_present\":");
        out.push_str(if proof_artifact.is_some() { "true" } else { "false" });
        out.push_str(",\"resource_summary_proof_artifact_preseed_candidates\":");
        out.push_str(
            &self
                .resource_summary_proof_artifact_preseed_candidates
                .borrow()
                .to_string(),
        );
        out.push_str(",\"resource_summary_proof_artifact_stores\":");
        out.push_str(
            &self
                .resource_summary_proof_artifact_stores
                .borrow()
                .to_string(),
        );
        out.push_str(",\"resource_summary_proof_artifact_total_entries\":");
        out.push_str(&proof_counts.total_entries().to_string());
        out.push_str(",\"resource_summary_proof_artifact_drop_traversal_forall_leaf_entries\":");
        out.push_str(&proof_counts.drop_traversal_forall_leaf_entries.to_string());
        out.push_str(",\"resource_summary_proof_artifact_raw_alias_return_entries\":");
        out.push_str(&proof_counts.raw_alias_return_entries.to_string());
        out.push_str(",\"resource_summary_proof_artifact_i32_scalar_return_facts_entries\":");
        out.push_str(&proof_counts.i32_scalar_return_facts_entries.to_string());
        out.push_str(",\"resource_summary_proof_artifact_initialized_function_check_entries\":");
        out.push_str(&proof_counts.initialized_function_check_entries.to_string());
        out.push_str(",\"resource_summary_proof_artifact_owner_obligation_check_entries\":");
        out.push_str(&proof_counts.owner_obligation_check_entries.to_string());
        out.push_str(",\"resource_summary_proof_artifact_raw_init_complete_leaf_entries\":");
        out.push_str(&proof_counts.raw_init_complete_leaf_entries.to_string());
        out.push_str(",\"resource_summary_proof_stdlib_hash_u64_parse_ok\":");
        out.push_str(if bundled_stdlib_hash_u64().is_some() {
            "true"
        } else {
            "false"
        });
        out.push_str(",\"compile_stage_timing_status\":\"");
        out.push_str(*self.last_compile_stage_timing_status.borrow());
        out.push('"');
        out.push_str(",\"compile_stage_timings\":");
        if let Some(stage_timings) = self.last_compile_stage_timings.borrow().as_deref() {
            out.push_str(stage_timings);
        } else {
            out.push_str("null");
        }
        out.push('}');
        out
    }

    /// Loader cache を明示的に空にする。
    ///
    /// 通常の Web session では artifact refresh 時に Worker ごと作り直すが、
    /// Node の regression test では同じ `CompilerSession` で cold/warm 境界を
    /// 固定したいため、cache の寿命を観測可能にしておく。
    pub fn clear_loader_cache(&self) {
        self.loader_cache.borrow_mut().clear();
        self.resource_summary_value_cache.borrow_mut().clear();
        *self.nepl_meta_artifact.borrow_mut() = None;
        self.nepl_meta_artifact_store.borrow_mut().clear();
        *self.resource_summary_proof_artifact.borrow_mut() = None;
        self.compiled_output_cache.borrow_mut().clear();
        self.prewarmed_import_surfaces.borrow_mut().clear();
        *self.compiled_output_cache_hits.borrow_mut() = 0;
        *self.compiled_output_cache_stores.borrow_mut() = 0;
        *self
            .resource_summary_proof_artifact_preseed_candidates
            .borrow_mut() = 0;
        *self.resource_summary_proof_artifact_stores.borrow_mut() = 0;
        *self.prewarm_surface_hits.borrow_mut() = 0;
        *self.prewarm_surface_stores.borrow_mut() = 0;
        *self.last_compile_stage_timing_status.borrow_mut() = "not_started";
        *self.last_compile_stage_timings.borrow_mut() = None;
    }

    /// entry source から到達する bundled stdlib の loader cache を warm する。
    ///
    /// ここで行うのは loader / parser 境界の query prewarm だけであり、typed HIR、
    /// `ImportResolution`、Resource IR、codegen fragment は保持しない。対象は root
    /// source の default prelude / prelude / import / include から解決できる stdlib
    /// closure に限定し、bundled artifact のファイル一覧を総なめしない。
    ///
    /// dependency aggregate public surface hash は typed public surface cache の
    /// invalidation key として使う設計段階の artifact であり、この関数では計算しない。
    /// Web playground の compile 前 prewarm は compile を始めるために必要な query だけを
    /// warm し、まだ消費していない将来用 artifact のために private implementation graph を
    /// 再帰的に歩かない。
    pub fn prewarm_loader_cache_for_source(
        &self,
        entry_path: &str,
        source: &str,
    ) -> Result<usize, JsValue> {
        let mut cache = self.loader_cache.borrow_mut();
        let loader = Loader::new(self.stdlib_root.clone());
        let (surface_hash, roots) = loader.root_prewarm_surface_for_source_with_cache(
            PathBuf::from(entry_path),
            source,
            &mut cache,
        );
        if let Some(warmed_count) = self
            .prewarmed_import_surfaces
            .borrow()
            .get(&surface_hash)
            .copied()
        {
            *self.prewarm_surface_hits.borrow_mut() += 1;
            return Ok(warmed_count);
        }
        let mut provider = |path: &PathBuf| {
            self.bundled_sources.get(path).map(|src| (*src).to_string()).ok_or_else(|| {
                nepl_core::loader::LoaderError::Io(format!(
                    "missing bundled stdlib source during prewarm: {}",
                    path.display()
                ))
            })
        };
        let warmed = loader
            .prewarm_provider_cache(&roots, &mut provider, &mut cache)
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;
        self.prewarmed_import_surfaces
            .borrow_mut()
            .insert(surface_hash, warmed);
        *self.prewarm_surface_stores.borrow_mut() += 1;
        Ok(warmed)
    }

    pub fn compile_source_with_vfs_and_profile(
        &self,
        entry_path: &str,
        source: &str,
        vfs: JsValue,
        profile: &str,
    ) -> Result<Vec<u8>, JsValue> {
        *self.last_compile_stage_timing_status.borrow_mut() = "not_started";
        *self.last_compile_stage_timings.borrow_mut() = None;
        let parsed = parse_profile(profile)
            .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
        let key = compiled_output_cache_key(entry_path, source, &vfs, false, profile);
        if let Some(compiled) = self
            .compiled_output_cache
            .borrow()
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.compiled.clone())
        {
            self.remember_nepl_meta_artifact(compiled.nepl_meta_artifact.clone(), false);
            if let Some(artifact) = compiled.resource_summary_proof_artifact.clone() {
                *self.resource_summary_proof_artifact.borrow_mut() = Some(artifact);
            }
            *self.compiled_output_cache_hits.borrow_mut() += 1;
            *self.last_compile_stage_timing_status.borrow_mut() = "cache_hit";
            *self.last_compile_stage_timings.borrow_mut() = Some(String::from("[]"));
            return Ok(compiled.wasm);
        }
        let preseed_artifact = self.resource_summary_proof_artifact.borrow().clone();
        if preseed_artifact.is_some() {
            *self
                .resource_summary_proof_artifact_preseed_candidates
                .borrow_mut() += 1;
        }
        let mut stage_timings = CompileStageTimings::new();
        let compiled = {
            let mut cache = self.loader_cache.borrow_mut();
            let mut resource_summary_value_cache = self.resource_summary_value_cache.borrow_mut();
            let mut nepl_meta_artifact_store = self.nepl_meta_artifact_store.borrow_mut();
            match compile_wasm_with_bundled_sources_and_cache(
                entry_path,
                source,
                &self.stdlib_root,
                &self.bundled_sources,
                Some(vfs),
                None,
                Some(parsed),
                false,
                Some(&mut cache),
                Some(&mut resource_summary_value_cache),
                preseed_artifact.as_ref(),
                Some(&mut stage_timings),
                Some(&mut nepl_meta_artifact_store),
            ) {
                Ok(compiled) => compiled,
                Err(msg) => {
                    *self.last_compile_stage_timing_status.borrow_mut() = "failed";
                    *self.last_compile_stage_timings.borrow_mut() =
                        Some(stage_timings.to_json_array());
                    return Err(JsValue::from_str(&msg));
                }
            }
        };
        if let Some(artifact) = compiled.resource_summary_proof_artifact.clone() {
            *self.resource_summary_proof_artifact.borrow_mut() = Some(artifact);
            *self.resource_summary_proof_artifact_stores.borrow_mut() += 1;
        }
        self.remember_nepl_meta_artifact(
            compiled.nepl_meta_artifact.clone(),
            !compiled.stdlib_overlay_used,
        );
        *self.last_compile_stage_timing_status.borrow_mut() = "compiled";
        *self.last_compile_stage_timings.borrow_mut() = Some(stage_timings.to_json_array());
        self.store_compiled_output_cache_entry(key, compiled.clone());
        Ok(compiled.wasm)
    }

    pub fn compile_source_with_vfs_stdlib_and_profile(
        &self,
        entry_path: &str,
        source: &str,
        vfs: JsValue,
        stdlib_vfs: JsValue,
        profile: &str,
    ) -> Result<Vec<u8>, JsValue> {
        *self.last_compile_stage_timing_status.borrow_mut() = "not_started";
        *self.last_compile_stage_timings.borrow_mut() = None;
        let parsed = parse_profile(profile)
            .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
        self.loader_cache.borrow_mut().record_stdlib_override_bypass();
        self.nepl_meta_artifact_store.borrow_mut().clear();
        let mut stage_timings = CompileStageTimings::new();
        let compiled = match compile_wasm_with_bundled_sources_and_cache(
            entry_path,
            source,
            &self.stdlib_root,
            &self.bundled_sources,
            Some(vfs),
            Some(stdlib_vfs),
            Some(parsed),
            false,
            None,
            None,
            None,
            Some(&mut stage_timings),
            None,
        ) {
            Ok(compiled) => compiled,
            Err(msg) => {
                *self.last_compile_stage_timing_status.borrow_mut() = "failed";
                *self.last_compile_stage_timings.borrow_mut() =
                    Some(stage_timings.to_json_array());
                return Err(JsValue::from_str(&msg));
            }
        };
        *self.last_compile_stage_timing_status.borrow_mut() = "compiled";
        *self.last_compile_stage_timings.borrow_mut() = Some(stage_timings.to_json_array());
        self.remember_nepl_meta_artifact(compiled.nepl_meta_artifact.clone(), false);
        Ok(compiled.wasm)
    }

    /// bundled stdlib を保持したまま、複数 emit の compiler output を生成する。
    ///
    /// Web playground worker は編集可能な source overlay だけを `vfs` に渡し、
    /// read-only stdlib はこの session 内の `bundled_sources` を再利用する。
    /// これにより compile ごとの stdlib table 再構築と巨大 overlay 転送を避け、
    /// 後続の parse/typecheck query cache を同じ API 境界へ追加できる。
    pub fn compile_outputs_with_vfs(
        &self,
        entry_path: &str,
        source: &str,
        vfs: JsValue,
        emit: JsValue,
        attach_source: bool,
    ) -> Result<JsValue, JsValue> {
        *self.last_compile_stage_timing_status.borrow_mut() = "not_started";
        *self.last_compile_stage_timings.borrow_mut() = None;
        let emit_list = parse_emit_list(emit)?;
        let include_wat_comments = emit_list.iter().any(|kind| kind == "wat");
        let key = compiled_output_cache_key(entry_path, source, &vfs, include_wat_comments, "debug");
        if let Some(compiled) = self
            .compiled_output_cache
            .borrow()
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.compiled.clone())
        {
            self.remember_nepl_meta_artifact(compiled.nepl_meta_artifact.clone(), false);
            if let Some(artifact) = compiled.resource_summary_proof_artifact.clone() {
                *self.resource_summary_proof_artifact.borrow_mut() = Some(artifact);
            }
            *self.compiled_output_cache_hits.borrow_mut() += 1;
            *self.last_compile_stage_timing_status.borrow_mut() = "cache_hit";
            *self.last_compile_stage_timings.borrow_mut() = Some(String::from("[]"));
            return compile_outputs_from_compiled(
                &compiled,
                entry_path,
                source,
                emit_list,
                attach_source,
            );
        }
        let preseed_artifact = self.resource_summary_proof_artifact.borrow().clone();
        if preseed_artifact.is_some() {
            *self
                .resource_summary_proof_artifact_preseed_candidates
                .borrow_mut() += 1;
        }
        let mut stage_timings = CompileStageTimings::new();
        let compiled = {
            let mut loader_cache = self.loader_cache.borrow_mut();
            let mut resource_summary_value_cache = self.resource_summary_value_cache.borrow_mut();
            let mut nepl_meta_artifact_store = self.nepl_meta_artifact_store.borrow_mut();
            match compile_wasm_with_bundled_sources_and_cache(
                entry_path,
                source,
                &self.stdlib_root,
                &self.bundled_sources,
                Some(vfs),
                None,
                Some(BuildProfile::default_source_profile()),
                include_wat_comments,
                Some(&mut loader_cache),
                Some(&mut resource_summary_value_cache),
                preseed_artifact.as_ref(),
                Some(&mut stage_timings),
                Some(&mut nepl_meta_artifact_store),
            ) {
                Ok(compiled) => compiled,
                Err(msg) => {
                    *self.last_compile_stage_timing_status.borrow_mut() = "failed";
                    *self.last_compile_stage_timings.borrow_mut() =
                        Some(stage_timings.to_json_array());
                    return Err(JsValue::from_str(&msg));
                }
            }
        };
        if let Some(artifact) = compiled.resource_summary_proof_artifact.clone() {
            *self.resource_summary_proof_artifact.borrow_mut() = Some(artifact);
            *self.resource_summary_proof_artifact_stores.borrow_mut() += 1;
        }
        self.remember_nepl_meta_artifact(
            compiled.nepl_meta_artifact.clone(),
            !compiled.stdlib_overlay_used,
        );
        *self.last_compile_stage_timing_status.borrow_mut() = "compiled";
        *self.last_compile_stage_timings.borrow_mut() = Some(stage_timings.to_json_array());
        self.store_compiled_output_cache_entry(key, compiled.clone());
        compile_outputs_from_compiled(&compiled, entry_path, source, emit_list, attach_source)
    }

    fn store_compiled_output_cache_entry(&self, key: String, compiled: CompiledWasm) {
        let mut cache = self.compiled_output_cache.borrow_mut();
        if cache.len() >= COMPILED_OUTPUT_CACHE_LIMIT {
            cache.remove(0);
        }
        cache.push(CompiledOutputCacheEntry { key, compiled });
        *self.compiled_output_cache_stores.borrow_mut() += 1;
    }

    /// 最後に生成または再利用した `.neplmeta` artifact を session に記録する。
    ///
    /// `store=true` の場合だけ module path keyed store へ保存する。compiled-output cache hit は
    /// 新しい compile artifact ではないため、store 統計を増やさず last artifact だけ更新する。
    /// stdlib override compile では将来の import materializer が通常 stdlib と取り違えないよう、
    /// store を clear してから last artifact だけ更新する。
    fn remember_nepl_meta_artifact(&self, artifact: NeplMetaArtifact, store: bool) {
        if store {
            let _ = self.nepl_meta_artifact_store.borrow_mut().store(artifact.clone());
        } else if artifact
            .module_surface()
            .is_some_and(|surface| surface.canonical_module_path.starts_with("/stdlib"))
        {
            self.nepl_meta_artifact_store.borrow_mut().clear();
        }
        *self.nepl_meta_artifact.borrow_mut() = Some(artifact);
    }
}

fn compiled_output_cache_key(
    entry_path: &str,
    source: &str,
    vfs: &JsValue,
    include_wat_comments: bool,
    profile: &str,
) -> String {
    let mut key = String::new();
    push_cache_key_part(&mut key, entry_path);
    push_cache_key_part(&mut key, profile);
    push_cache_key_part(&mut key, if include_wat_comments { "wat" } else { "wasm" });
    push_cache_key_part(&mut key, &compiled_source_cache_key_part(source));
    if vfs.is_object() {
        let mut entries = js_sys::Object::entries(&vfs.clone().into())
            .iter()
            .filter_map(|entry| {
                let pair = js_sys::Array::from(&entry);
                let path = pair.get(0).as_string().unwrap_or_default();
                if path.is_empty() {
                    return None;
                }
                let content = pair.get(1).as_string().unwrap_or_default();
                Some((path, content))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (path, content) in entries {
            push_cache_key_part(&mut key, &path);
            push_cache_key_part(&mut key, &compiled_source_cache_key_part(&content));
        }
    }
    key
}

fn push_cache_key_part(key: &mut String, value: &str) {
    key.push_str(&value.len().to_string());
    key.push(':');
    key.push_str(value);
    key.push('\n');
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
        false,
    )
    .map(|a| a.wasm)
    .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_profile(source: &str, profile: &str) -> Result<Vec<u8>, JsValue> {
    let parsed = parse_profile(profile)
        .ok_or_else(|| JsValue::from_str("invalid profile (expected 'debug' or 'release')"))?;
    compile_wasm_with_entry_and_profile_and_stdlib(
        "/virtual/entry.nepl",
        source,
        None,
        None,
        Some(parsed),
        false,
    )
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
    compile_wasm_with_entry_and_profile_and_stdlib(
        entry_path,
        source,
        Some(vfs),
        None,
        Some(parsed),
        false,
    )
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
        false,
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

fn render_loader_error(err: LoaderError, sm: &SourceMap) -> String {
    match err {
        LoaderError::Io(msg) => format!("IO error: {}", msg),
        LoaderError::Core(core) => render_core_error(core, sm),
    }
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) -> String {
    let mut out = String::new();
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const BLUE: &str = "\x1b[34m";

    for d in diags {
        let (severity_str, severity_color) = match d.severity {
            Severity::Error => ("error", RED),
            Severity::Warning => ("warning", YELLOW),
        };
        let code_display = format!("[{}]", d.code.as_str());
        let primary = &d.primary;
        let (line, col) = sm
            .line_col(primary.span.file_id, primary.span.start)
            .unwrap_or((0, 0));
        let path = sm
            .path(primary.span.file_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());

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

fn stdlib_sources(root: &PathBuf) -> BTreeMap<PathBuf, &'static str> {
    let mut map = BTreeMap::new();
    for (path, src) in stdlib_entries() {
        map.insert(root.join(path), *src);
    }
    map
}

include!(concat!(env!("OUT_DIR"), "/stdlib_entries.rs"));

fn stdlib_entries() -> &'static [(&'static str, &'static str)] {
    STD_LIB_ENTRIES
}

fn stdlib_hash() -> &'static str {
    STD_LIB_HASH
}

fn bundled_stdlib_hash_u64() -> Option<u64> {
    let hex = stdlib_hash().strip_prefix("fnv1a64:")?;
    u64::from_str_radix(hex, 16).ok()
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

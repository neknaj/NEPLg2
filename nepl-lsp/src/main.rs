use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use nepl_core::nm::render_document_markdown;
use nepl_language::{
    analyze_loaded_semantics, default_stdlib_root, load_inline_module_with_provider,
    EditorDiagnostic, NameDefinitionInfo, SemanticExpressionInfo, SemanticsAnalysis, TextRange,
};
use serde_json::{json, Value};

#[derive(Default)]
struct ServerState {
    root_path: Option<PathBuf>,
    stdlib_root: Option<PathBuf>,
    open_documents: BTreeMap<String, DocumentState>,
}

#[derive(Clone)]
struct DocumentState {
    uri: String,
    path: PathBuf,
    text: String,
    analysis: SemanticsAnalysis,
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    let mut state = ServerState::default();

    while let Some(message) = read_message(&mut reader)? {
        if let Err(error) = handle_message(&mut state, &mut writer, message) {
            let _ = log_message(
                &mut writer,
                1,
                &format!("nepl-lsp internal error: {error:#}"),
            );
        }
    }

    Ok(())
}

fn handle_message(state: &mut ServerState, writer: &mut dyn Write, message: Value) -> Result<()> {
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id").cloned();
    let params = message.get("params").cloned().unwrap_or(Value::Null);

    match method {
        Some("initialize") => {
            handle_initialize(state, writer, id, params)?;
        }
        Some("initialized") => {}
        Some("shutdown") => {
            write_result(writer, id, Value::Null)?;
        }
        Some("exit") => {}
        Some("textDocument/didOpen") => {
            handle_did_open(state, writer, params)?;
        }
        Some("textDocument/didChange") => {
            handle_did_change(state, writer, params)?;
        }
        Some("textDocument/didSave") => {
            handle_did_save(state, writer, params)?;
        }
        Some("textDocument/hover") => {
            let result = handle_hover(state, params)?;
            write_result(writer, id, result)?;
        }
        Some("textDocument/definition") => {
            let result = handle_definition(state, params)?;
            write_result(writer, id, result)?;
        }
        Some("textDocument/semanticTokens/full") => {
            let result = handle_semantic_tokens(state, params)?;
            write_result(writer, id, result)?;
        }
        Some("textDocument/inlayHint") => {
            let result = handle_inlay_hints(state, params)?;
            write_result(writer, id, result)?;
        }
        Some("$/setTrace") => {}
        Some(other) => {
            let _ = log_message(writer, 3, &format!("unhandled method: {other}"));
            if id.is_some() {
                write_result(writer, id, Value::Null)?;
            }
        }
        None => {}
    }

    Ok(())
}

fn handle_initialize(
    state: &mut ServerState,
    writer: &mut dyn Write,
    id: Option<Value>,
    params: Value,
) -> Result<()> {
    let root_uri = params.get("rootUri").and_then(Value::as_str);
    let root_path = params.get("rootPath").and_then(Value::as_str);
    let initialization_options = params
        .get("initializationOptions")
        .cloned()
        .unwrap_or(Value::Null);

    state.root_path = root_uri
        .and_then(uri_to_path)
        .or_else(|| root_path.map(PathBuf::from));
    state.stdlib_root = initialization_options
        .get("stdlibRoot")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("NEPL_STDLIB_ROOT").map(PathBuf::from))
        .or_else(|| state.root_path.as_ref().map(default_stdlib_root));

    let result = json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true,
            "definitionProvider": true,
            "inlayHintProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": [
                        "namespace",
                        "type",
                        "struct",
                        "enum",
                        "typeParameter",
                        "parameter",
                        "variable",
                        "property",
                        "function",
                        "keyword",
                        "comment",
                        "string",
                        "number",
                        "operator"
                    ],
                    "tokenModifiers": []
                },
                "full": true
            }
        },
        "serverInfo": {
            "name": "nepl-lsp",
            "version": env!("CARGO_PKG_VERSION")
        }
    });
    write_result(writer, id, result)?;
    let _ = log_message(writer, 3, "nepl-lsp initialized");
    Ok(())
}

fn handle_did_open(state: &mut ServerState, writer: &mut dyn Write, params: Value) -> Result<()> {
    let doc = params
        .get("textDocument")
        .ok_or_else(|| anyhow!("missing textDocument"))?;
    let uri = doc
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing uri"))?
        .to_string();
    let text = doc
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing text"))?
        .to_string();
    update_document(state, writer, uri, text)
}

fn handle_did_change(state: &mut ServerState, writer: &mut dyn Write, params: Value) -> Result<()> {
    let uri = params
        .get("textDocument")
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing uri"))?
        .to_string();
    let text = params
        .get("contentChanges")
        .and_then(Value::as_array)
        .and_then(|changes| changes.last())
        .and_then(|change| change.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing changed text"))?
        .to_string();
    update_document(state, writer, uri, text)
}

fn handle_did_save(state: &mut ServerState, writer: &mut dyn Write, params: Value) -> Result<()> {
    let uri = params
        .get("textDocument")
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing uri"))?
        .to_string();
    let text = if let Some(text) = params.get("text").and_then(Value::as_str) {
        text.to_string()
    } else if let Some(document) = state.open_documents.get(&uri) {
        document.text.clone()
    } else {
        let path = uri_to_path(&uri).ok_or_else(|| anyhow!("unsupported uri: {uri}"))?;
        fs::read_to_string(path)?
    };
    update_document(state, writer, uri, text)
}

fn handle_hover(state: &mut ServerState, params: Value) -> Result<Value> {
    let (document, line, character) = lookup_document_and_position(state, &params)?;
    let hover = find_hover(document, line, character);
    Ok(hover.unwrap_or(Value::Null))
}

fn handle_definition(state: &mut ServerState, params: Value) -> Result<Value> {
    let (document, line, character) = lookup_document_and_position(state, &params)?;
    let location = find_definition(document, line, character);
    Ok(location.unwrap_or(Value::Null))
}

fn handle_semantic_tokens(state: &mut ServerState, params: Value) -> Result<Value> {
    let document = lookup_document(state, &params)?;
    Ok(build_semantic_tokens(&document.analysis))
}

fn handle_inlay_hints(state: &mut ServerState, params: Value) -> Result<Value> {
    let document = lookup_document(state, &params)?;
    Ok(build_inlay_hints(&document.analysis))
}

fn lookup_document_and_position<'a>(
    state: &'a ServerState,
    params: &Value,
) -> Result<(&'a DocumentState, usize, usize)> {
    let uri = params
        .get("textDocument")
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing uri"))?;
    let position = params
        .get("position")
        .or_else(|| params.get("range").and_then(|value| value.get("start")))
        .ok_or_else(|| anyhow!("missing position"))?;
    let line = position
        .get("line")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing line"))? as usize;
    let character = position
        .get("character")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing character"))? as usize;
    let document = state
        .open_documents
        .get(uri)
        .ok_or_else(|| anyhow!("document not opened: {uri}"))?;
    Ok((document, line, character))
}

fn lookup_document<'a>(state: &'a ServerState, params: &Value) -> Result<&'a DocumentState> {
    let uri = params
        .get("textDocument")
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing uri"))?;
    state
        .open_documents
        .get(uri)
        .ok_or_else(|| anyhow!("document not opened: {uri}"))
}

fn update_document(
    state: &mut ServerState,
    writer: &mut dyn Write,
    uri: String,
    text: String,
) -> Result<()> {
    let path = uri_to_path(&uri).ok_or_else(|| anyhow!("unsupported uri: {uri}"))?;
    let analysis = analyze_document(state, &path, &text)
        .with_context(|| format!("analyze document failed: {}", path.display()))?;
    let diagnostics = analysis
        .diagnostics
        .iter()
        .map(editor_diagnostic_to_lsp)
        .collect::<Vec<_>>();
    state.open_documents.insert(
        uri.clone(),
        DocumentState {
            uri: uri.clone(),
            path,
            text,
            analysis,
        },
    );
    write_notification(
        writer,
        "textDocument/publishDiagnostics",
        json!({
            "uri": uri,
            "diagnostics": diagnostics
        }),
    )
}

fn analyze_document(
    state: &ServerState,
    entry_path: &Path,
    source: &str,
) -> Result<SemanticsAnalysis> {
    let stdlib_root = state
        .stdlib_root
        .clone()
        .or_else(|| find_repo_root(entry_path).map(default_stdlib_root))
        .ok_or_else(|| anyhow!("failed to resolve stdlib root"))?;

    let entry_path = entry_path.to_path_buf();
    let provider_entry_path = entry_path.clone();
    let entry_source = source.to_string();
    let mut provider = move |path: &PathBuf| -> Result<String, nepl_core::loader::LoaderError> {
        if *path == provider_entry_path {
            return Ok(entry_source.clone());
        }
        fs::read_to_string(path).map_err(|error| {
            nepl_core::loader::LoaderError::Io(format!("{}: {}", path.display(), error))
        })
    };

    let loaded = load_inline_module_with_provider(
        stdlib_root,
        entry_path.clone(),
        source.to_string(),
        &mut provider,
    )?;
    Ok(analyze_loaded_semantics(source, &loaded))
}

fn find_hover(document: &DocumentState, line: usize, character: usize) -> Option<Value> {
    let hint = document.analysis.token_hints.iter().find(|hint| {
        range_contains_position(
            hint.ref_range.as_ref().or(hint.expression_range.as_ref()),
            line,
            character,
        )
    })?;

    let mut parts = Vec::new();
    if let Some(name) = &hint.name {
        parts.push(format!("symbol: {name}"));
    }
    if let Some(ty) = &hint.inferred_type {
        parts.push(format!("type: {ty}"));
    }
    if let Some(definition) = &hint.resolved_definition {
        parts.push(format!("definition: {}", definition.name));
        if let Some(doc) = render_hover_doc(definition) {
            parts.push(doc);
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(json!({
        "contents": {
            "kind": "markdown",
            "value": parts.join("\n\n")
        },
        "range": hint
            .ref_range
            .as_ref()
            .or(hint.expression_range.as_ref())
            .map(text_range_to_lsp)
    }))
}

fn render_hover_doc(definition: &NameDefinitionInfo) -> Option<String> {
    if let Some(doc) = &definition.doc_ast {
        let rendered = render_document_markdown(doc);
        if !rendered.is_empty() {
            return Some(rendered);
        }
    }
    definition.doc.clone()
}

fn find_definition(document: &DocumentState, line: usize, character: usize) -> Option<Value> {
    let hint = document
        .analysis
        .token_hints
        .iter()
        .find(|hint| range_contains_position(hint.ref_range.as_ref(), line, character))?;
    let definition = hint.resolved_definition.as_ref()?;
    let path = definition.range.path.as_ref()?;
    Some(json!({
        "uri": path_to_uri(path),
        "range": text_range_to_lsp(&definition.range)
    }))
}

fn build_semantic_tokens(analysis: &SemanticsAnalysis) -> Value {
    let mut encoded = Vec::<u32>::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    let mut emitted_any = false;

    for token in &analysis.tokens {
        let Some(token_type) = semantic_token_type(token, analysis) else {
            continue;
        };
        let line = token.range.start.line as u32;
        let start = token.range.start.column as u32;
        let length = token
            .range
            .end
            .column
            .saturating_sub(token.range.start.column) as u32;
        if length == 0 {
            continue;
        }

        let delta_line = if emitted_any {
            line.saturating_sub(prev_line)
        } else {
            line
        };
        let delta_start = if emitted_any && delta_line == 0 {
            start.saturating_sub(prev_start)
        } else {
            start
        };
        encoded.extend_from_slice(&[delta_line, delta_start, length, token_type, 0]);
        prev_line = line;
        prev_start = start;
        emitted_any = true;
    }

    json!({ "data": encoded })
}

fn semantic_token_type(
    token: &nepl_language::TokenInfo,
    analysis: &SemanticsAnalysis,
) -> Option<u32> {
    let idx = analysis
        .tokens
        .iter()
        .position(|candidate| candidate.range == token.range)?;
    let hint = analysis.token_hints.get(idx);

    let token_type = match token.kind.as_str() {
        "DocComment" => 10,
        "StringLiteral" => 11,
        "IntLiteral" | "FloatLiteral" | "BoolLiteral" => 12,
        "Colon" | "Semicolon" | "Pipe" | "Arrow" | "PathSep" | "At" | "Dot" | "Ampersand"
        | "Star" | "Minus" | "Equals" => 13,
        kind if kind.starts_with("Kw") || kind.starts_with("Dir") => 9,
        "Ident" => {
            match hint
                .and_then(|hint| hint.resolved_definition.as_ref())
                .map(|definition| definition.kind)
            {
                Some("fn") | Some("fn_alias") => 8,
                Some("param") => 5,
                Some("struct") => 2,
                Some("enum") => 3,
                Some("trait") => 1,
                _ => 6,
            }
        }
        _ => return None,
    };

    Some(token_type)
}

fn build_inlay_hints(analysis: &SemanticsAnalysis) -> Value {
    let hints = analysis
        .expressions
        .iter()
        .filter(|expr| should_emit_type_hint(expr))
        .map(expression_type_hint_to_lsp)
        .collect::<Vec<_>>();
    Value::Array(hints)
}

fn should_emit_type_hint(expr: &SemanticExpressionInfo) -> bool {
    !expr.inferred_type.is_empty()
        && expr.inferred_type != "()"
        && matches!(expr.kind, "Call" | "If" | "While" | "Block" | "Let" | "Set")
}

fn expression_type_hint_to_lsp(expr: &SemanticExpressionInfo) -> Value {
    json!({
        "position": {
            "line": expr.range.start.line,
            "character": expr.range.start.column
        },
        "label": format!("<{}>", expr.inferred_type),
        "kind": 1,
        "paddingRight": true
    })
}

fn editor_diagnostic_to_lsp(diagnostic: &EditorDiagnostic) -> Value {
    json!({
        "range": text_range_to_lsp(&diagnostic.range),
        "severity": match diagnostic.severity {
            nepl_core::diagnostic::Severity::Error => 1,
            nepl_core::diagnostic::Severity::Warning => 2,
        },
        "code": diagnostic.id,
        "source": "nepl-lsp",
        "message": diagnostic.message
    })
}

fn text_range_to_lsp(range: &TextRange) -> Value {
    json!({
        "start": {
            "line": range.start.line,
            "character": range.start.column
        },
        "end": {
            "line": range.end.line,
            "character": range.end.column
        }
    })
}

fn range_contains_position(range: Option<&TextRange>, line: usize, character: usize) -> bool {
    let Some(range) = range else {
        return false;
    };
    let starts_before =
        line > range.start.line || (line == range.start.line && character >= range.start.column);
    let ends_after =
        line < range.end.line || (line == range.end.line && character <= range.end.column);
    starts_before && ends_after
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }
    let raw = uri.trim_start_matches("file://");
    let decoded = percent_decode(raw);
    if cfg!(windows) && decoded.starts_with('/') && decoded.as_bytes().get(2) == Some(&b':') {
        Some(PathBuf::from(&decoded[1..]))
    } else {
        Some(PathBuf::from(decoded))
    }
}

fn path_to_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{}", percent_encode(&text))
    } else {
        format!("file:///{}", percent_encode(&text))
    }
}

fn percent_decode(input: &str) -> String {
    let mut out = String::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((hi * 16 + lo) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn find_repo_root(entry_path: &Path) -> Option<PathBuf> {
    let mut current = entry_path.parent()?.to_path_buf();
    loop {
        if current.join("Cargo.toml").is_file() && current.join("stdlib").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn read_message(reader: &mut dyn BufRead) -> Result<Option<Value>> {
    let mut content_length = None::<usize>;
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse()?);
            }
        }
    }

    let length = content_length.ok_or_else(|| anyhow!("missing Content-Length"))?;
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let message = serde_json::from_slice::<Value>(&body)?;
    Ok(Some(message))
}

fn write_message(writer: &mut dyn Write, value: &Value) -> Result<()> {
    let text = serde_json::to_vec(value)?;
    write!(writer, "Content-Length: {}\r\n\r\n", text.len())?;
    writer.write_all(&text)?;
    writer.flush()?;
    Ok(())
}

fn write_result(writer: &mut dyn Write, id: Option<Value>, result: Value) -> Result<()> {
    if let Some(id) = id {
        write_message(
            writer,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
        )
    } else {
        Ok(())
    }
}

fn write_notification(writer: &mut dyn Write, method: &str, params: Value) -> Result<()> {
    write_message(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }),
    )
}

fn log_message(writer: &mut dyn Write, typ: i32, message: &str) -> Result<()> {
    write_notification(
        writer,
        "window/logMessage",
        json!({
            "type": typ,
            "message": message
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_roundtrip_unix_style() {
        let path = PathBuf::from("/tmp/nepl test.nepl");
        let uri = path_to_uri(&path);
        assert_eq!(uri_to_path(&uri), Some(path));
    }

    #[test]
    fn inlay_hint_generation_emits_type_hints() {
        let analysis = nepl_language::analyze_semantics(
            "#no_prelude\nfn main <()->i32> ():\n    if true 1 2\n",
        );
        let hints = build_inlay_hints(&analysis);
        assert!(hints.as_array().is_some_and(|items| !items.is_empty()));
    }
}

use std::collections::BTreeMap;
use std::path::PathBuf;

use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::error::CoreError;
use nepl_core::loader::{Loader, SourceMap};
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_source(source: &str) -> Result<Vec<u8>, JsValue> {
    compile_with_entry("/virtual/entry.nepl", source)
        .map_err(|msg| JsValue::from_str(&msg))
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
pub fn compile_test(name: &str) -> Result<Vec<u8>, JsValue> {
    let src = test_sources()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, src)| *src)
        .ok_or_else(|| JsValue::from_str("unknown test"))?;
    compile_with_entry(&format!("/virtual/tests/{name}.nepl"), src)
        .map_err(|msg| JsValue::from_str(&msg))
}

fn compile_with_entry(entry_path: &str, source: &str) -> Result<Vec<u8>, String> {
    let stdlib_root = PathBuf::from("/stdlib");
    let sources = stdlib_sources(&stdlib_root);
    let mut loader = Loader::new(stdlib_root);
    let mut provider = |path: &PathBuf| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| nepl_core::loader::LoaderError::Io(format!("missing source: {}", path.display())))
    };
    let loaded = loader
        .load_inline_with_provider(PathBuf::from(entry_path), source.to_string(), &mut provider)
        .map_err(|e| e.to_string())?;
    let artifact = compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose: false,
            profile: None,
        },
    )
    .map_err(|e| render_core_error(e, &loaded.source_map))?;
    Ok(artifact.wasm)
}

fn render_core_error(err: CoreError, sm: &SourceMap) -> String {
    match err {
        CoreError::Diagnostics(diags) => render_diagnostics(&diags, sm),
        other => other.to_string(),
    }
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) -> String {
    let mut out = String::new();
    for d in diags {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
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
        out.push_str(&format!("{severity}{code_display}: {message}\n", message = d.message));
        out.push_str(&format!(" --> {path}:{line}:{col}\n", line = line + 1, col = col + 1));
        if let Some(line_str) = sm.line_str(primary.span.file_id, line) {
            out.push_str(&format!(
                "  {line_num:>4} | {text}\n",
                line_num = line + 1,
                text = line_str
            ));
            let caret_pos = col;
            out.push_str(&format!(
                "       | {spaces}{carets}\n",
                spaces = " ".repeat(caret_pos),
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
                " note: {p}:{line}:{col}: {msg}\n",
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

fn stdlib_entries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("std.nepl", include_str!("../../stdlib/std.nepl")),
        ("std/cast.nepl", include_str!("../../stdlib/std/cast.nepl")),
        ("std/diag.nepl", include_str!("../../stdlib/std/diag.nepl")),
        ("std/error.nepl", include_str!("../../stdlib/std/error.nepl")),
        ("std/hashmap.nepl", include_str!("../../stdlib/std/hashmap.nepl")),
        ("std/hashset.nepl", include_str!("../../stdlib/std/hashset.nepl")),
        ("std/json.nepl", include_str!("../../stdlib/std/json.nepl")),
        ("std/list.nepl", include_str!("../../stdlib/std/list.nepl")),
        ("std/math.nepl", include_str!("../../stdlib/std/math.nepl")),
        ("std/mem.nepl", include_str!("../../stdlib/std/mem.nepl")),
        ("std/option.nepl", include_str!("../../stdlib/std/option.nepl")),
        ("std/result.nepl", include_str!("../../stdlib/std/result.nepl")),
        ("std/stack.nepl", include_str!("../../stdlib/std/stack.nepl")),
        ("std/stdio.nepl", include_str!("../../stdlib/std/stdio.nepl")),
        ("std/string.nepl", include_str!("../../stdlib/std/string.nepl")),
        ("std/test.nepl", include_str!("../../stdlib/std/test.nepl")),
        ("std/vec.nepl", include_str!("../../stdlib/std/vec.nepl")),
        ("kp/kpread.nepl", include_str!("../../stdlib/kp/kpread.nepl")),
    ]
}

fn test_sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("math", include_str!("../../stdlib/tests/math.nepl")),
        ("list", include_str!("../../stdlib/tests/list.nepl")),
        ("error", include_str!("../../stdlib/tests/error.nepl")),
        ("diag", include_str!("../../stdlib/tests/diag.nepl")),
        ("cast", include_str!("../../stdlib/tests/cast.nepl")),
        ("stack", include_str!("../../stdlib/tests/stack.nepl")),
        ("result", include_str!("../../stdlib/tests/result.nepl")),
        ("option", include_str!("../../stdlib/tests/option.nepl")),
        ("string", include_str!("../../stdlib/tests/string.nepl")),
        ("vec", include_str!("../../stdlib/tests/vec.nepl")),
    ]
}

use std::collections::BTreeMap;
use std::path::PathBuf;

use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::error::CoreError;
use nepl_core::loader::{Loader, SourceMap};
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use wasmprinter::print_bytes;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_source(source: &str) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry("/virtual/entry.nepl", source)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_to_wat(source: &str) -> Result<String, JsValue> {
    let wasm = compile_wasm_with_entry("/virtual/entry.nepl", source)
        .map_err(|msg| JsValue::from_str(&msg))?;
    print_bytes(&wasm).map_err(|e| JsValue::from_str(&e.to_string()))
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
    compile_wasm_with_entry(&format!("/virtual/tests/{name}.nepl"), src)
        .map_err(|msg| JsValue::from_str(&msg))
}

fn compile_wasm_with_entry(entry_path: &str, source: &str) -> Result<Vec<u8>, String> {
    let stdlib_root = PathBuf::from("/stdlib");
    let sources = stdlib_sources(&stdlib_root);
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Standard library bundle contains {} files", sources.len()).into());
    
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

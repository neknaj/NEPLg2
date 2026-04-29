use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Module, Stmt};
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::ResolveDiagnosticCode;
use crate::span::Span;

use super::diagnostics::resolve_error;
use super::env::{BindingKind, Env};

pub(super) fn resolve_entry_function(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    env: &Env,
    entry: Option<(String, Span)>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    if let Some((name, entry_span)) = entry {
        let bindings = env.lookup_all_callables(&name);
        let mut func_symbols = Vec::new();
        for b in bindings {
            if let BindingKind::Func { symbol, .. } = &b.kind {
                func_symbols.push(symbol.clone());
            }
        }
        if func_symbols.len() == 1 {
            Some(func_symbols.remove(0))
        } else if top_level_llvmir_defines_entry(module, target, profile, name.as_str()) {
            None
        } else {
            diagnostics.push(resolve_error(
                ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous,
                "entry function is missing or ambiguous",
                entry_span,
            ));
            None
        }
    } else {
        None
    }
}

fn top_level_llvmir_defines_entry(
    module: &Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry: &str,
) -> bool {
    if !matches!(target, CompileTarget::Llvm) {
        return false;
    }
    for idx in crate::target_precheck::active_stmt_indices(&module.root, target, profile) {
        if let Stmt::LlvmIr(block) = &module.root.items[idx] {
            for line in &block.lines {
                if crate::llvm_ir::parse_defined_function_name(line) == Some(entry) {
                    return true;
                }
            }
        }
    }
    false
}

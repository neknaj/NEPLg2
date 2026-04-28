extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::hir::HirBody;
use crate::runtime_helpers::helper_base_name;

pub const IMPURE_IO_EFFECT_MARKERS: &[&str] = &[
    "fd_read",
    "fd_write",
    "path_open",
    "path_create_directory",
    "path_filestat_get",
    "path_filestat_set_times",
    "path_link",
    "path_readlink",
    "path_remove_directory",
    "path_rename",
    "path_symlink",
    "path_unlink_file",
    "fd_advise",
    "fd_allocate",
    "fd_close",
    "fd_datasync",
    "fd_fdstat_get",
    "fd_fdstat_set_flags",
    "fd_fdstat_set_rights",
    "fd_filestat_get",
    "fd_filestat_set_size",
    "fd_filestat_set_times",
    "fd_pread",
    "fd_prestat_get",
    "fd_prestat_dir_name",
    "fd_pwrite",
    "fd_readdir",
    "fd_renumber",
    "fd_seek",
    "fd_sync",
    "fd_tell",
    "poll_oneoff",
    "proc_exit",
    "proc_raise",
    "sched_yield",
    "random_get",
    "sock_accept",
    "sock_recv",
    "sock_send",
    "sock_shutdown",
    "clock_time_get",
    "clock_res_get",
    "args_get",
    "args_sizes_get",
    "environ_get",
    "environ_sizes_get",
];

pub const RAW_MEMORY_INTRINSIC_EFFECT_MARKERS: &[&str] = &["load", "store"];

pub const RAW_MEMORY_HELPER_EFFECT_MARKERS: &[&str] = &[
    "__nepl_rt_alloc",
    "__nepl_rt_dealloc",
    "__nepl_rt_realloc",
    "alloc_raw",
    "dealloc_raw",
    "realloc_raw",
    "mem_size",
    "mem_grow",
    "load",
    "store",
    "load_i32",
    "store_i32",
    "load_u8",
    "store_u8",
    "mem_copy",
    "mem_move",
    "mem_fill",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalEffect {
    Pure,
    InternalAlloc { operation: String },
    UnsafeMemory { operation: String },
    ExternalIo { operation: String },
    Nondet { operation: String },
}

impl InternalEffect {
    pub fn operation(&self) -> Option<&str> {
        match self {
            InternalEffect::Pure => None,
            InternalEffect::InternalAlloc { operation }
            | InternalEffect::UnsafeMemory { operation }
            | InternalEffect::ExternalIo { operation }
            | InternalEffect::Nondet { operation } => Some(operation.as_str()),
        }
    }
}

pub fn intrinsic_effect(name: &str) -> Effect {
    internal_effect_untrusted_surface(&intrinsic_internal_effect(name))
}

pub fn intrinsic_internal_effect(name: &str) -> InternalEffect {
    raw_memory_internal_effect(name).unwrap_or_else(|| named_internal_effect(name))
}

pub fn intrinsic_is_raw_memory_effect(name: &str) -> bool {
    RAW_MEMORY_INTRINSIC_EFFECT_MARKERS
        .iter()
        .any(|marker| *marker == name)
}

pub fn raw_callee_internal_effect(name: &str) -> Option<InternalEffect> {
    raw_memory_internal_effect(name).or_else(|| {
        let base = helper_base_name(name);
        match named_internal_effect(base) {
            InternalEffect::Pure => None,
            effect => Some(effect),
        }
    })
}

pub fn raw_memory_callee_internal_effect(name: &str) -> Option<InternalEffect> {
    raw_memory_internal_effect(name)
}

pub fn raw_callee_is_raw_memory_effect(name: &str) -> bool {
    raw_memory_internal_effect(name).is_some()
}

pub fn internal_effect_surface_fold(effect: &InternalEffect) -> Option<Effect> {
    match effect {
        InternalEffect::Pure => Some(Effect::Pure),
        InternalEffect::InternalAlloc { .. } => Some(Effect::Pure),
        InternalEffect::ExternalIo { .. } | InternalEffect::Nondet { .. } => Some(Effect::Impure),
        InternalEffect::UnsafeMemory { .. } => None,
    }
}

pub fn internal_effect_untrusted_surface(effect: &InternalEffect) -> Effect {
    internal_effect_surface_fold(effect).unwrap_or(Effect::Impure)
}

pub fn raw_body_direct_callees(body: &HirBody) -> Vec<String> {
    let lines = match body {
        HirBody::Wasm(w) => &w.lines,
        HirBody::LlvmIr(l) => &l.lines,
        HirBody::Block(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in lines {
        let callee = match body {
            HirBody::Wasm(_) => wasm_direct_callee(line),
            HirBody::LlvmIr(_) => llvm_direct_callee(line),
            HirBody::Block(_) => None,
        };
        if let Some(callee) = callee {
            out.push(callee);
        }
    }
    out
}

pub fn raw_body_memory_operations(body: &HirBody) -> Vec<String> {
    let lines = match body {
        HirBody::Wasm(w) => &w.lines,
        HirBody::LlvmIr(l) => &l.lines,
        HirBody::Block(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in lines {
        let op = match body {
            HirBody::Wasm(_) => wasm_memory_operation(line),
            HirBody::LlvmIr(_) => llvm_memory_operation(line),
            HirBody::Block(_) => None,
        };
        if let Some(op) = op {
            out.push(op);
        }
    }
    out
}

fn wasm_direct_callee(line: &str) -> Option<String> {
    let code = strip_wasm_comment(line).trim();
    let mut parts = code.split_whitespace();
    if parts.next()? != "call" {
        return None;
    }
    parts.next().map(normalize_raw_symbol)
}

fn strip_wasm_comment(line: &str) -> &str {
    let semi = line.find(";;");
    let slash = line.find("//");
    match (semi, slash) {
        (Some(a), Some(b)) => &line[..core::cmp::min(a, b)],
        (Some(a), None) | (None, Some(a)) => &line[..a],
        (None, None) => line,
    }
}

fn wasm_memory_operation(line: &str) -> Option<String> {
    let code = strip_wasm_comment(line).trim();
    let op = code.split_whitespace().next()?;
    if wasm_op_is_memory_effect(op) {
        Some(String::from(op))
    } else {
        None
    }
}

fn wasm_op_is_memory_effect(op: &str) -> bool {
    op.starts_with("memory.") || op == "data.drop" || op.contains(".load") || op.contains(".store")
}

fn llvm_direct_callee(line: &str) -> Option<String> {
    let code = line.split(';').next().unwrap_or(line).trim();
    let call_idx = code.find("call ")?;
    let rest = &code[(call_idx + "call ".len())..];
    let at_idx = rest.find('@')?;
    let after_at = &rest[(at_idx + 1)..];
    parse_llvm_symbol(after_at).map(normalize_raw_symbol)
}

fn parse_llvm_symbol(text: &str) -> Option<&str> {
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(&rest[..end]);
    }
    let end = text
        .find(|c: char| c == '(' || c.is_whitespace())
        .unwrap_or(text.len());
    if end == 0 {
        None
    } else {
        Some(&text[..end])
    }
}

fn llvm_memory_operation(line: &str) -> Option<String> {
    let code = line.split(';').next().unwrap_or(line).trim();
    if code.is_empty() {
        return None;
    }
    if let Some(callee) = llvm_direct_callee(line) {
        if llvm_callee_is_memory_effect(&callee) {
            return Some(callee);
        }
    }
    let op = llvm_instruction_opcode(code)?;
    if llvm_op_is_memory_effect(op) {
        Some(String::from(op))
    } else {
        None
    }
}

fn llvm_instruction_opcode(code: &str) -> Option<&str> {
    let mut text = code.trim_start();
    if let Some(eq_idx) = text.find('=') {
        text = text[(eq_idx + 1)..].trim_start();
    }
    text.split_whitespace().next()
}

fn llvm_op_is_memory_effect(op: &str) -> bool {
    matches!(
        op,
        "alloca" | "load" | "store" | "atomicrmw" | "cmpxchg" | "fence"
    )
}

fn llvm_callee_is_memory_effect(callee: &str) -> bool {
    callee.starts_with("llvm.memcpy")
        || callee.starts_with("llvm.memmove")
        || callee.starts_with("llvm.memset")
}

fn raw_memory_internal_effect(name: &str) -> Option<InternalEffect> {
    let base = helper_base_name(name);
    if !RAW_MEMORY_HELPER_EFFECT_MARKERS
        .iter()
        .any(|marker| *marker == base)
        && !RAW_MEMORY_INTRINSIC_EFFECT_MARKERS
            .iter()
            .any(|marker| *marker == base)
    {
        return None;
    }
    let operation = String::from(base);
    match base {
        "__nepl_rt_alloc" | "__nepl_rt_dealloc" | "__nepl_rt_realloc" | "alloc_raw"
        | "dealloc_raw" | "realloc_raw" | "alloc" | "dealloc" | "realloc" | "mem_size"
        | "mem_grow" => Some(InternalEffect::InternalAlloc { operation }),
        _ => Some(InternalEffect::UnsafeMemory { operation }),
    }
}

fn named_internal_effect(name: &str) -> InternalEffect {
    if !IMPURE_IO_EFFECT_MARKERS
        .iter()
        .any(|marker| *marker == name)
    {
        return InternalEffect::Pure;
    }
    let operation = String::from(name);
    match name {
        "random_get" | "clock_time_get" | "clock_res_get" => InternalEffect::Nondet { operation },
        _ => InternalEffect::ExternalIo { operation },
    }
}

fn normalize_raw_symbol(symbol: &str) -> String {
    let trimmed = symbol.trim();
    let without_prefix = trimmed
        .strip_prefix('$')
        .or_else(|| trimmed.strip_prefix('@'))
        .unwrap_or(trimmed);
    if let Some(inner) = without_prefix
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
    {
        String::from(inner)
    } else {
        String::from(without_prefix)
    }
}

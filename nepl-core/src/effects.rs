extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::hir::HirBody;
use core::fmt;

use crate::runtime_helpers::{
    helper_base_name, ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI,
};

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
    "memset_u8",
    "fill_u8",
    "fill_i32",
    "mem_fill",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RawMemoryOp {
    Alloc,
    Dealloc,
    Realloc,
    Load,
    Store,
    BulkCopy,
    BulkMove,
    MemorySize,
    MemoryGrow,
    Fill,
}

impl RawMemoryOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            RawMemoryOp::Alloc => "alloc",
            RawMemoryOp::Dealloc => "dealloc",
            RawMemoryOp::Realloc => "realloc",
            RawMemoryOp::Load => "load",
            RawMemoryOp::Store => "store",
            RawMemoryOp::BulkCopy => "bulk_copy",
            RawMemoryOp::BulkMove => "bulk_move",
            RawMemoryOp::MemorySize => "memory_size",
            RawMemoryOp::MemoryGrow => "memory_grow",
            RawMemoryOp::Fill => "fill",
        }
    }
}

impl fmt::Display for RawMemoryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExternalIoOp {
    FdRead,
    FdWrite,
    PathOpen,
    PathCreateDirectory,
    PathFilestatGet,
    PathFilestatSetTimes,
    PathLink,
    PathReadlink,
    PathRemoveDirectory,
    PathRename,
    PathSymlink,
    PathUnlinkFile,
    FdAdvise,
    FdAllocate,
    FdClose,
    FdDatasync,
    FdFdstatGet,
    FdFdstatSetFlags,
    FdFdstatSetRights,
    FdFilestatGet,
    FdFilestatSetSize,
    FdFilestatSetTimes,
    FdPread,
    FdPrestatGet,
    FdPrestatDirName,
    FdPwrite,
    FdReaddir,
    FdRenumber,
    FdSeek,
    FdSync,
    FdTell,
    PollOneoff,
    ProcExit,
    ProcRaise,
    SchedYield,
    SockAccept,
    SockRecv,
    SockSend,
    SockShutdown,
    ArgsGet,
    ArgsSizesGet,
    EnvironGet,
    EnvironSizesGet,
}

impl ExternalIoOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            ExternalIoOp::FdRead => "fd_read",
            ExternalIoOp::FdWrite => "fd_write",
            ExternalIoOp::PathOpen => "path_open",
            ExternalIoOp::PathCreateDirectory => "path_create_directory",
            ExternalIoOp::PathFilestatGet => "path_filestat_get",
            ExternalIoOp::PathFilestatSetTimes => "path_filestat_set_times",
            ExternalIoOp::PathLink => "path_link",
            ExternalIoOp::PathReadlink => "path_readlink",
            ExternalIoOp::PathRemoveDirectory => "path_remove_directory",
            ExternalIoOp::PathRename => "path_rename",
            ExternalIoOp::PathSymlink => "path_symlink",
            ExternalIoOp::PathUnlinkFile => "path_unlink_file",
            ExternalIoOp::FdAdvise => "fd_advise",
            ExternalIoOp::FdAllocate => "fd_allocate",
            ExternalIoOp::FdClose => "fd_close",
            ExternalIoOp::FdDatasync => "fd_datasync",
            ExternalIoOp::FdFdstatGet => "fd_fdstat_get",
            ExternalIoOp::FdFdstatSetFlags => "fd_fdstat_set_flags",
            ExternalIoOp::FdFdstatSetRights => "fd_fdstat_set_rights",
            ExternalIoOp::FdFilestatGet => "fd_filestat_get",
            ExternalIoOp::FdFilestatSetSize => "fd_filestat_set_size",
            ExternalIoOp::FdFilestatSetTimes => "fd_filestat_set_times",
            ExternalIoOp::FdPread => "fd_pread",
            ExternalIoOp::FdPrestatGet => "fd_prestat_get",
            ExternalIoOp::FdPrestatDirName => "fd_prestat_dir_name",
            ExternalIoOp::FdPwrite => "fd_pwrite",
            ExternalIoOp::FdReaddir => "fd_readdir",
            ExternalIoOp::FdRenumber => "fd_renumber",
            ExternalIoOp::FdSeek => "fd_seek",
            ExternalIoOp::FdSync => "fd_sync",
            ExternalIoOp::FdTell => "fd_tell",
            ExternalIoOp::PollOneoff => "poll_oneoff",
            ExternalIoOp::ProcExit => "proc_exit",
            ExternalIoOp::ProcRaise => "proc_raise",
            ExternalIoOp::SchedYield => "sched_yield",
            ExternalIoOp::SockAccept => "sock_accept",
            ExternalIoOp::SockRecv => "sock_recv",
            ExternalIoOp::SockSend => "sock_send",
            ExternalIoOp::SockShutdown => "sock_shutdown",
            ExternalIoOp::ArgsGet => "args_get",
            ExternalIoOp::ArgsSizesGet => "args_sizes_get",
            ExternalIoOp::EnvironGet => "environ_get",
            ExternalIoOp::EnvironSizesGet => "environ_sizes_get",
        }
    }
}

impl fmt::Display for ExternalIoOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NondetOp {
    RandomGet,
    ClockTimeGet,
    ClockResGet,
}

impl NondetOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            NondetOp::RandomGet => "random_get",
            NondetOp::ClockTimeGet => "clock_time_get",
            NondetOp::ClockResGet => "clock_res_get",
        }
    }
}

impl fmt::Display for NondetOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalEffect {
    Pure,
    InternalAlloc { operation: RawMemoryOp },
    UnsafeMemory { operation: RawMemoryOp },
    ExternalIo { operation: ExternalIoOp },
    Nondet { operation: NondetOp },
}

impl InternalEffect {
    pub fn operation(&self) -> Option<&str> {
        match self {
            InternalEffect::Pure => None,
            InternalEffect::InternalAlloc { operation }
            | InternalEffect::UnsafeMemory { operation } => Some(operation.as_str()),
            InternalEffect::ExternalIo { operation } => Some(operation.as_str()),
            InternalEffect::Nondet { operation } => Some(operation.as_str()),
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

pub fn raw_memory_op_from_name(name: &str) -> Option<RawMemoryOp> {
    let base = helper_base_name(name);
    if !raw_memory_base_is_known(base) {
        return None;
    }
    let operation = match base {
        ALLOC_RUNTIME_ABI | "alloc_raw" => RawMemoryOp::Alloc,
        DEALLOC_RUNTIME_ABI | "dealloc_raw" => RawMemoryOp::Dealloc,
        REALLOC_RUNTIME_ABI | "realloc_raw" => RawMemoryOp::Realloc,
        "load" | "load_i32" | "load_u8" => RawMemoryOp::Load,
        "store" | "store_i32" | "store_u8" => RawMemoryOp::Store,
        "mem_copy" => RawMemoryOp::BulkCopy,
        "mem_move" => RawMemoryOp::BulkMove,
        "memset_u8" | "fill_u8" | "fill_i32" | "mem_fill" => RawMemoryOp::Fill,
        "mem_size" => RawMemoryOp::MemorySize,
        "mem_grow" => RawMemoryOp::MemoryGrow,
        _ => return None,
    };
    Some(operation)
}

pub fn external_io_op_from_name(name: &str) -> Option<ExternalIoOp> {
    let base = helper_base_name(name);
    let operation = match base {
        "fd_read" => ExternalIoOp::FdRead,
        "fd_write" => ExternalIoOp::FdWrite,
        "path_open" => ExternalIoOp::PathOpen,
        "path_create_directory" => ExternalIoOp::PathCreateDirectory,
        "path_filestat_get" => ExternalIoOp::PathFilestatGet,
        "path_filestat_set_times" => ExternalIoOp::PathFilestatSetTimes,
        "path_link" => ExternalIoOp::PathLink,
        "path_readlink" => ExternalIoOp::PathReadlink,
        "path_remove_directory" => ExternalIoOp::PathRemoveDirectory,
        "path_rename" => ExternalIoOp::PathRename,
        "path_symlink" => ExternalIoOp::PathSymlink,
        "path_unlink_file" => ExternalIoOp::PathUnlinkFile,
        "fd_advise" => ExternalIoOp::FdAdvise,
        "fd_allocate" => ExternalIoOp::FdAllocate,
        "fd_close" => ExternalIoOp::FdClose,
        "fd_datasync" => ExternalIoOp::FdDatasync,
        "fd_fdstat_get" => ExternalIoOp::FdFdstatGet,
        "fd_fdstat_set_flags" => ExternalIoOp::FdFdstatSetFlags,
        "fd_fdstat_set_rights" => ExternalIoOp::FdFdstatSetRights,
        "fd_filestat_get" => ExternalIoOp::FdFilestatGet,
        "fd_filestat_set_size" => ExternalIoOp::FdFilestatSetSize,
        "fd_filestat_set_times" => ExternalIoOp::FdFilestatSetTimes,
        "fd_pread" => ExternalIoOp::FdPread,
        "fd_prestat_get" => ExternalIoOp::FdPrestatGet,
        "fd_prestat_dir_name" => ExternalIoOp::FdPrestatDirName,
        "fd_pwrite" => ExternalIoOp::FdPwrite,
        "fd_readdir" => ExternalIoOp::FdReaddir,
        "fd_renumber" => ExternalIoOp::FdRenumber,
        "fd_seek" => ExternalIoOp::FdSeek,
        "fd_sync" => ExternalIoOp::FdSync,
        "fd_tell" => ExternalIoOp::FdTell,
        "poll_oneoff" => ExternalIoOp::PollOneoff,
        "proc_exit" => ExternalIoOp::ProcExit,
        "proc_raise" => ExternalIoOp::ProcRaise,
        "sched_yield" => ExternalIoOp::SchedYield,
        "sock_accept" => ExternalIoOp::SockAccept,
        "sock_recv" => ExternalIoOp::SockRecv,
        "sock_send" => ExternalIoOp::SockSend,
        "sock_shutdown" => ExternalIoOp::SockShutdown,
        "args_get" => ExternalIoOp::ArgsGet,
        "args_sizes_get" => ExternalIoOp::ArgsSizesGet,
        "environ_get" => ExternalIoOp::EnvironGet,
        "environ_sizes_get" => ExternalIoOp::EnvironSizesGet,
        _ => return None,
    };
    Some(operation)
}

pub fn nondet_op_from_name(name: &str) -> Option<NondetOp> {
    let base = helper_base_name(name);
    let operation = match base {
        "random_get" => NondetOp::RandomGet,
        "clock_time_get" => NondetOp::ClockTimeGet,
        "clock_res_get" => NondetOp::ClockResGet,
        _ => return None,
    };
    Some(operation)
}

fn raw_memory_base_is_known(base: &str) -> bool {
    RAW_MEMORY_HELPER_EFFECT_MARKERS
        .iter()
        .any(|marker| *marker == base)
        || RAW_MEMORY_INTRINSIC_EFFECT_MARKERS
            .iter()
            .any(|marker| *marker == base)
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
    let operation = raw_memory_op_from_name(name)?;
    match operation {
        RawMemoryOp::Alloc
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => Some(InternalEffect::InternalAlloc { operation }),
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::Fill => Some(InternalEffect::UnsafeMemory { operation }),
    }
}

fn named_internal_effect(name: &str) -> InternalEffect {
    if let Some(operation) = nondet_op_from_name(name) {
        return InternalEffect::Nondet { operation };
    }
    if let Some(operation) = external_io_op_from_name(name) {
        return InternalEffect::ExternalIo { operation };
    }
    InternalEffect::Pure
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

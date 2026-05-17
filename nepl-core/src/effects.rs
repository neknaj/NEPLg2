extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::hir::HirBody;
use core::fmt;

use crate::runtime_helpers::{
    helper_base_name, ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI,
};

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
    FillBytes,
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
            RawMemoryOp::FillBytes => "fill_bytes",
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
pub enum RawMemoryHelper {
    RuntimeAlloc,
    RuntimeDealloc,
    RuntimeRealloc,
    AllocRaw,
    DeallocRaw,
    ReallocRaw,
    MemSize,
    MemGrow,
    Load,
    Store,
    LoadI32,
    StoreI32,
    LoadU8,
    StoreU8,
    MemCopy,
    MemMove,
    MemsetU8,
    FillU8,
    FillI32,
    MemFill,
}

impl RawMemoryHelper {
    pub const ALL: &'static [Self] = &[
        Self::RuntimeAlloc,
        Self::RuntimeDealloc,
        Self::RuntimeRealloc,
        Self::AllocRaw,
        Self::DeallocRaw,
        Self::ReallocRaw,
        Self::MemSize,
        Self::MemGrow,
        Self::Load,
        Self::Store,
        Self::LoadI32,
        Self::StoreI32,
        Self::LoadU8,
        Self::StoreU8,
        Self::MemCopy,
        Self::MemMove,
        Self::MemsetU8,
        Self::FillU8,
        Self::FillI32,
        Self::MemFill,
    ];

    pub const fn base_name(self) -> &'static str {
        match self {
            RawMemoryHelper::RuntimeAlloc => ALLOC_RUNTIME_ABI,
            RawMemoryHelper::RuntimeDealloc => DEALLOC_RUNTIME_ABI,
            RawMemoryHelper::RuntimeRealloc => REALLOC_RUNTIME_ABI,
            RawMemoryHelper::AllocRaw => "alloc_raw",
            RawMemoryHelper::DeallocRaw => "dealloc_raw",
            RawMemoryHelper::ReallocRaw => "realloc_raw",
            RawMemoryHelper::MemSize => "mem_size",
            RawMemoryHelper::MemGrow => "mem_grow",
            RawMemoryHelper::Load => "load",
            RawMemoryHelper::Store => "store",
            RawMemoryHelper::LoadI32 => "load_i32",
            RawMemoryHelper::StoreI32 => "store_i32",
            RawMemoryHelper::LoadU8 => "load_u8",
            RawMemoryHelper::StoreU8 => "store_u8",
            RawMemoryHelper::MemCopy => "mem_copy",
            RawMemoryHelper::MemMove => "mem_move",
            RawMemoryHelper::MemsetU8 => "memset_u8",
            RawMemoryHelper::FillU8 => "fill_u8",
            RawMemoryHelper::FillI32 => "fill_i32",
            RawMemoryHelper::MemFill => "mem_fill",
        }
    }

    pub const fn operation(self) -> RawMemoryOp {
        match self {
            RawMemoryHelper::RuntimeAlloc | RawMemoryHelper::AllocRaw => RawMemoryOp::Alloc,
            RawMemoryHelper::RuntimeDealloc | RawMemoryHelper::DeallocRaw => RawMemoryOp::Dealloc,
            RawMemoryHelper::RuntimeRealloc | RawMemoryHelper::ReallocRaw => RawMemoryOp::Realloc,
            RawMemoryHelper::Load | RawMemoryHelper::LoadI32 | RawMemoryHelper::LoadU8 => {
                RawMemoryOp::Load
            }
            RawMemoryHelper::Store | RawMemoryHelper::StoreI32 | RawMemoryHelper::StoreU8 => {
                RawMemoryOp::Store
            }
            RawMemoryHelper::MemCopy => RawMemoryOp::BulkCopy,
            RawMemoryHelper::MemMove => RawMemoryOp::BulkMove,
            RawMemoryHelper::MemsetU8 | RawMemoryHelper::FillU8 | RawMemoryHelper::MemFill => {
                RawMemoryOp::FillBytes
            }
            RawMemoryHelper::FillI32 => RawMemoryOp::Fill,
            RawMemoryHelper::MemSize => RawMemoryOp::MemorySize,
            RawMemoryHelper::MemGrow => RawMemoryOp::MemoryGrow,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let base = helper_base_name(name);
        let helper = match base {
            ALLOC_RUNTIME_ABI => RawMemoryHelper::RuntimeAlloc,
            DEALLOC_RUNTIME_ABI => RawMemoryHelper::RuntimeDealloc,
            REALLOC_RUNTIME_ABI => RawMemoryHelper::RuntimeRealloc,
            "alloc_raw" => RawMemoryHelper::AllocRaw,
            "dealloc_raw" => RawMemoryHelper::DeallocRaw,
            "realloc_raw" => RawMemoryHelper::ReallocRaw,
            "mem_size" => RawMemoryHelper::MemSize,
            "mem_grow" => RawMemoryHelper::MemGrow,
            "load" => RawMemoryHelper::Load,
            "store" => RawMemoryHelper::Store,
            "load_i32" => RawMemoryHelper::LoadI32,
            "store_i32" => RawMemoryHelper::StoreI32,
            "load_u8" => RawMemoryHelper::LoadU8,
            "store_u8" => RawMemoryHelper::StoreU8,
            "mem_copy" => RawMemoryHelper::MemCopy,
            "mem_move" => RawMemoryHelper::MemMove,
            "memset_u8" => RawMemoryHelper::MemsetU8,
            "fill_u8" => RawMemoryHelper::FillU8,
            "fill_i32" => RawMemoryHelper::FillI32,
            "mem_fill" => RawMemoryHelper::MemFill,
            _ => return None,
        };
        Some(helper)
    }
}

impl fmt::Display for RawMemoryHelper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.base_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RawBodyMemoryOp {
    Wasm(WasmRawBodyMemoryOp),
    Llvm(LlvmRawBodyMemoryOp),
}

impl RawBodyMemoryOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            RawBodyMemoryOp::Wasm(operation) => operation.as_str(),
            RawBodyMemoryOp::Llvm(operation) => operation.as_str(),
        }
    }
}

impl fmt::Display for RawBodyMemoryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawBodyDirectCallee {
    RawMemory {
        callee: String,
        operation: RawMemoryOp,
    },
    BackendIntrinsic {
        callee: String,
        backend: RawBodyBackend,
    },
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawBodyBackend {
    Wasm,
    Llvm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WasmRawBodyMemoryOp {
    Load,
    Store,
    MemorySize,
    MemoryGrow,
    MemoryCopy,
    MemoryFill,
    MemoryInit,
    DataDrop,
    Memory,
}

impl WasmRawBodyMemoryOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            WasmRawBodyMemoryOp::Load => "wasm.load",
            WasmRawBodyMemoryOp::Store => "wasm.store",
            WasmRawBodyMemoryOp::MemorySize => "wasm.memory.size",
            WasmRawBodyMemoryOp::MemoryGrow => "wasm.memory.grow",
            WasmRawBodyMemoryOp::MemoryCopy => "wasm.memory.copy",
            WasmRawBodyMemoryOp::MemoryFill => "wasm.memory.fill",
            WasmRawBodyMemoryOp::MemoryInit => "wasm.memory.init",
            WasmRawBodyMemoryOp::DataDrop => "wasm.data.drop",
            WasmRawBodyMemoryOp::Memory => "wasm.memory",
        }
    }
}

impl fmt::Display for WasmRawBodyMemoryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LlvmRawBodyMemoryOp {
    Alloca,
    Load,
    Store,
    AtomicRmw,
    Cmpxchg,
    Fence,
    Memcpy,
    Memmove,
    Memset,
}

impl LlvmRawBodyMemoryOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            LlvmRawBodyMemoryOp::Alloca => "llvm.alloca",
            LlvmRawBodyMemoryOp::Load => "llvm.load",
            LlvmRawBodyMemoryOp::Store => "llvm.store",
            LlvmRawBodyMemoryOp::AtomicRmw => "llvm.atomicrmw",
            LlvmRawBodyMemoryOp::Cmpxchg => "llvm.cmpxchg",
            LlvmRawBodyMemoryOp::Fence => "llvm.fence",
            LlvmRawBodyMemoryOp::Memcpy => "llvm.memcpy",
            LlvmRawBodyMemoryOp::Memmove => "llvm.memmove",
            LlvmRawBodyMemoryOp::Memset => "llvm.memset",
        }
    }
}

impl fmt::Display for LlvmRawBodyMemoryOp {
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
    pub const ALL: &'static [Self] = &[
        Self::FdRead,
        Self::FdWrite,
        Self::PathOpen,
        Self::PathCreateDirectory,
        Self::PathFilestatGet,
        Self::PathFilestatSetTimes,
        Self::PathLink,
        Self::PathReadlink,
        Self::PathRemoveDirectory,
        Self::PathRename,
        Self::PathSymlink,
        Self::PathUnlinkFile,
        Self::FdAdvise,
        Self::FdAllocate,
        Self::FdClose,
        Self::FdDatasync,
        Self::FdFdstatGet,
        Self::FdFdstatSetFlags,
        Self::FdFdstatSetRights,
        Self::FdFilestatGet,
        Self::FdFilestatSetSize,
        Self::FdFilestatSetTimes,
        Self::FdPread,
        Self::FdPrestatGet,
        Self::FdPrestatDirName,
        Self::FdPwrite,
        Self::FdReaddir,
        Self::FdRenumber,
        Self::FdSeek,
        Self::FdSync,
        Self::FdTell,
        Self::PollOneoff,
        Self::ProcExit,
        Self::ProcRaise,
        Self::SchedYield,
        Self::SockAccept,
        Self::SockRecv,
        Self::SockSend,
        Self::SockShutdown,
        Self::ArgsGet,
        Self::ArgsSizesGet,
        Self::EnvironGet,
        Self::EnvironSizesGet,
    ];

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
    pub const ALL: &'static [Self] = &[Self::RandomGet, Self::ClockTimeGet, Self::ClockResGet];

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
    raw_memory_intrinsic_op_from_name(name).is_some()
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
    RawMemoryHelper::from_name(name).map(RawMemoryHelper::operation)
}

pub fn raw_memory_intrinsic_op_from_name(name: &str) -> Option<RawMemoryOp> {
    let operation = match name {
        "load" => RawMemoryOp::Load,
        "store" => RawMemoryOp::Store,
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

pub fn raw_body_direct_callee_effects(body: &HirBody) -> Vec<RawBodyDirectCallee> {
    let (backend, lines) = match body {
        HirBody::Wasm(w) => (RawBodyBackend::Wasm, &w.lines),
        HirBody::LlvmIr(l) => (RawBodyBackend::Llvm, &l.lines),
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
            out.push(classify_raw_body_direct_callee(backend, callee));
        }
    }
    out
}

fn classify_raw_body_direct_callee(backend: RawBodyBackend, callee: String) -> RawBodyDirectCallee {
    if let Some(operation) = raw_memory_op_from_name(&callee) {
        return RawBodyDirectCallee::RawMemory { callee, operation };
    }
    if raw_body_backend_intrinsic_callee(backend, &callee) {
        return RawBodyDirectCallee::BackendIntrinsic { callee, backend };
    }
    RawBodyDirectCallee::Other(callee)
}

fn raw_body_backend_intrinsic_callee(backend: RawBodyBackend, callee: &str) -> bool {
    match backend {
        RawBodyBackend::Wasm => false,
        RawBodyBackend::Llvm => callee.starts_with("llvm."),
    }
}

pub fn raw_body_memory_operations(body: &HirBody) -> Vec<RawBodyMemoryOp> {
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

fn wasm_memory_operation(line: &str) -> Option<RawBodyMemoryOp> {
    let code = strip_wasm_comment(line).trim();
    let op = code.split_whitespace().next()?;
    wasm_memory_op_from_opcode(op).map(RawBodyMemoryOp::Wasm)
}

fn wasm_memory_op_from_opcode(op: &str) -> Option<WasmRawBodyMemoryOp> {
    if op.contains(".load") {
        return Some(WasmRawBodyMemoryOp::Load);
    }
    if op.contains(".store") {
        return Some(WasmRawBodyMemoryOp::Store);
    }
    let operation = match op {
        "memory.size" => WasmRawBodyMemoryOp::MemorySize,
        "memory.grow" => WasmRawBodyMemoryOp::MemoryGrow,
        "memory.copy" => WasmRawBodyMemoryOp::MemoryCopy,
        "memory.fill" => WasmRawBodyMemoryOp::MemoryFill,
        "memory.init" => WasmRawBodyMemoryOp::MemoryInit,
        "data.drop" => WasmRawBodyMemoryOp::DataDrop,
        _ if op.starts_with("memory.") => WasmRawBodyMemoryOp::Memory,
        _ => return None,
    };
    Some(operation)
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

fn llvm_memory_operation(line: &str) -> Option<RawBodyMemoryOp> {
    let code = line.split(';').next().unwrap_or(line).trim();
    if code.is_empty() {
        return None;
    }
    if let Some(callee) = llvm_direct_callee(line) {
        if let Some(operation) = llvm_memory_op_from_callee(&callee) {
            return Some(RawBodyMemoryOp::Llvm(operation));
        }
    }
    let op = llvm_instruction_opcode(code)?;
    llvm_memory_op_from_opcode(op).map(RawBodyMemoryOp::Llvm)
}

fn llvm_instruction_opcode(code: &str) -> Option<&str> {
    let mut text = code.trim_start();
    if let Some(eq_idx) = text.find('=') {
        text = text[(eq_idx + 1)..].trim_start();
    }
    text.split_whitespace().next()
}

fn llvm_memory_op_from_opcode(op: &str) -> Option<LlvmRawBodyMemoryOp> {
    let operation = match op {
        "alloca" => LlvmRawBodyMemoryOp::Alloca,
        "load" => LlvmRawBodyMemoryOp::Load,
        "store" => LlvmRawBodyMemoryOp::Store,
        "atomicrmw" => LlvmRawBodyMemoryOp::AtomicRmw,
        "cmpxchg" => LlvmRawBodyMemoryOp::Cmpxchg,
        "fence" => LlvmRawBodyMemoryOp::Fence,
        _ => return None,
    };
    Some(operation)
}

fn llvm_memory_op_from_callee(callee: &str) -> Option<LlvmRawBodyMemoryOp> {
    if callee.starts_with("llvm.memcpy") {
        return Some(LlvmRawBodyMemoryOp::Memcpy);
    }
    if callee.starts_with("llvm.memmove") {
        return Some(LlvmRawBodyMemoryOp::Memmove);
    }
    if callee.starts_with("llvm.memset") {
        return Some(LlvmRawBodyMemoryOp::Memset);
    }
    None
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
        | RawMemoryOp::FillBytes
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

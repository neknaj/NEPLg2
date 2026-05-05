extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_summary::{
    compute_raw_identity_return_summaries, compute_raw_pointer_return_summaries,
};
use super::model::{ExternalIoOp, NondetOp, Place, RawMemoryOp, ResourceModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectBoundaryReport {
    pub functions: Vec<ResourceEffectFunctionCheck>,
    pub diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectFunctionCheck {
    pub name: String,
    pub counts: ResourceEffectCounts,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceEffectCounts {
    pub internal_memory_ops: RawMemoryEffectCounts,
    pub unsafe_memory_ops: RawMemoryEffectCounts,
    pub external_io_ops: ExternalIoEffectCounts,
    pub nondet_ops: NondetEffectCounts,
    pub unknown_ops: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RawMemoryEffectCounts {
    pub alloc: usize,
    pub dealloc: usize,
    pub realloc: usize,
    pub load: usize,
    pub store: usize,
    pub bulk_copy: usize,
    pub bulk_move: usize,
    pub memory_size: usize,
    pub memory_grow: usize,
    pub fill: usize,
}

impl RawMemoryEffectCounts {
    pub fn record(&mut self, operation: RawMemoryOp) {
        match operation {
            RawMemoryOp::Alloc => self.alloc += 1,
            RawMemoryOp::Dealloc => self.dealloc += 1,
            RawMemoryOp::Realloc => self.realloc += 1,
            RawMemoryOp::Load => self.load += 1,
            RawMemoryOp::Store => self.store += 1,
            RawMemoryOp::BulkCopy => self.bulk_copy += 1,
            RawMemoryOp::BulkMove => self.bulk_move += 1,
            RawMemoryOp::MemorySize => self.memory_size += 1,
            RawMemoryOp::MemoryGrow => self.memory_grow += 1,
            RawMemoryOp::Fill => self.fill += 1,
        }
    }

    pub fn total(self) -> usize {
        self.alloc
            + self.dealloc
            + self.realloc
            + self.load
            + self.store
            + self.bulk_copy
            + self.bulk_move
            + self.memory_size
            + self.memory_grow
            + self.fill
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExternalIoEffectCounts {
    pub fd_read: usize,
    pub fd_write: usize,
    pub path_open: usize,
    pub path_create_directory: usize,
    pub path_filestat_get: usize,
    pub path_filestat_set_times: usize,
    pub path_link: usize,
    pub path_readlink: usize,
    pub path_remove_directory: usize,
    pub path_rename: usize,
    pub path_symlink: usize,
    pub path_unlink_file: usize,
    pub fd_advise: usize,
    pub fd_allocate: usize,
    pub fd_close: usize,
    pub fd_datasync: usize,
    pub fd_fdstat_get: usize,
    pub fd_fdstat_set_flags: usize,
    pub fd_fdstat_set_rights: usize,
    pub fd_filestat_get: usize,
    pub fd_filestat_set_size: usize,
    pub fd_filestat_set_times: usize,
    pub fd_pread: usize,
    pub fd_prestat_get: usize,
    pub fd_prestat_dir_name: usize,
    pub fd_pwrite: usize,
    pub fd_readdir: usize,
    pub fd_renumber: usize,
    pub fd_seek: usize,
    pub fd_sync: usize,
    pub fd_tell: usize,
    pub poll_oneoff: usize,
    pub proc_exit: usize,
    pub proc_raise: usize,
    pub sched_yield: usize,
    pub sock_accept: usize,
    pub sock_recv: usize,
    pub sock_send: usize,
    pub sock_shutdown: usize,
    pub args_get: usize,
    pub args_sizes_get: usize,
    pub environ_get: usize,
    pub environ_sizes_get: usize,
}

impl ExternalIoEffectCounts {
    pub fn record(&mut self, operation: ExternalIoOp) {
        match operation {
            ExternalIoOp::FdRead => self.fd_read += 1,
            ExternalIoOp::FdWrite => self.fd_write += 1,
            ExternalIoOp::PathOpen => self.path_open += 1,
            ExternalIoOp::PathCreateDirectory => self.path_create_directory += 1,
            ExternalIoOp::PathFilestatGet => self.path_filestat_get += 1,
            ExternalIoOp::PathFilestatSetTimes => self.path_filestat_set_times += 1,
            ExternalIoOp::PathLink => self.path_link += 1,
            ExternalIoOp::PathReadlink => self.path_readlink += 1,
            ExternalIoOp::PathRemoveDirectory => self.path_remove_directory += 1,
            ExternalIoOp::PathRename => self.path_rename += 1,
            ExternalIoOp::PathSymlink => self.path_symlink += 1,
            ExternalIoOp::PathUnlinkFile => self.path_unlink_file += 1,
            ExternalIoOp::FdAdvise => self.fd_advise += 1,
            ExternalIoOp::FdAllocate => self.fd_allocate += 1,
            ExternalIoOp::FdClose => self.fd_close += 1,
            ExternalIoOp::FdDatasync => self.fd_datasync += 1,
            ExternalIoOp::FdFdstatGet => self.fd_fdstat_get += 1,
            ExternalIoOp::FdFdstatSetFlags => self.fd_fdstat_set_flags += 1,
            ExternalIoOp::FdFdstatSetRights => self.fd_fdstat_set_rights += 1,
            ExternalIoOp::FdFilestatGet => self.fd_filestat_get += 1,
            ExternalIoOp::FdFilestatSetSize => self.fd_filestat_set_size += 1,
            ExternalIoOp::FdFilestatSetTimes => self.fd_filestat_set_times += 1,
            ExternalIoOp::FdPread => self.fd_pread += 1,
            ExternalIoOp::FdPrestatGet => self.fd_prestat_get += 1,
            ExternalIoOp::FdPrestatDirName => self.fd_prestat_dir_name += 1,
            ExternalIoOp::FdPwrite => self.fd_pwrite += 1,
            ExternalIoOp::FdReaddir => self.fd_readdir += 1,
            ExternalIoOp::FdRenumber => self.fd_renumber += 1,
            ExternalIoOp::FdSeek => self.fd_seek += 1,
            ExternalIoOp::FdSync => self.fd_sync += 1,
            ExternalIoOp::FdTell => self.fd_tell += 1,
            ExternalIoOp::PollOneoff => self.poll_oneoff += 1,
            ExternalIoOp::ProcExit => self.proc_exit += 1,
            ExternalIoOp::ProcRaise => self.proc_raise += 1,
            ExternalIoOp::SchedYield => self.sched_yield += 1,
            ExternalIoOp::SockAccept => self.sock_accept += 1,
            ExternalIoOp::SockRecv => self.sock_recv += 1,
            ExternalIoOp::SockSend => self.sock_send += 1,
            ExternalIoOp::SockShutdown => self.sock_shutdown += 1,
            ExternalIoOp::ArgsGet => self.args_get += 1,
            ExternalIoOp::ArgsSizesGet => self.args_sizes_get += 1,
            ExternalIoOp::EnvironGet => self.environ_get += 1,
            ExternalIoOp::EnvironSizesGet => self.environ_sizes_get += 1,
        }
    }

    pub fn total(self) -> usize {
        self.fd_read
            + self.fd_write
            + self.path_open
            + self.path_create_directory
            + self.path_filestat_get
            + self.path_filestat_set_times
            + self.path_link
            + self.path_readlink
            + self.path_remove_directory
            + self.path_rename
            + self.path_symlink
            + self.path_unlink_file
            + self.fd_advise
            + self.fd_allocate
            + self.fd_close
            + self.fd_datasync
            + self.fd_fdstat_get
            + self.fd_fdstat_set_flags
            + self.fd_fdstat_set_rights
            + self.fd_filestat_get
            + self.fd_filestat_set_size
            + self.fd_filestat_set_times
            + self.fd_pread
            + self.fd_prestat_get
            + self.fd_prestat_dir_name
            + self.fd_pwrite
            + self.fd_readdir
            + self.fd_renumber
            + self.fd_seek
            + self.fd_sync
            + self.fd_tell
            + self.poll_oneoff
            + self.proc_exit
            + self.proc_raise
            + self.sched_yield
            + self.sock_accept
            + self.sock_recv
            + self.sock_send
            + self.sock_shutdown
            + self.args_get
            + self.args_sizes_get
            + self.environ_get
            + self.environ_sizes_get
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NondetEffectCounts {
    pub random_get: usize,
    pub clock_time_get: usize,
    pub clock_res_get: usize,
}

impl NondetEffectCounts {
    pub fn record(&mut self, operation: NondetOp) {
        match operation {
            NondetOp::RandomGet => self.random_get += 1,
            NondetOp::ClockTimeGet => self.clock_time_get += 1,
            NondetOp::ClockResGet => self.clock_res_get += 1,
        }
    }

    pub fn total(self) -> usize {
        self.random_get + self.clock_time_get + self.clock_res_get
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectBoundaryDiagnostic {
    ImpureCallInPureFunction {
        function: String,
        call: ResourceEffectCallKind,
        span: Span,
    },
    UnsafeMemoryInPureFunction {
        function: String,
        operation: RawMemoryOp,
        span: Span,
    },
    RawAddressEscapeFromInternalAlloc {
        function: String,
        place: Place,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEffectCallKind {
    Direct { name: String },
    ExternalIo { operation: ExternalIoOp },
    Nondet { operation: NondetOp },
    Indirect,
}

pub fn check_resource_effect_boundaries(module: &ResourceModule) -> ResourceEffectBoundaryReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let pointer_summaries = compute_raw_pointer_return_summaries(module);
    let summaries = compute_raw_identity_return_summaries(module, &pointer_summaries);

    for function in &module.functions {
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summaries,
            pointer_summaries: &pointer_summaries,
            track_alloc_identities: true,
            diagnostics: Vec::new(),
            counts: ResourceEffectCounts::default(),
        };
        engine.check_function(function);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceEffectFunctionCheck {
            name: function.name.clone(),
            counts: engine.counts,
        });
    }

    ResourceEffectBoundaryReport {
        functions,
        diagnostics,
    }
}

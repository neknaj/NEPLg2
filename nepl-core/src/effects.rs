extern crate alloc;

use alloc::string::String;

use crate::ast::Effect;
use crate::hir::HirBody;

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

pub fn marker_is_impure_io(text: &str) -> bool {
    IMPURE_IO_EFFECT_MARKERS.iter().any(|m| text.contains(m))
}

pub fn intrinsic_effect(name: &str) -> Effect {
    if IMPURE_IO_EFFECT_MARKERS.iter().any(|m| *m == name) {
        Effect::Impure
    } else {
        Effect::Pure
    }
}

pub fn raw_lines_effect(lines: &[String]) -> Effect {
    if lines.iter().any(|line| marker_is_impure_io(line)) {
        Effect::Impure
    } else {
        Effect::Pure
    }
}

pub fn raw_body_effect(body: &HirBody) -> Effect {
    match body {
        HirBody::Wasm(w) => raw_lines_effect(&w.lines),
        HirBody::LlvmIr(l) => raw_lines_effect(&l.lines),
        HirBody::Block(_) => Effect::Pure,
    }
}

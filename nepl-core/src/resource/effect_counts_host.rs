use super::model::{ExternalIoOp, NondetOp};

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

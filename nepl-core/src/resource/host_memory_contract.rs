use crate::types::TypeId;

use super::model::{EffectOp, ExternalIoOp, NondetOp, Place};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostMemoryDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostMemoryDirectUnit {
    Bytes,
    I32Cell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostMemoryLength {
    Arg(usize),
    ConstI32(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostMemorySpan {
    Direct {
        address_arg: usize,
        length: HostMemoryLength,
        unit: HostMemoryDirectUnit,
        direction: HostMemoryDirection,
    },
    IovDescriptor {
        iovs_arg: usize,
    },
    IovPayload {
        iovs_arg: usize,
        iov_count_arg: usize,
        transferred_count_arg: Option<usize>,
        direction: HostMemoryDirection,
    },
}

impl HostMemoryLength {
    pub(super) fn resolve(self, args: &[Place], i32_ty: TypeId) -> Option<Place> {
        match self {
            HostMemoryLength::Arg(index) => args.get(index).cloned(),
            HostMemoryLength::ConstI32(value) => Some(Place::i32_constant(value, i32_ty)),
        }
    }
}

const FD_READ_SPANS: &[HostMemorySpan] = &[
    HostMemorySpan::IovDescriptor { iovs_arg: 1 },
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: Some(3),
        direction: HostMemoryDirection::Output,
    },
    i32_output(3),
];
const FD_WRITE_SPANS: &[HostMemorySpan] = &[
    HostMemorySpan::IovDescriptor { iovs_arg: 1 },
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: None,
        direction: HostMemoryDirection::Input,
    },
    i32_output(3),
];
const FD_PREAD_SPANS: &[HostMemorySpan] = &[
    HostMemorySpan::IovDescriptor { iovs_arg: 1 },
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: Some(4),
        direction: HostMemoryDirection::Output,
    },
    i32_output(4),
];
const FD_PWRITE_SPANS: &[HostMemorySpan] = &[
    HostMemorySpan::IovDescriptor { iovs_arg: 1 },
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: None,
        direction: HostMemoryDirection::Input,
    },
    i32_output(4),
];
const FD_READDIR_SPANS: &[HostMemorySpan] =
    &[bytes_output(1, HostMemoryLength::Arg(2)), i32_output(4)];
const FD_PRESTAT_DIR_NAME_SPANS: &[HostMemorySpan] = &[bytes_output(1, HostMemoryLength::Arg(2))];
const FD_FDSTAT_GET_SPANS: &[HostMemorySpan] = &[bytes_output(1, HostMemoryLength::ConstI32(24))];
const FD_FILESTAT_GET_SPANS: &[HostMemorySpan] = &[bytes_output(1, HostMemoryLength::ConstI32(64))];
const FD_PRESTAT_GET_SPANS: &[HostMemorySpan] = &[bytes_output(1, HostMemoryLength::ConstI32(8))];
const PATH_OPEN_SPANS: &[HostMemorySpan] =
    &[bytes_input(1, HostMemoryLength::Arg(2)), i32_output(8)];
const PATH_FILESTAT_GET_SPANS: &[HostMemorySpan] = &[
    bytes_input(2, HostMemoryLength::Arg(3)),
    bytes_output(4, HostMemoryLength::ConstI32(64)),
];
const ARGS_SIZES_GET_SPANS: &[HostMemorySpan] = &[i32_output(0), i32_output(1)];
const RANDOM_GET_SPANS: &[HostMemorySpan] = &[bytes_output(0, HostMemoryLength::Arg(1))];
const EMPTY_SPANS: &[HostMemorySpan] = &[];

pub(super) fn host_memory_spans(effect: &EffectOp) -> &'static [HostMemorySpan] {
    match effect {
        EffectOp::ExternalIo { operation } => external_io_host_memory_spans(*operation),
        EffectOp::Nondet {
            operation: NondetOp::RandomGet,
        } => RANDOM_GET_SPANS,
        EffectOp::Nondet {
            operation: NondetOp::ClockTimeGet | NondetOp::ClockResGet,
        }
        | EffectOp::Pure
        | EffectOp::UserCall { .. }
        | EffectOp::IndirectCall { .. }
        | EffectOp::InternalAlloc { .. }
        | EffectOp::UnsafeMemory { .. }
        | EffectOp::Unknown { .. } => EMPTY_SPANS,
    }
}

fn external_io_host_memory_spans(operation: ExternalIoOp) -> &'static [HostMemorySpan] {
    match operation {
        ExternalIoOp::FdRead => FD_READ_SPANS,
        ExternalIoOp::FdWrite => FD_WRITE_SPANS,
        ExternalIoOp::FdPread => FD_PREAD_SPANS,
        ExternalIoOp::FdPwrite => FD_PWRITE_SPANS,
        ExternalIoOp::FdReaddir => FD_READDIR_SPANS,
        ExternalIoOp::FdPrestatDirName => FD_PRESTAT_DIR_NAME_SPANS,
        ExternalIoOp::FdFdstatGet => FD_FDSTAT_GET_SPANS,
        ExternalIoOp::FdFilestatGet => FD_FILESTAT_GET_SPANS,
        ExternalIoOp::FdPrestatGet => FD_PRESTAT_GET_SPANS,
        ExternalIoOp::PathOpen => PATH_OPEN_SPANS,
        ExternalIoOp::PathFilestatGet => PATH_FILESTAT_GET_SPANS,
        ExternalIoOp::ArgsSizesGet | ExternalIoOp::EnvironSizesGet => ARGS_SIZES_GET_SPANS,
        ExternalIoOp::PathCreateDirectory
        | ExternalIoOp::PathFilestatSetTimes
        | ExternalIoOp::PathLink
        | ExternalIoOp::PathReadlink
        | ExternalIoOp::PathRemoveDirectory
        | ExternalIoOp::PathRename
        | ExternalIoOp::PathSymlink
        | ExternalIoOp::PathUnlinkFile
        | ExternalIoOp::FdAdvise
        | ExternalIoOp::FdAllocate
        | ExternalIoOp::FdClose
        | ExternalIoOp::FdDatasync
        | ExternalIoOp::FdFdstatSetFlags
        | ExternalIoOp::FdFdstatSetRights
        | ExternalIoOp::FdFilestatSetSize
        | ExternalIoOp::FdFilestatSetTimes
        | ExternalIoOp::FdRenumber
        | ExternalIoOp::FdSeek
        | ExternalIoOp::FdSync
        | ExternalIoOp::FdTell
        | ExternalIoOp::PollOneoff
        | ExternalIoOp::ProcExit
        | ExternalIoOp::ProcRaise
        | ExternalIoOp::SchedYield
        | ExternalIoOp::SockAccept
        | ExternalIoOp::SockRecv
        | ExternalIoOp::SockSend
        | ExternalIoOp::SockShutdown
        | ExternalIoOp::ArgsGet
        | ExternalIoOp::EnvironGet => EMPTY_SPANS,
    }
}

const fn bytes_input(address_arg: usize, length: HostMemoryLength) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Input,
    }
}

const fn bytes_output(address_arg: usize, length: HostMemoryLength) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Output,
    }
}

const fn i32_output(address_arg: usize) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::ConstI32(4),
        unit: HostMemoryDirectUnit::I32Cell,
        direction: HostMemoryDirection::Output,
    }
}

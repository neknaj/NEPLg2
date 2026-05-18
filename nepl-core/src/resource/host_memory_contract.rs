use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
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
    ArgScaled { arg: usize, bytes_per_item: i32 },
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
        iov_count_arg: usize,
    },
    IovPayload {
        iovs_arg: usize,
        iov_count_arg: usize,
        transferred_count_arg: Option<usize>,
        direction: HostMemoryDirection,
    },
}

impl HostMemoryLength {
    pub(super) fn resolve(
        self,
        args: &[Place],
        i32_ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) -> Option<Place> {
        match self {
            HostMemoryLength::Arg(index) => args.get(index).cloned(),
            HostMemoryLength::ArgScaled {
                arg,
                bytes_per_item,
            } => {
                let arg = args.get(arg)?;
                raw_aliases
                    .i32_value(arg)
                    .and_then(|value| value.checked_mul(bytes_per_item))
                    .map(|value| Place::i32_constant(value, i32_ty))
            }
            HostMemoryLength::ConstI32(value) => Some(Place::i32_constant(value, i32_ty)),
        }
    }
}

const FD_READ_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: Some(3),
        direction: HostMemoryDirection::Output,
    },
    i32_output(3),
];
const FD_WRITE_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: None,
        direction: HostMemoryDirection::Input,
    },
    i32_output(3),
];
const FD_PREAD_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: Some(4),
        direction: HostMemoryDirection::Output,
    },
    i32_output(4),
];
const FD_PWRITE_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
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
const PATH_CREATE_DIRECTORY_SPANS: &[HostMemorySpan] = &[bytes_input(1, HostMemoryLength::Arg(2))];
const PATH_FILESTAT_GET_SPANS: &[HostMemorySpan] = &[
    bytes_input(2, HostMemoryLength::Arg(3)),
    bytes_output(4, HostMemoryLength::ConstI32(64)),
];
const PATH_FILESTAT_SET_TIMES_SPANS: &[HostMemorySpan] =
    &[bytes_input(2, HostMemoryLength::Arg(3))];
const PATH_LINK_SPANS: &[HostMemorySpan] = &[
    bytes_input(2, HostMemoryLength::Arg(3)),
    bytes_input(5, HostMemoryLength::Arg(6)),
];
const PATH_READLINK_SPANS: &[HostMemorySpan] = &[
    bytes_input(1, HostMemoryLength::Arg(2)),
    bytes_output(3, HostMemoryLength::Arg(4)),
    i32_output(5),
];
const PATH_REMOVE_DIRECTORY_SPANS: &[HostMemorySpan] = &[bytes_input(1, HostMemoryLength::Arg(2))];
const PATH_RENAME_SPANS: &[HostMemorySpan] = &[
    bytes_input(1, HostMemoryLength::Arg(2)),
    bytes_input(4, HostMemoryLength::Arg(5)),
];
const PATH_SYMLINK_SPANS: &[HostMemorySpan] = &[
    bytes_input(0, HostMemoryLength::Arg(1)),
    bytes_input(3, HostMemoryLength::Arg(4)),
];
const PATH_UNLINK_FILE_SPANS: &[HostMemorySpan] = &[bytes_input(1, HostMemoryLength::Arg(2))];
const FD_SEEK_SPANS: &[HostMemorySpan] = &[bytes_output(3, HostMemoryLength::ConstI32(8))];
const FD_TELL_SPANS: &[HostMemorySpan] = &[bytes_output(1, HostMemoryLength::ConstI32(8))];
const POLL_ONEOFF_SPANS: &[HostMemorySpan] = &[
    bytes_input(
        0,
        HostMemoryLength::ArgScaled {
            arg: 2,
            bytes_per_item: 48,
        },
    ),
    bytes_output(
        1,
        HostMemoryLength::ArgScaled {
            arg: 2,
            bytes_per_item: 32,
        },
    ),
    i32_output(3),
];
const SOCK_ACCEPT_SPANS: &[HostMemorySpan] = &[i32_output(2)];
const SOCK_RECV_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: Some(4),
        direction: HostMemoryDirection::Output,
    },
    i32_output(4),
    i32_output(5),
];
const SOCK_SEND_SPANS: &[HostMemorySpan] = &[
    iov_descriptor(1, 2),
    HostMemorySpan::IovPayload {
        iovs_arg: 1,
        iov_count_arg: 2,
        transferred_count_arg: None,
        direction: HostMemoryDirection::Input,
    },
    i32_output(4),
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
        ExternalIoOp::PathCreateDirectory => PATH_CREATE_DIRECTORY_SPANS,
        ExternalIoOp::PathFilestatGet => PATH_FILESTAT_GET_SPANS,
        ExternalIoOp::PathFilestatSetTimes => PATH_FILESTAT_SET_TIMES_SPANS,
        ExternalIoOp::PathLink => PATH_LINK_SPANS,
        ExternalIoOp::PathReadlink => PATH_READLINK_SPANS,
        ExternalIoOp::PathRemoveDirectory => PATH_REMOVE_DIRECTORY_SPANS,
        ExternalIoOp::PathRename => PATH_RENAME_SPANS,
        ExternalIoOp::PathSymlink => PATH_SYMLINK_SPANS,
        ExternalIoOp::PathUnlinkFile => PATH_UNLINK_FILE_SPANS,
        ExternalIoOp::FdSeek => FD_SEEK_SPANS,
        ExternalIoOp::FdTell => FD_TELL_SPANS,
        ExternalIoOp::PollOneoff => POLL_ONEOFF_SPANS,
        ExternalIoOp::SockAccept => SOCK_ACCEPT_SPANS,
        ExternalIoOp::SockRecv => SOCK_RECV_SPANS,
        ExternalIoOp::SockSend => SOCK_SEND_SPANS,
        ExternalIoOp::ArgsSizesGet | ExternalIoOp::EnvironSizesGet => ARGS_SIZES_GET_SPANS,
        ExternalIoOp::FdAdvise
        | ExternalIoOp::FdAllocate
        | ExternalIoOp::FdClose
        | ExternalIoOp::FdDatasync
        | ExternalIoOp::FdFdstatSetFlags
        | ExternalIoOp::FdFdstatSetRights
        | ExternalIoOp::FdFilestatSetSize
        | ExternalIoOp::FdFilestatSetTimes
        | ExternalIoOp::FdRenumber
        | ExternalIoOp::FdSync
        | ExternalIoOp::ProcExit
        | ExternalIoOp::ProcRaise
        | ExternalIoOp::SchedYield
        | ExternalIoOp::SockShutdown
        | ExternalIoOp::ArgsGet
        | ExternalIoOp::EnvironGet => EMPTY_SPANS,
    }
}

const fn iov_descriptor(iovs_arg: usize, iov_count_arg: usize) -> HostMemorySpan {
    HostMemorySpan::IovDescriptor {
        iovs_arg,
        iov_count_arg,
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

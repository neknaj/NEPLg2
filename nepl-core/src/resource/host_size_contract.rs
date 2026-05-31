use super::host_memory_contract::HostMemoryDirection;
use super::model::{EffectOp, ExternalIoOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HostSizeKind {
    ArgsCount,
    ArgsBufferBytes,
    EnvironCount,
    EnvironBufferBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HostSizeOutput {
    pub(super) address_arg: usize,
    pub(super) kind: HostSizeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostDependentLength {
    HostSize(HostSizeKind),
    HostSizeScaled {
        kind: HostSizeKind,
        bytes_per_item: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HostDependentMemorySpan {
    pub(super) address_arg: usize,
    pub(super) length: HostDependentLength,
    pub(super) direction: HostMemoryDirection,
}

const ARGS_SIZES_OUTPUTS: &[HostSizeOutput] = &[
    HostSizeOutput {
        address_arg: 0,
        kind: HostSizeKind::ArgsCount,
    },
    HostSizeOutput {
        address_arg: 1,
        kind: HostSizeKind::ArgsBufferBytes,
    },
];

const ENVIRON_SIZES_OUTPUTS: &[HostSizeOutput] = &[
    HostSizeOutput {
        address_arg: 0,
        kind: HostSizeKind::EnvironCount,
    },
    HostSizeOutput {
        address_arg: 1,
        kind: HostSizeKind::EnvironBufferBytes,
    },
];

const ARGS_GET_SPANS: &[HostDependentMemorySpan] = &[
    host_pointer_table_output(0, HostSizeKind::ArgsCount),
    host_byte_buffer_output(1, HostSizeKind::ArgsBufferBytes),
];

const ENVIRON_GET_SPANS: &[HostDependentMemorySpan] = &[
    host_pointer_table_output(0, HostSizeKind::EnvironCount),
    host_byte_buffer_output(1, HostSizeKind::EnvironBufferBytes),
];

const EMPTY_HOST_SIZE_OUTPUTS: &[HostSizeOutput] = &[];
const EMPTY_DEPENDENT_SPANS: &[HostDependentMemorySpan] = &[];

pub(super) fn host_size_outputs(effect: &EffectOp) -> &'static [HostSizeOutput] {
    match effect {
        EffectOp::ExternalIo {
            operation: ExternalIoOp::ArgsSizesGet,
        } => ARGS_SIZES_OUTPUTS,
        EffectOp::ExternalIo {
            operation: ExternalIoOp::EnvironSizesGet,
        } => ENVIRON_SIZES_OUTPUTS,
        EffectOp::ExternalIo { .. }
        | EffectOp::Nondet { .. }
        | EffectOp::Pure
        | EffectOp::UserCall { .. }
        | EffectOp::IndirectCall { .. }
        | EffectOp::InternalAlloc { .. }
        | EffectOp::UnsafeMemory { .. }
        | EffectOp::PrivateState { .. }
        | EffectOp::PrivateCache { .. }
        | EffectOp::Unknown { .. } => EMPTY_HOST_SIZE_OUTPUTS,
    }
}

pub(super) fn dependent_host_memory_spans(effect: &EffectOp) -> &'static [HostDependentMemorySpan] {
    match effect {
        EffectOp::ExternalIo {
            operation: ExternalIoOp::ArgsGet,
        } => ARGS_GET_SPANS,
        EffectOp::ExternalIo {
            operation: ExternalIoOp::EnvironGet,
        } => ENVIRON_GET_SPANS,
        EffectOp::ExternalIo { .. }
        | EffectOp::Nondet { .. }
        | EffectOp::Pure
        | EffectOp::UserCall { .. }
        | EffectOp::IndirectCall { .. }
        | EffectOp::InternalAlloc { .. }
        | EffectOp::UnsafeMemory { .. }
        | EffectOp::PrivateState { .. }
        | EffectOp::PrivateCache { .. }
        | EffectOp::Unknown { .. } => EMPTY_DEPENDENT_SPANS,
    }
}

const fn host_pointer_table_output(
    address_arg: usize,
    kind: HostSizeKind,
) -> HostDependentMemorySpan {
    HostDependentMemorySpan {
        address_arg,
        length: HostDependentLength::HostSizeScaled {
            kind,
            bytes_per_item: 4,
        },
        direction: HostMemoryDirection::Output,
    }
}

const fn host_byte_buffer_output(
    address_arg: usize,
    kind: HostSizeKind,
) -> HostDependentMemorySpan {
    HostDependentMemorySpan {
        address_arg,
        length: HostDependentLength::HostSize(kind),
        direction: HostMemoryDirection::Output,
    }
}

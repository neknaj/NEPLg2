use super::host_memory_contract::{
    host_memory_spans, HostMemoryDirectUnit, HostMemoryDirection, HostMemoryInitializedLength,
    HostMemoryLength, HostMemorySpan,
};
use super::model::{EffectOp, ExternalIoOp};

fn spans_for(operation: ExternalIoOp) -> &'static [HostMemorySpan] {
    host_memory_spans(&EffectOp::ExternalIo { operation })
}

fn bytes_input_arg(address_arg: usize, length_arg: usize) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::Arg(length_arg),
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Input,
    }
}

fn bytes_output_arg(address_arg: usize, length_arg: usize) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::Arg(length_arg),
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Output,
    }
}

fn bytes_output_counted(
    address_arg: usize,
    capacity_arg: usize,
    initialized_count_address_arg: usize,
) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::Arg(capacity_arg),
        initialized_length: HostMemoryInitializedLength::OutputI32Cell {
            address_arg: initialized_count_address_arg,
        },
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Output,
    }
}

fn bytes_output_const(address_arg: usize, length: i32) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::ConstI32(length),
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Output,
    }
}

fn bytes_input_scaled(address_arg: usize, count_arg: usize, bytes_per_item: i32) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::ArgScaled {
            arg: count_arg,
            bytes_per_item,
        },
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Input,
    }
}

fn bytes_output_scaled(
    address_arg: usize,
    count_arg: usize,
    bytes_per_item: i32,
) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::ArgScaled {
            arg: count_arg,
            bytes_per_item,
        },
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::Bytes,
        direction: HostMemoryDirection::Output,
    }
}

fn i32_output(address_arg: usize) -> HostMemorySpan {
    HostMemorySpan::Direct {
        address_arg,
        length: HostMemoryLength::ConstI32(4),
        initialized_length: HostMemoryInitializedLength::SameAsLength,
        unit: HostMemoryDirectUnit::I32Cell,
        direction: HostMemoryDirection::Output,
    }
}

fn iov_descriptor(iovs_arg: usize, iov_count_arg: usize) -> HostMemorySpan {
    HostMemorySpan::IovDescriptor {
        iovs_arg,
        iov_count_arg,
    }
}

fn iov_payload(
    iovs_arg: usize,
    iov_count_arg: usize,
    transferred_count_arg: Option<usize>,
    direction: HostMemoryDirection,
) -> HostMemorySpan {
    HostMemorySpan::IovPayload {
        iovs_arg,
        iov_count_arg,
        transferred_count_arg,
        direction,
    }
}

#[test]
fn same_call_pointer_length_external_io_spans_match_wasi_abi() {
    assert_eq!(
        spans_for(ExternalIoOp::PathCreateDirectory),
        &[bytes_input_arg(1, 2)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathOpen),
        &[bytes_input_arg(2, 3), i32_output(8)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathFilestatSetTimes),
        &[bytes_input_arg(2, 3)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathLink),
        &[bytes_input_arg(2, 3), bytes_input_arg(5, 6)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathReadlink),
        &[bytes_input_arg(1, 2), bytes_output_arg(3, 4), i32_output(5)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::FdReaddir),
        &[bytes_output_counted(1, 2, 4), i32_output(4)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathRemoveDirectory),
        &[bytes_input_arg(1, 2)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathRename),
        &[bytes_input_arg(1, 2), bytes_input_arg(4, 5)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathSymlink),
        &[bytes_input_arg(0, 1), bytes_input_arg(3, 4)]
    );
    assert_eq!(
        spans_for(ExternalIoOp::PathUnlinkFile),
        &[bytes_input_arg(1, 2)]
    );
    assert_eq!(spans_for(ExternalIoOp::FdSeek), &[bytes_output_const(3, 8)]);
    assert_eq!(spans_for(ExternalIoOp::FdTell), &[bytes_output_const(1, 8)]);
    assert_eq!(
        spans_for(ExternalIoOp::PollOneoff),
        &[
            bytes_input_scaled(0, 2, 48),
            bytes_output_scaled(1, 2, 32),
            i32_output(3),
        ]
    );
    assert_eq!(spans_for(ExternalIoOp::SockAccept), &[i32_output(2)]);
    assert_eq!(
        spans_for(ExternalIoOp::SockRecv),
        &[
            iov_descriptor(1, 2),
            iov_payload(1, 2, Some(4), HostMemoryDirection::Output),
            i32_output(4),
            i32_output(5),
        ]
    );
    assert_eq!(
        spans_for(ExternalIoOp::SockSend),
        &[
            iov_descriptor(1, 2),
            iov_payload(1, 2, None, HostMemoryDirection::Input),
            i32_output(4),
        ]
    );
}

use super::model::ExternalIoOp;

pub(super) fn external_io_iov_payload_arg(operation: ExternalIoOp) -> Option<usize> {
    match operation {
        ExternalIoOp::FdRead
        | ExternalIoOp::FdWrite
        | ExternalIoOp::FdPread
        | ExternalIoOp::FdPwrite => Some(1),
        ExternalIoOp::PathOpen
        | ExternalIoOp::PathCreateDirectory
        | ExternalIoOp::PathFilestatGet
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
        | ExternalIoOp::FdFdstatGet
        | ExternalIoOp::FdFdstatSetFlags
        | ExternalIoOp::FdFdstatSetRights
        | ExternalIoOp::FdFilestatGet
        | ExternalIoOp::FdFilestatSetSize
        | ExternalIoOp::FdFilestatSetTimes
        | ExternalIoOp::FdPrestatGet
        | ExternalIoOp::FdPrestatDirName
        | ExternalIoOp::FdReaddir
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
        | ExternalIoOp::ArgsSizesGet
        | ExternalIoOp::EnvironGet
        | ExternalIoOp::EnvironSizesGet => None,
    }
}

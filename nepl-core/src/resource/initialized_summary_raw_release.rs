extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{RawCellReleaseParamRequirement, RawCellReleaseRequirementKind};
use super::initialized_summary_release_build::collect_address_release_requirements;
use super::model::{Place, RawMemoryOp, ResourceLocal};

pub(super) fn collect_raw_memory_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    operation: &RawMemoryOp,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for (arg_index, kind) in release_requirement_args(operation) {
        let Some(address) = args.get(arg_index) else {
            continue;
        };
        collect_address_release_requirements(out, address, kind, raw_aliases, params);
    }
}

fn release_requirement_args(
    operation: &RawMemoryOp,
) -> Vec<(usize, RawCellReleaseRequirementKind)> {
    match operation {
        RawMemoryOp::Store => alloc::vec![(0, RawCellReleaseRequirementKind::Store)],
        RawMemoryOp::Dealloc => alloc::vec![(0, RawCellReleaseRequirementKind::Dealloc)],
        RawMemoryOp::Realloc => alloc::vec![(0, RawCellReleaseRequirementKind::Realloc)],
        RawMemoryOp::Fill => alloc::vec![(0, RawCellReleaseRequirementKind::Fill)],
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => alloc::vec![
            (0, RawCellReleaseRequirementKind::BulkDestination),
            (1, RawCellReleaseRequirementKind::BulkSource),
        ],
        RawMemoryOp::Alloc
        | RawMemoryOp::Load
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => Vec::new(),
    }
}

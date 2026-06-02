extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{RawCellReleaseParamRequirement, RawCellReleaseRequirementKind};
use super::initialized_summary_release_build::collect_address_release_requirements;
use super::model::{Place, RawMemoryOp, ResourceLocal};

pub(super) fn collect_raw_memory_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    types: &TypeCtx,
    operation: &RawMemoryOp,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for (arg_index, kind) in release_requirement_args(operation) {
        let Some(address) = args.get(*arg_index) else {
            continue;
        };
        collect_address_release_requirements(out, types, address, *kind, raw_aliases, params);
    }
}

fn release_requirement_args(
    operation: &RawMemoryOp,
) -> &'static [(usize, RawCellReleaseRequirementKind)] {
    match operation {
        RawMemoryOp::Store | RawMemoryOp::StoreU8 => &[(0, RawCellReleaseRequirementKind::Store)],
        RawMemoryOp::Dealloc => &[(0, RawCellReleaseRequirementKind::Dealloc)],
        RawMemoryOp::Realloc => &[(0, RawCellReleaseRequirementKind::Realloc)],
        RawMemoryOp::FillBytes | RawMemoryOp::Fill => &[(0, RawCellReleaseRequirementKind::Fill)],
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => &[
            (0, RawCellReleaseRequirementKind::BulkDestination),
            (1, RawCellReleaseRequirementKind::BulkSource),
        ],
        RawMemoryOp::Alloc
        | RawMemoryOp::Load
        | RawMemoryOp::LoadU8
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => &[],
    }
}

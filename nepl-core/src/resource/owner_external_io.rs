extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::external_io_iov_layout::{
    iov_buffer_pointer_cells, iov_length_cell, raw_cell_is_under_any_address,
};
use super::host_memory_address::host_memory_address_place;
use super::host_memory_contract::host_memory_spans;
use super::host_memory_contract::HostMemoryDirection;
use super::host_size_contract::dependent_host_memory_spans;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{OwnerExtentProof, PendingOwnerExtentRequirement};
use super::owner_extent_compare::comparable_owner_extent;
use super::owner_extent_coverage::prove_owner_extent_covers_argument;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_external_io_owner_spans_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        effect: &super::model::EffectOp,
        args: &[Place],
        span: Span,
    ) -> bool {
        let mut available = true;
        for contract in host_memory_spans(effect) {
            available &= self.ensure_host_memory_contract_owner_span_available(
                owners,
                raw_aliases,
                contract,
                args,
                span,
            );
        }
        for contract in dependent_host_memory_spans(effect) {
            available &= self.ensure_dependent_host_memory_owner_span_available(
                owners,
                raw_aliases,
                contract,
                args,
                span,
            );
        }
        if available {
            self.record_host_size_outputs(raw_aliases, effect, args);
        }
        available
    }

    pub(super) fn ensure_iov_payload_owner_extents_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        iov_count: Option<&Place>,
        direction: HostMemoryDirection,
        span: Span,
    ) -> bool {
        let (Some(iovs), Some(iov_count)) = (iovs, iov_count) else {
            return true;
        };
        let iovs = host_memory_address_place(self.types, raw_aliases, iovs);
        let iov_aliases = raw_aliases.aliases_for(&iovs);
        let mut available = true;
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            let Some(length_cell) = iov_length_cell(&buffer_cell, self.types.i32()) else {
                if self.try_record_deferred_host_memory_span_requirement(
                    raw_aliases,
                    &super::host_memory_contract::HostMemorySpan::IovPayload {
                        iovs_arg: 0,
                        iov_count_arg: 1,
                        transferred_count_arg: None,
                        direction,
                    },
                    &[iovs.clone(), iov_count.clone()],
                ) {
                    continue;
                }
                self.push_unavailable(
                    ResourceOwnerOperation::ExternalIoPayloadExtent,
                    &buffer_cell,
                    OwnerState::NoFreeObligation,
                    span,
                );
                available = false;
                continue;
            };
            let length_cell = raw_aliases.canonicalize(&length_cell);
            let payload_buffers =
                iov_payload_buffer_aliases(raw_aliases, &buffer_cell, &iov_aliases);
            if payload_buffers.is_empty() {
                if self.try_record_deferred_host_memory_span_requirement(
                    raw_aliases,
                    &super::host_memory_contract::HostMemorySpan::IovPayload {
                        iovs_arg: 0,
                        iov_count_arg: 1,
                        transferred_count_arg: None,
                        direction,
                    },
                    &[iovs.clone(), iov_count.clone()],
                ) {
                    continue;
                }
                self.push_unavailable(
                    ResourceOwnerOperation::ExternalIoPayloadExtent,
                    &buffer_cell,
                    OwnerState::NoFreeObligation,
                    span,
                );
                available = false;
                continue;
            }
            for buffer in payload_buffers {
                available &= self.ensure_external_io_payload_extent_available(
                    owners,
                    raw_aliases,
                    &buffer,
                    &length_cell,
                    direction,
                    span,
                );
            }
        }
        available
    }

    pub(super) fn ensure_external_io_payload_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        length: &Place,
        direction: HostMemoryDirection,
        span: Span,
    ) -> bool {
        let buffer = host_memory_address_place(self.types, raw_aliases, buffer);
        let resolved = resolve_owner_alias_place(owners, raw_aliases, &buffer);
        let state = owners
            .state(&resolved)
            .unwrap_or(OwnerState::NoFreeObligation);
        let OwnerState::Live { extent, .. } = &state else {
            if self.try_record_deferred_direct_host_memory_span_requirement(
                raw_aliases,
                &buffer,
                length,
                direction,
            ) {
                return true;
            }
            self.push_unavailable(
                ResourceOwnerOperation::ExternalIoPayloadExtent,
                &resolved,
                state,
                span,
            );
            return false;
        };
        let extent = comparable_owner_extent(&resolved, extent.clone());
        match prove_owner_extent_covers_argument(raw_aliases, &extent, length) {
            OwnerExtentProof::Proven => true,
            OwnerExtentProof::Unknown => {
                if self.try_record_deferred_direct_host_memory_span_requirement(
                    raw_aliases,
                    &buffer,
                    length,
                    direction,
                ) {
                    return true;
                }
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: resolved,
                        expected: OwnerStorageExtent::payload_bytes(length),
                        operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                    });
                true
            }
            OwnerExtentProof::Mismatch => {
                self.push_unavailable(
                    ResourceOwnerOperation::ExternalIoPayloadExtent,
                    &resolved,
                    state,
                    span,
                );
                false
            }
        }
    }
}

fn iov_payload_buffer_aliases(
    raw_aliases: &RawCellAddressAliases,
    buffer_cell: &Place,
    iov_aliases: &[Place],
) -> Vec<Place> {
    raw_aliases
        .aliases_for(buffer_cell)
        .into_iter()
        .filter(|buffer| {
            buffer != buffer_cell && !raw_cell_is_under_any_address(buffer, iov_aliases)
        })
        .collect()
}

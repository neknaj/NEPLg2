extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::external_io_iov_layout::{
    iov_buffer_pointer_cells, iov_length_cell, raw_cell_is_under_any_address,
};
use super::host_memory_address::host_memory_address_place;
use super::host_memory_contract::{HostMemoryDirection, HostMemorySpan};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{OwnerExtentProof, PendingOwnerExtentRequirement};
use super::owner_extent_coverage::prove_owner_extent_covers_argument;
use super::owner_host_payload_extent::host_payload_address_value_candidates;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_iov_payload_owner_extents_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        iovs: Option<&Place>,
        iov_count: Option<&Place>,
        direction: HostMemoryDirection,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        let (Some(iovs), Some(iov_count)) = (iovs, iov_count) else {
            return true;
        };
        let iovs = host_memory_address_place(self.types, raw_aliases, iovs);
        let iov_aliases = host_payload_address_value_candidates(raw_aliases, &iovs);
        let mut available = true;
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            let Some(length_cell) = iov_length_cell(&buffer_cell, self.types.i32()) else {
                if self.defer_iov_payload_requirement(
                    raw_aliases,
                    &iovs,
                    iov_count,
                    direction,
                    operation,
                ) {
                    continue;
                }
                self.push_unavailable(operation, &buffer_cell, OwnerState::NoFreeObligation, span);
                available = false;
                continue;
            };
            let length_cell = raw_aliases.canonicalize(&length_cell);
            let payload_buffers =
                iov_payload_buffer_aliases(raw_aliases, &buffer_cell, &iov_aliases);
            if payload_buffers.is_empty() {
                if self.defer_iov_payload_requirement(
                    raw_aliases,
                    &iovs,
                    iov_count,
                    direction,
                    operation,
                ) {
                    continue;
                }
                self.push_unavailable(operation, &buffer_cell, OwnerState::NoFreeObligation, span);
                available = false;
                continue;
            }
            for buffer in payload_buffers {
                available &= self.ensure_memory_payload_extent_available(
                    owners,
                    raw_aliases,
                    raw_views,
                    &buffer,
                    &length_cell,
                    direction,
                    operation,
                    span,
                );
            }
        }
        available
    }

    pub(super) fn ensure_memory_payload_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        buffer: &Place,
        length: &Place,
        direction: HostMemoryDirection,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        let buffer = host_memory_address_place(self.types, raw_aliases, buffer);
        if let Some(result) = self.try_ensure_known_host_payload_extent_available(
            owners,
            raw_aliases,
            &buffer,
            length,
            direction,
            operation,
            span,
        ) {
            return result;
        }
        let resolved = resolve_owner_alias_place(owners, raw_aliases, &buffer);
        let state = owners
            .state(&resolved)
            .unwrap_or(OwnerState::NoFreeObligation);
        let OwnerState::Live { extent, .. } = &state else {
            if self.try_record_deferred_direct_memory_span_requirement(
                raw_aliases,
                &buffer,
                length,
                direction,
                operation,
            ) {
                return true;
            }
            if direction == HostMemoryDirection::Input
                && non_owning_input_payload_has_no_free_obligation(
                    owners,
                    raw_aliases,
                    raw_views,
                    &buffer,
                )
            {
                return true;
            }
            if operation == ResourceOwnerOperation::RawMemoryPayloadExtent {
                return true;
            }
            self.push_unavailable(operation, &resolved, state, span);
            return false;
        };
        let extent =
            super::owner_extent_compare::comparable_owner_extent(&resolved, extent.clone());
        match prove_owner_extent_covers_argument(raw_aliases, &extent, length) {
            OwnerExtentProof::Proven => true,
            OwnerExtentProof::Unknown => {
                if self.try_record_deferred_direct_memory_span_requirement(
                    raw_aliases,
                    &buffer,
                    length,
                    direction,
                    operation,
                ) {
                    return true;
                }
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: resolved,
                        expected: OwnerStorageExtent::payload_bytes(length),
                        operation,
                    });
                true
            }
            OwnerExtentProof::Mismatch => {
                self.push_unavailable(operation, &resolved, state, span);
                false
            }
        }
    }

    fn defer_iov_payload_requirement(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        iovs: &Place,
        iov_count: &Place,
        direction: HostMemoryDirection,
        operation: ResourceOwnerOperation,
    ) -> bool {
        self.try_record_deferred_memory_span_requirement(
            raw_aliases,
            &HostMemorySpan::IovPayload {
                iovs_arg: 0,
                iov_count_arg: 1,
                transferred_count_arg: None,
                direction,
            },
            &[iovs.clone(), iov_count.clone()],
            operation,
        )
    }
}

fn non_owning_input_payload_has_no_free_obligation(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    raw_views: &RawAddressViewTable,
    buffer: &Place,
) -> bool {
    let candidates = raw_aliases.raw_address_aliases_for_value(buffer);
    if !candidates
        .iter()
        .any(|candidate| raw_views.contains_non_owning(candidate))
    {
        return false;
    }

    // Host input reads bytes through the view but does not acquire or release the
    // backing storage. Initialized-cell checking proves that the readable byte span is
    // valid; the owner checker only needs to reject cases that are already known to
    // carry a transferable owner at this address.
    !candidates
        .iter()
        .any(|candidate| owners.has_transferable_owner(candidate))
}

fn iov_payload_buffer_aliases(
    raw_aliases: &RawCellAddressAliases,
    buffer_cell: &Place,
    iov_aliases: &[Place],
) -> Vec<Place> {
    let candidates = host_payload_address_value_candidates(raw_aliases, buffer_cell);
    let out = candidates
        .into_iter()
        .filter(|buffer| {
            buffer != buffer_cell && !raw_cell_is_under_any_address(buffer, iov_aliases)
        })
        .collect::<Vec<_>>();
    out
}

extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::external_io_iov_contract::external_io_iov_payload_arg;
use super::external_io_iov_layout::{
    iov_buffer_pointer_cells, iov_length_cell, raw_cell_is_under_any_address,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{
    prove_owner_extent_matches_argument, OwnerExtentProof, PendingOwnerExtentRequirement,
};
use super::owner_extent_compare::comparable_owner_extent;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_external_io_owner_spans_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        effect: &EffectOp,
        args: &[Place],
        span: Span,
    ) -> bool {
        let operation = match effect {
            EffectOp::ExternalIo { operation } => *operation,
            EffectOp::Pure
            | EffectOp::UserCall { .. }
            | EffectOp::IndirectCall { .. }
            | EffectOp::InternalAlloc { .. }
            | EffectOp::UnsafeMemory { .. }
            | EffectOp::Nondet { .. }
            | EffectOp::Unknown { .. } => return true,
        };
        let Some(iovs_arg) = external_io_iov_payload_arg(operation) else {
            return true;
        };
        self.ensure_iov_payload_owner_extents_available(
            owners,
            raw_aliases,
            args.get(iovs_arg),
            span,
        )
    }

    fn ensure_iov_payload_owner_extents_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        span: Span,
    ) -> bool {
        let Some(iovs) = iovs else {
            return true;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_aliases = raw_aliases.aliases_for(&iovs);
        let mut available = true;
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            let Some(length_cell) = iov_length_cell(&buffer_cell, self.types.i32()) else {
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
                    span,
                );
            }
        }
        available
    }

    fn ensure_external_io_payload_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        length: &Place,
        span: Span,
    ) -> bool {
        let resolved = resolve_owner_alias_place(owners, raw_aliases, buffer);
        let state = owners
            .state(&resolved)
            .unwrap_or(OwnerState::NoFreeObligation);
        let OwnerState::Live { extent, .. } = &state else {
            self.push_unavailable(
                ResourceOwnerOperation::ExternalIoPayloadExtent,
                &resolved,
                state,
                span,
            );
            return false;
        };
        let extent = comparable_owner_extent(&resolved, extent.clone());
        match prove_owner_extent_matches_argument(raw_aliases, &extent, length) {
            OwnerExtentProof::Proven => true,
            OwnerExtentProof::Unknown => {
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

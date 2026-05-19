use crate::span::Span;

use super::host_dependent_length::dependent_host_length_candidates;
use super::host_memory_address::host_memory_address_place;
use super::host_size_contract::HostDependentMemorySpan;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{OwnerExtentProof, PendingOwnerExtentRequirement};
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_dependent_host_memory_owner_span_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        contract: &HostDependentMemorySpan,
        args: &[Place],
        span: Span,
    ) -> bool {
        let Some(buffer) = args.get(contract.address_arg) else {
            return true;
        };
        let required_lengths = dependent_host_length_candidates(raw_aliases, contract.length);
        self.ensure_external_io_payload_extent_covers_any(
            owners,
            raw_aliases,
            buffer,
            &required_lengths,
            span,
        )
    }

    fn ensure_external_io_payload_extent_covers_any(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        required_lengths: &[Place],
        span: Span,
    ) -> bool {
        let buffer = host_memory_address_place(self.types, raw_aliases, buffer);
        if let Some(result) = self.try_ensure_known_dependent_host_payload_extent_available(
            owners,
            raw_aliases,
            &buffer,
            required_lengths,
            span,
        ) {
            return result;
        }
        let resolved = resolve_owner_alias_place(owners, raw_aliases, &buffer);
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
        let extent =
            super::owner_extent_compare::comparable_owner_extent(&resolved, extent.clone());
        if required_lengths.is_empty() {
            self.push_unavailable(
                ResourceOwnerOperation::ExternalIoPayloadExtent,
                &resolved,
                state,
                span,
            );
            return false;
        }
        let mut pending_requirement = None;
        for required in required_lengths {
            match super::owner_extent_coverage::prove_owner_extent_covers_argument(
                raw_aliases,
                &extent,
                required,
            ) {
                OwnerExtentProof::Proven => return true,
                OwnerExtentProof::Unknown => pending_requirement.get_or_insert(required.clone()),
                OwnerExtentProof::Mismatch => continue,
            };
        }
        if let Some(required) = pending_requirement {
            self.owner_extent_requirements
                .push(PendingOwnerExtentRequirement {
                    owner: resolved,
                    expected: OwnerStorageExtent::payload_bytes(&required),
                    operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                });
            return true;
        }
        self.push_unavailable(
            ResourceOwnerOperation::ExternalIoPayloadExtent,
            &resolved,
            state,
            span,
        );
        false
    }
}

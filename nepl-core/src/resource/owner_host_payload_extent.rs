extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::host_memory_contract::HostMemoryDirection;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place, PlaceProjection, ResourceOffset};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{OwnerExtentProof, PendingOwnerExtentRequirement};
use super::owner_extent_compare::comparable_owner_extent;
use super::owner_extent_coverage::prove_owner_extent_covers_argument;
use super::owner_state::OwnerTable;
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::report::ResourceOwnerOperation;

pub(super) struct HostPayloadOwner {
    pub(super) owner: Place,
    pub(super) state: OwnerState,
    extent: OwnerStorageExtent,
    address: Place,
    address_base: Place,
}

impl HostPayloadOwner {
    pub(super) fn comparable_extent(&self) -> OwnerStorageExtent {
        comparable_owner_extent(&self.owner, self.extent.clone())
    }

    pub(super) fn required_extent_for_address(
        &self,
        raw_aliases: &RawCellAddressAliases,
        required: &Place,
    ) -> Option<Place> {
        let offset = known_storage_offset_between(&self.address_base, &self.address)?;
        required_extent_with_offset(raw_aliases, required, offset)
    }
}

pub(super) fn find_host_payload_owner(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
) -> Option<HostPayloadOwner> {
    for (base, address) in host_payload_owner_candidates(raw_aliases, address) {
        let owner = resolve_owner_alias_place(owners, raw_aliases, &base);
        let state = owners.state(&owner).unwrap_or(OwnerState::NoFreeObligation);
        if let OwnerState::Live { extent, .. } = &state {
            return Some(HostPayloadOwner {
                owner,
                extent: extent.clone(),
                state: state.clone(),
                address,
                address_base: base,
            });
        }
        if let Some(state @ OwnerState::Live { extent, .. }) = owners.state(&base).as_ref() {
            return Some(HostPayloadOwner {
                owner: base.clone(),
                extent: extent.clone(),
                state: state.clone(),
                address,
                address_base: base,
            });
        }
    }
    None
}

pub(super) fn prove_host_payload_extent_covers_argument(
    raw_aliases: &RawCellAddressAliases,
    candidate: &HostPayloadOwner,
    required: &Place,
) -> Option<(OwnerExtentProof, Place)> {
    let required_extent = candidate.required_extent_for_address(raw_aliases, required)?;
    let proof = prove_owner_extent_covers_argument(
        raw_aliases,
        &candidate.comparable_extent(),
        &required_extent,
    );
    Some((proof, required_extent))
}

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn try_ensure_known_host_payload_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        length: &Place,
        direction: HostMemoryDirection,
        span: Span,
    ) -> Option<bool> {
        let candidate = find_host_payload_owner(owners, raw_aliases, buffer)?;
        match prove_host_payload_extent_covers_argument(raw_aliases, &candidate, length) {
            Some((OwnerExtentProof::Proven, _)) => Some(true),
            Some((OwnerExtentProof::Unknown, required_extent)) => {
                if self.try_record_deferred_direct_host_memory_span_requirement(
                    raw_aliases,
                    &candidate.address_base,
                    &required_extent,
                    direction,
                ) {
                    return Some(true);
                }
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: candidate.owner,
                        expected: OwnerStorageExtent::payload_bytes(&required_extent),
                        operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                    });
                Some(true)
            }
            Some((OwnerExtentProof::Mismatch, _)) => {
                self.push_unavailable(
                    ResourceOwnerOperation::ExternalIoPayloadExtent,
                    &candidate.owner,
                    candidate.state,
                    span,
                );
                Some(false)
            }
            None => None,
        }
    }

    pub(super) fn try_ensure_known_dependent_host_payload_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        required_lengths: &[Place],
        span: Span,
    ) -> Option<bool> {
        let candidate = find_host_payload_owner(owners, raw_aliases, buffer)?;
        if required_lengths.is_empty() {
            self.push_unavailable(
                ResourceOwnerOperation::ExternalIoPayloadExtent,
                &candidate.owner,
                candidate.state,
                span,
            );
            return Some(false);
        }
        let mut pending_requirement = None;
        for required in required_lengths {
            let Some((proof, required_extent)) =
                prove_host_payload_extent_covers_argument(raw_aliases, &candidate, required)
            else {
                continue;
            };
            match proof {
                OwnerExtentProof::Proven => return Some(true),
                OwnerExtentProof::Unknown => pending_requirement.get_or_insert(required_extent),
                OwnerExtentProof::Mismatch => continue,
            };
        }
        if let Some(required) = pending_requirement {
            self.owner_extent_requirements
                .push(PendingOwnerExtentRequirement {
                    owner: candidate.owner,
                    expected: OwnerStorageExtent::payload_bytes(&required),
                    operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                });
            return Some(true);
        }
        self.push_unavailable(
            ResourceOwnerOperation::ExternalIoPayloadExtent,
            &candidate.owner,
            candidate.state,
            span,
        );
        Some(false)
    }
}

fn host_payload_owner_candidates(
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
) -> Vec<(Place, Place)> {
    let mut addresses = Vec::new();
    push_unique_place(&mut addresses, address);
    push_unique_place(&mut addresses, &raw_aliases.canonicalize(address));
    for alias in raw_aliases.aliases_for(address) {
        push_unique_place(&mut addresses, &alias);
    }
    for alias in raw_aliases.prefix_aliases_for(address) {
        push_unique_place(&mut addresses, &alias);
    }
    let mut out = Vec::new();
    for address in addresses {
        if let Some(base) = trailing_storage_base(&address) {
            push_unique_candidate(&mut out, base, address);
        }
    }
    out
}

fn trailing_storage_base(address: &Place) -> Option<Place> {
    let mut base = address.clone();
    let mut changed = false;
    while matches!(
        base.projections.last(),
        Some(PlaceProjection::StorageOffset(_))
    ) {
        base.projections.pop();
        changed = true;
    }
    Some(if changed { base } else { address.clone() })
}

fn known_storage_offset_between(base: &Place, address: &Place) -> Option<usize> {
    let suffix = place_suffix_after_prefix(address, base)?;
    let mut total = 0usize;
    for projection in suffix {
        let PlaceProjection::StorageOffset(ResourceOffset::Known(offset)) = projection else {
            return None;
        };
        total = total.checked_add(offset)?;
    }
    Some(total)
}

fn required_extent_with_offset(
    raw_aliases: &RawCellAddressAliases,
    required: &Place,
    offset: usize,
) -> Option<Place> {
    if offset == 0 {
        return Some(raw_aliases.canonicalize_scalar(required));
    }
    let offset_i32 = i32::try_from(offset).ok()?;
    if let Some(required_value) = raw_aliases.i32_value(required) {
        return required_value
            .checked_add(offset_i32)
            .map(|value| Place::i32_constant(value, required.ty));
    }
    for (target, target_offset) in raw_aliases.i32_offset_targets(required) {
        if target_offset == i64::from(offset_i32) {
            return Some(target);
        }
    }
    None
}

fn push_unique_candidate(out: &mut Vec<(Place, Place)>, base: Place, address: Place) {
    if !out.iter().any(|(existing_base, existing_address)| {
        *existing_base == base && *existing_address == address
    }) {
        out.push((base, address));
    }
}

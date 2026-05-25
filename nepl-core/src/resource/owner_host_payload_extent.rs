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
        let offset = storage_offset_between(&self.address_base, &self.address)?;
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
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> Option<bool> {
        let candidate = find_host_payload_owner(owners, raw_aliases, buffer)?;
        match prove_host_payload_extent_covers_argument(raw_aliases, &candidate, length) {
            Some((OwnerExtentProof::Proven, _)) => Some(true),
            Some((OwnerExtentProof::Unknown, required_extent)) => {
                if self.try_record_deferred_direct_memory_span_requirement(
                    raw_aliases,
                    &candidate.address_base,
                    &required_extent,
                    direction,
                    operation,
                ) {
                    return Some(true);
                }
                self.owner_extent_requirements
                    .push(PendingOwnerExtentRequirement {
                        owner: candidate.owner,
                        expected: OwnerStorageExtent::payload_bytes(&required_extent),
                        operation,
                    });
                Some(true)
            }
            Some((OwnerExtentProof::Mismatch, _)) => {
                self.push_unavailable(operation, &candidate.owner, candidate.state, span);
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
    let mut out = Vec::new();
    for address in host_payload_address_value_candidates(raw_aliases, address) {
        if let Some(base) = trailing_storage_base(&address) {
            push_unique_candidate(&mut out, base, address);
        }
    }
    out
}

pub(super) fn host_payload_address_value_candidates(
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
) -> Vec<Place> {
    let mut addresses = Vec::new();
    push_unique_place(&mut addresses, address);
    let mut index = 0;
    while index < addresses.len() {
        let candidate = addresses[index].clone();
        index += 1;
        push_host_payload_address_aliases(raw_aliases, &mut addresses, &candidate);
        push_host_payload_i32_offset_sources(raw_aliases, &mut addresses, &candidate);
    }
    addresses
}

fn push_host_payload_address_aliases(
    raw_aliases: &RawCellAddressAliases,
    addresses: &mut Vec<Place>,
    address: &Place,
) {
    push_unique_place(addresses, &raw_aliases.canonicalize(address));
    for alias in raw_aliases.raw_address_aliases_for_value(address) {
        push_unique_place(addresses, &alias);
    }
    for alias in raw_aliases.prefix_aliases_for(address) {
        push_unique_place(addresses, &alias);
    }
}

fn push_host_payload_i32_offset_sources(
    raw_aliases: &RawCellAddressAliases,
    addresses: &mut Vec<Place>,
    address: &Place,
) {
    // A raw iovec descriptor stores its buffer pointer as an i32 cell.  Pointer
    // arithmetic such as `add buf 0` is represented as an i32 offset fact instead
    // of a raw-address alias group, so host memory checks must recover the owner
    // base from that scalar relation before comparing extents.
    for (source, offset) in raw_aliases.i32_offset_sources(address) {
        if let Some(address) = place_with_known_i32_storage_offset(&source, offset) {
            push_unique_place(addresses, &address);
        }
    }
}

fn place_with_known_i32_storage_offset(source: &Place, offset: i64) -> Option<Place> {
    let offset = usize::try_from(offset).ok()?;
    let mut address = source.clone();
    if offset != 0 {
        add_known_storage_offset(&mut address, offset)?;
    }
    Some(address)
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum HostPayloadAddressOffset {
    Known(usize),
    Symbolic { place: Place, offset: i64 },
}

fn storage_offset_between(base: &Place, address: &Place) -> Option<HostPayloadAddressOffset> {
    let suffix = place_suffix_after_prefix(address, base)?;
    let mut total = HostPayloadAddressOffset::Known(0);
    for projection in suffix {
        let PlaceProjection::StorageOffset(ResourceOffset::Known(offset)) = projection else {
            let PlaceProjection::StorageOffset(offset) = projection else {
                return None;
            };
            add_storage_offset_component(&mut total, offset)?;
            continue;
        };
        add_known_host_payload_offset(&mut total, offset)?;
    }
    Some(total)
}

fn add_storage_offset_component(
    total: &mut HostPayloadAddressOffset,
    offset: ResourceOffset,
) -> Option<()> {
    match offset {
        ResourceOffset::Known(bytes) => add_known_host_payload_offset(total, bytes),
        ResourceOffset::Symbolic { place } => add_symbolic_host_payload_offset(total, *place, 0),
        ResourceOffset::Offset { place, offset } => {
            add_symbolic_host_payload_offset(total, *place, offset)
        }
        ResourceOffset::ScaledSymbolic { place, scale } if scale == 1 => {
            add_symbolic_host_payload_offset(total, *place, 0)
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } if scale == 1 => add_symbolic_host_payload_offset(total, *place, offset),
        ResourceOffset::ScaledSymbolic { .. }
        | ResourceOffset::ScaledOffset { .. }
        | ResourceOffset::Unknown => None,
    }
}

fn add_known_host_payload_offset(total: &mut HostPayloadAddressOffset, bytes: usize) -> Option<()> {
    match total {
        HostPayloadAddressOffset::Known(existing) => {
            *existing = existing.checked_add(bytes)?;
        }
        HostPayloadAddressOffset::Symbolic { offset, .. } => {
            let bytes = i64::try_from(bytes).ok()?;
            *offset = offset.checked_add(bytes)?;
        }
    }
    Some(())
}

fn add_symbolic_host_payload_offset(
    total: &mut HostPayloadAddressOffset,
    place: Place,
    offset: i64,
) -> Option<()> {
    match total {
        HostPayloadAddressOffset::Known(existing) => {
            let existing = i64::try_from(*existing).ok()?;
            *total = HostPayloadAddressOffset::Symbolic {
                place,
                offset: existing.checked_add(offset)?,
            };
        }
        HostPayloadAddressOffset::Symbolic { .. } => return None,
    }
    Some(())
}

fn add_known_storage_offset(place: &mut Place, offset: usize) -> Option<()> {
    match place.projections.last_mut() {
        Some(PlaceProjection::StorageOffset(ResourceOffset::Known(existing))) => {
            *existing = existing.checked_add(offset)?;
        }
        _ => place
            .projections
            .push(PlaceProjection::StorageOffset(ResourceOffset::Known(
                offset,
            ))),
    }
    Some(())
}

fn required_extent_with_offset(
    raw_aliases: &RawCellAddressAliases,
    required: &Place,
    offset: HostPayloadAddressOffset,
) -> Option<Place> {
    match offset {
        HostPayloadAddressOffset::Known(offset) => {
            required_extent_with_known_offset(raw_aliases, required, offset)
        }
        HostPayloadAddressOffset::Symbolic { place, offset } => {
            required_extent_with_symbolic_offset(raw_aliases, required, &place, offset)
        }
    }
}

fn required_extent_with_known_offset(
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

fn required_extent_with_symbolic_offset(
    raw_aliases: &RawCellAddressAliases,
    required: &Place,
    offset_place: &Place,
    known_offset: i64,
) -> Option<Place> {
    let offset_place = raw_aliases.canonicalize_scalar(offset_place);
    for (minuend, subtrahend) in raw_aliases.i32_difference_sources(required) {
        if raw_aliases.canonicalize_scalar(&subtrahend) == offset_place {
            return required_extent_with_signed_known_offset(raw_aliases, &minuend, known_offset);
        }
    }
    let required_value = raw_aliases.i32_value(required)?;
    let offset_value = raw_aliases.i32_value(&offset_place)?;
    let total = i64::from(required_value)
        .checked_add(i64::from(offset_value))?
        .checked_add(known_offset)?;
    i32::try_from(total)
        .ok()
        .map(|value| Place::i32_constant(value, required.ty))
}

fn required_extent_with_signed_known_offset(
    raw_aliases: &RawCellAddressAliases,
    required: &Place,
    offset: i64,
) -> Option<Place> {
    if offset == 0 {
        return Some(raw_aliases.canonicalize_scalar(required));
    }
    let offset = usize::try_from(offset).ok()?;
    required_extent_with_known_offset(raw_aliases, required, offset)
}

fn push_unique_candidate(out: &mut Vec<(Place, Place)>, base: Place, address: Place) {
    if !out.iter().any(|(existing_base, existing_address)| {
        *existing_base == base && *existing_address == address
    }) {
        out.push((base, address));
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;

    use crate::types::TypeCtx;

    use super::*;
    use crate::resource::{OwnerStorageExtent, PlaceRoot};

    fn local(name: &str, ty: crate::types::TypeId) -> Place {
        Place {
            root: PlaceRoot::Local(String::from(name)),
            projections: Vec::new(),
            ty,
        }
    }

    #[test]
    fn host_payload_address_candidates_include_zero_offset_raw_pointer_source() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let mut raw_aliases = RawCellAddressAliases::default();
        let buffer = local("buf", i32_ty);
        let stored_cell = local("iov_cell", i32_ty).with_projection(PlaceProjection::Deref, i32_ty);

        raw_aliases.mark(&buffer);
        raw_aliases.add_i32_offset(&buffer, &stored_cell, 0);

        let candidates = host_payload_address_value_candidates(&raw_aliases, &stored_cell);
        assert!(
            candidates.iter().any(|candidate| candidate == &buffer),
            "i32 offset facts for stored raw pointer values must expose the buffer owner base: {candidates:?}"
        );
    }

    #[test]
    fn host_payload_address_candidates_follow_stored_i32_offset_value_copy() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let mut raw_aliases = RawCellAddressAliases::default();
        let buffer = local("buf", i32_ty);
        let arithmetic_result = local("add_result", i32_ty);
        let stored_cell = local("iov_cell", i32_ty).with_projection(PlaceProjection::Deref, i32_ty);

        raw_aliases.mark(&buffer);
        raw_aliases.add_i32_offset(&buffer, &arithmetic_result, 0);
        raw_aliases.copy_scalar_facts_if_tracked(&arithmetic_result, &stored_cell);

        let candidates = host_payload_address_value_candidates(&raw_aliases, &stored_cell);
        assert!(
            candidates.iter().any(|candidate| candidate == &buffer),
            "raw memory stores copy scalar pointer facts into the descriptor cell: {candidates:?}"
        );
    }

    #[test]
    fn host_payload_owner_uses_i32_offset_to_compare_payload_extent_from_owner_base() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut owners = OwnerTable::default();
        let buffer = local("buf", i32_ty);
        let stored_cell = local("iov_cell", i32_ty).with_projection(PlaceProjection::Deref, i32_ty);
        let allocated_bytes = Place::i32_constant(16, i32_ty);
        let requested_bytes = Place::i32_constant(8, i32_ty);

        raw_aliases.mark(&buffer);
        raw_aliases.add_i32_offset(&buffer, &stored_cell, 4);
        owners.allocate_with_extent(&buffer, OwnerStorageExtent::payload_bytes(&allocated_bytes));

        let owner = find_host_payload_owner(&owners, &raw_aliases, &stored_cell)
            .expect("stored raw pointer value should resolve to its owner base");
        assert_eq!(owner.owner, buffer);
        assert_eq!(
            owner.required_extent_for_address(&raw_aliases, &requested_bytes),
            Some(Place::i32_constant(12, i32_ty))
        );
    }

    #[test]
    fn host_payload_owner_uses_difference_to_cover_symbolic_tail_extent() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut owners = OwnerTable::default();
        let buffer = local("buf", i32_ty);
        let capacity = local("cap", i32_ty);
        let initialized_len = local("len", i32_ty);
        let remaining_len = local("remaining", i32_ty);
        let tail = buffer.clone().with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                place: Box::new(initialized_len.clone()),
            }),
            i32_ty,
        );

        raw_aliases.mark(&buffer);
        raw_aliases.add_i32_difference(&capacity, &initialized_len, &remaining_len);
        owners.allocate_with_extent(&buffer, OwnerStorageExtent::payload_bytes(&capacity));

        let owner = find_host_payload_owner(&owners, &raw_aliases, &tail)
            .expect("symbolic tail address should resolve to the allocation owner base");
        assert_eq!(owner.owner, buffer);
        assert_eq!(
            owner.required_extent_for_address(&raw_aliases, &remaining_len),
            Some(capacity)
        );
    }
}

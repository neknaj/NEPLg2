extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::host_memory_contract::{
    HostMemoryDirectUnit, HostMemoryDirection, HostMemoryLength, HostMemorySpan,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp, ResourceLocal,
    ResourceOffset,
};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_return_apply_place::owner_projection_source_place;
use super::owner_state::OwnerTable;
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::summary::{
    OwnerHostMemoryArgSummary, OwnerHostMemorySpanRequirement, OwnerProjectionSource,
};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn try_record_deferred_host_memory_span_requirement(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        contract: &HostMemorySpan,
        args: &[Place],
    ) -> bool {
        let Some(requirement) =
            summarize_host_memory_span_requirement(raw_aliases, self.params, contract, args)
        else {
            return false;
        };
        if !self
            .host_memory_span_requirements
            .iter()
            .any(|existing| existing == &requirement)
        {
            self.host_memory_span_requirements.push(requirement);
        }
        true
    }

    pub(super) fn try_record_deferred_direct_host_memory_span_requirement(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        length: &Place,
        direction: HostMemoryDirection,
    ) -> bool {
        let contract = HostMemorySpan::Direct {
            address_arg: 0,
            length: HostMemoryLength::Arg(1),
            unit: HostMemoryDirectUnit::Bytes,
            direction,
        };
        if let Some((base_address, base_length)) =
            direct_symbolic_slice_base_requirement(raw_aliases, address, length)
        {
            return self.try_record_deferred_host_memory_span_requirement(
                raw_aliases,
                &contract,
                &[base_address, base_length],
            );
        }
        self.try_record_deferred_host_memory_span_requirement(
            raw_aliases,
            &contract,
            &[address.clone(), length.clone()],
        )
    }

    pub(super) fn apply_owner_host_memory_span_requirements(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        args: &[Place],
        requirements: &[OwnerHostMemorySpanRequirement],
        span: Span,
    ) -> bool {
        let mut available = true;
        for requirement in requirements {
            let instantiated_args = instantiate_host_memory_requirement_args(args, requirement);
            available &= self.ensure_host_memory_contract_owner_span_available(
                owners,
                raw_aliases,
                &requirement.span,
                &instantiated_args,
                span,
            );
        }
        available
    }
}

fn summarize_host_memory_span_requirement(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    contract: &HostMemorySpan,
    args: &[Place],
) -> Option<OwnerHostMemorySpanRequirement> {
    let referenced = referenced_host_memory_args(contract);
    let mut summarized_args = args
        .iter()
        .map(|arg| OwnerHostMemoryArgSummary::Unknown { ty: arg.ty })
        .collect::<Vec<_>>();
    for (index, role) in referenced {
        let arg = args.get(index)?;
        summarized_args[index] = summarize_host_memory_arg(raw_aliases, params, arg, role)?;
    }
    Some(OwnerHostMemorySpanRequirement {
        span: *contract,
        args: summarized_args,
    })
}

#[derive(Clone, Copy)]
enum HostMemoryArgRole {
    Address,
    Scalar,
}

fn referenced_host_memory_args(contract: &HostMemorySpan) -> Vec<(usize, HostMemoryArgRole)> {
    let mut out = Vec::new();
    match *contract {
        HostMemorySpan::Direct {
            address_arg,
            length,
            ..
        } => {
            push_unique_arg(&mut out, address_arg, HostMemoryArgRole::Address);
            push_host_memory_length_arg(&mut out, length);
        }
        HostMemorySpan::IovDescriptor {
            iovs_arg,
            iov_count_arg,
        }
        | HostMemorySpan::IovPayload {
            iovs_arg,
            iov_count_arg,
            ..
        } => {
            push_unique_arg(&mut out, iovs_arg, HostMemoryArgRole::Address);
            push_unique_arg(&mut out, iov_count_arg, HostMemoryArgRole::Scalar);
        }
    }
    out
}

fn push_host_memory_length_arg(
    out: &mut Vec<(usize, HostMemoryArgRole)>,
    length: HostMemoryLength,
) {
    match length {
        HostMemoryLength::Arg(arg) | HostMemoryLength::ArgScaled { arg, .. } => {
            push_unique_arg(out, arg, HostMemoryArgRole::Scalar);
        }
        HostMemoryLength::ConstI32(_) => {}
    }
}

fn summarize_host_memory_arg(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    arg: &Place,
    role: HostMemoryArgRole,
) -> Option<OwnerHostMemoryArgSummary> {
    if let Some(value) = i32_constant_value(raw_aliases, arg) {
        return Some(OwnerHostMemoryArgSummary::I32Constant { value, ty: arg.ty });
    }
    match role {
        HostMemoryArgRole::Address => {
            parameter_address_source_for_host_memory_arg(raw_aliases, params, arg)
        }
        HostMemoryArgRole::Scalar => {
            parameter_scalar_source_for_host_memory_arg(raw_aliases, params, arg)
        }
    }
    .map(OwnerHostMemoryArgSummary::Parameter)
}

fn direct_symbolic_slice_base_requirement(
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
    length: &Place,
) -> Option<(Place, Place)> {
    let (base_address, offset) = address_without_symbolic_offset(address)?;
    if raw_aliases.i32_condition_truth(&offset, I32ValueCondition::NonNegative) != Some(true) {
        return None;
    }
    for (base_length, subtrahend) in raw_aliases.i32_difference_sources(length) {
        if raw_aliases.canonicalize_scalar(&subtrahend) != raw_aliases.canonicalize_scalar(&offset)
        {
            continue;
        }
        if raw_aliases.i32_relation_truth(&offset, ResourceI32RelationOp::Lt, &base_length)
            != Some(true)
        {
            continue;
        }
        return Some((base_address, base_length));
    }
    None
}

fn address_without_symbolic_offset(address: &Place) -> Option<(Place, Place)> {
    let offset_index = address.projections.iter().position(|projection| {
        matches!(
            projection,
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { .. })
        )
    })?;
    let PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place: offset }) =
        &address.projections[offset_index]
    else {
        return None;
    };
    let mut base = address.clone();
    base.projections.remove(offset_index);
    Some((base, *offset.clone()))
}

fn i32_constant_value(raw_aliases: &RawCellAddressAliases, arg: &Place) -> Option<i32> {
    match arg.root {
        PlaceRoot::I32Constant(value) => Some(value),
        _ => raw_aliases.i32_value(arg),
    }
}

fn parameter_address_source_for_host_memory_arg(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    arg: &Place,
) -> Option<OwnerProjectionSource> {
    let mut candidates = Vec::new();
    push_unique_place(&mut candidates, arg);
    push_unique_place(&mut candidates, &raw_aliases.canonicalize(arg));
    for alias in raw_aliases.aliases_for(arg) {
        push_unique_place(&mut candidates, &alias);
    }
    for alias in raw_aliases.prefix_aliases_for(arg) {
        push_unique_place(&mut candidates, &alias);
    }
    for alias in raw_aliases.scalar_aliases_for_value(arg) {
        push_unique_place(&mut candidates, &alias);
    }
    for candidate in candidates {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(suffix) = place_suffix_after_prefix(&candidate, &param.place) else {
                continue;
            };
            return Some(OwnerProjectionSource {
                parameter_index,
                suffix,
                ty: candidate.ty,
            });
        }
    }
    None
}

fn parameter_scalar_source_for_host_memory_arg(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    arg: &Place,
) -> Option<OwnerProjectionSource> {
    let mut candidates = Vec::new();
    push_scalar_candidate(&mut candidates, arg);
    push_scalar_candidate(&mut candidates, &raw_aliases.canonicalize_scalar(arg));
    for alias in raw_aliases.scalar_aliases_for_value(arg) {
        push_scalar_candidate(&mut candidates, &alias);
    }
    for alias in raw_aliases.aliases_for(arg) {
        push_scalar_candidate(&mut candidates, &alias);
    }
    parameter_source_for_candidates(params, candidates)
}

fn push_scalar_candidate(candidates: &mut Vec<Place>, place: &Place) {
    if !place_has_raw_address_projection(place) {
        push_unique_place(candidates, place);
    }
}

fn parameter_source_for_candidates(
    params: &[ResourceLocal],
    candidates: Vec<Place>,
) -> Option<OwnerProjectionSource> {
    for candidate in candidates {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(suffix) = place_suffix_after_prefix(&candidate, &param.place) else {
                continue;
            };
            return Some(OwnerProjectionSource {
                parameter_index,
                suffix,
                ty: candidate.ty,
            });
        }
    }
    None
}

fn place_has_raw_address_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

fn instantiate_host_memory_requirement_args(
    args: &[Place],
    requirement: &OwnerHostMemorySpanRequirement,
) -> Vec<Place> {
    requirement
        .args
        .iter()
        .map(|arg| instantiate_host_memory_requirement_arg(args, arg))
        .collect()
}

fn instantiate_host_memory_requirement_arg(
    args: &[Place],
    arg: &OwnerHostMemoryArgSummary,
) -> Place {
    match arg {
        OwnerHostMemoryArgSummary::Unknown { ty } => Place::unknown(*ty),
        OwnerHostMemoryArgSummary::Parameter(source) => {
            owner_projection_source_place(args, source).unwrap_or_else(|| Place::unknown(source.ty))
        }
        OwnerHostMemoryArgSummary::I32Constant { value, ty } => Place::i32_constant(*value, *ty),
    }
}

fn push_unique_arg(
    out: &mut Vec<(usize, HostMemoryArgRole)>,
    value: usize,
    role: HostMemoryArgRole,
) {
    if !out.iter().any(|(existing, _)| *existing == value) {
        out.push((value, role));
    }
}

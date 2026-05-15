use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::effect::ResourceEffectBoundaryDiagnostic;
use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_identity::{
    raw_memory_op_produces_identity, RawIdentityTable, RawPointerAliasTable,
};
use super::model::{EffectOp, Place, PlaceRoot};
use super::place_utils::{checked_mem_ptr_wrapper_arg_indices, mem_ptr_raw_field_place};

impl ResourceEffectBoundaryEngine<'_> {
    pub(super) fn report_unproven_checked_mem_ptr_access(
        &mut self,
        identities: &RawIdentityTable,
        pointer_aliases: &RawPointerAliasTable,
        effect: &EffectOp,
        args: &[Place],
        span: Span,
    ) {
        let Some(types) = self.types else {
            return;
        };
        let EffectOp::UnsafeMemory { operation } = effect else {
            return;
        };
        for index in checked_mem_ptr_wrapper_arg_indices(types, *operation, args) {
            let Some(ptr) = args.get(index) else {
                continue;
            };
            let raw = mem_ptr_raw_field_place(ptr, types.i32());
            match checked_mem_ptr_access_proof(identities, pointer_aliases, &raw) {
                CheckedMemPtrAccessProof::InternalAllocationIdentity
                | CheckedMemPtrAccessProof::NullSentinel => {}
                CheckedMemPtrAccessProof::Unproven => {
                    self.diagnostics.push(
                        ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary {
                            function: String::from(self.function),
                            operation: *operation,
                            place: raw,
                            span,
                        },
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckedMemPtrAccessProof {
    InternalAllocationIdentity,
    NullSentinel,
    Unproven,
}

fn checked_mem_ptr_access_proof(
    identities: &RawIdentityTable,
    pointer_aliases: &RawPointerAliasTable,
    raw: &Place,
) -> CheckedMemPtrAccessProof {
    if identities
        .operations(raw)
        .into_iter()
        .any(|operation| raw_memory_op_produces_identity(&operation))
    {
        return CheckedMemPtrAccessProof::InternalAllocationIdentity;
    }
    if checked_mem_ptr_raw_is_null_sentinel(pointer_aliases, raw) {
        return CheckedMemPtrAccessProof::NullSentinel;
    }
    CheckedMemPtrAccessProof::Unproven
}

fn checked_mem_ptr_raw_is_null_sentinel(
    pointer_aliases: &RawPointerAliasTable,
    raw: &Place,
) -> bool {
    raw_and_mem_ptr_view_places(raw).iter().any(|place| {
        pointer_aliases
            .group_for_or_singleton(place)
            .iter()
            .any(|alias| matches!(alias.root, PlaceRoot::I32Constant(value) if value <= 0))
    })
}

fn raw_and_mem_ptr_view_places(raw: &Place) -> Vec<Place> {
    let mut places = Vec::new();
    places.push(raw.clone());
    if !raw.projections.is_empty() {
        let mut ptr = raw.clone();
        ptr.projections.pop();
        places.push(ptr);
    }
    places
}

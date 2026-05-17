extern crate alloc;

use alloc::vec::Vec;

use crate::ast::Effect;
use crate::resource_primitives::{type_is_owner_token, type_is_raw_pointer};
use crate::types::{TypeCtx, TypeId};

use super::compiler_memory_place::{mem_ptr_raw_field_place, region_token_raw_field_place};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{RawCellReleaseParamRequirement, RawCellReleaseRequirementKind};
use super::initialized_summary_release_build::collect_address_release_requirements;
use super::model::{EffectOp, Place, ResourceLocal};

pub(super) fn collect_unknown_indirect_call_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    types: &TypeCtx,
    call_params: &[TypeId],
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for (arg, param_ty) in args.iter().zip(call_params.iter().copied()) {
        let Some(address) = raw_address_release_place_for_call_arg(types, arg, param_ty) else {
            continue;
        };
        collect_address_release_requirements(
            out,
            &address,
            RawCellReleaseRequirementKind::Store,
            raw_aliases,
            params,
        );
    }
}

pub(super) fn indirect_call_may_release_raw_cells(effect: &EffectOp) -> bool {
    !matches!(
        effect,
        EffectOp::Pure
            | EffectOp::UserCall {
                effect: Effect::Pure,
                ..
            }
            | EffectOp::IndirectCall {
                effect: Effect::Pure
            }
    )
}

fn raw_address_release_place_for_call_arg(
    types: &TypeCtx,
    arg: &Place,
    param_ty: TypeId,
) -> Option<Place> {
    if type_is_raw_pointer(types, param_ty) {
        return Some(mem_ptr_raw_field_place(types, arg, types.i32()));
    }
    if type_is_owner_token(types, param_ty) {
        return Some(region_token_raw_field_place(types, arg, types.i32()));
    }
    None
}

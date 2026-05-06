extern crate alloc;

use alloc::vec::Vec;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{RawCellReleaseParamRequirement, RawCellReleaseRequirementKind};
use super::initialized_summary_release_build::collect_address_release_requirements;
use super::model::{EffectOp, Place, PlaceProjection, ResourceLocal};

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
    if is_named_struct_type(types, param_ty, "MemPtr") {
        return Some(mem_ptr_raw_field_place(arg, types.i32()));
    }
    if is_named_struct_type(types, param_ty, "RegionToken") {
        return Some(region_token_raw_field_place(arg, types.i32()));
    }
    None
}

fn mem_ptr_raw_field_place(ptr: &Place, raw_ty: TypeId) -> Place {
    ptr.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        raw_ty,
    )
}

fn region_token_raw_field_place(token: &Place, raw_ty: TypeId) -> Place {
    mem_ptr_raw_field_place(&mem_ptr_raw_field_place(token, token.ty), raw_ty)
}

fn is_named_struct_type(types: &TypeCtx, ty: TypeId, expected: &str) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == expected,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == expected)
        }
        _ => false,
    }
}

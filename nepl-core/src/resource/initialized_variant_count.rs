use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_byte_range_model::RawCellInitializationParamCount;
use super::model::{Place, PlaceProjection};
use super::owner_extent_summary::instantiate_summary_type;
use super::place_utils::projected_place_with_concrete_type;
use super::summary_projection::instantiate_summary_suffix_with_types;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PendingVariantRawByteRangeCount {
    ArgProjection {
        arg: Place,
        suffix: Vec<PlaceProjection>,
        ty: TypeId,
    },
    KnownI32 {
        value: i32,
        ty: TypeId,
    },
}

pub(super) fn pending_variant_count_source(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary_type_params: &[TypeId],
    type_args: &[TypeId],
    count: &RawCellInitializationParamCount,
) -> Option<PendingVariantRawByteRangeCount> {
    match count {
        RawCellInitializationParamCount::ParamProjection {
            param_index,
            suffix,
            ty,
        } => {
            let arg = raw_aliases.canonicalize_scalar(args.get(*param_index)?);
            let ty = instantiate_summary_type(summary_type_params, type_args, *ty);
            let suffix = instantiate_summary_suffix_with_types(types, args, arg.ty, suffix, ty)?;
            Some(PendingVariantRawByteRangeCount::ArgProjection { arg, suffix, ty })
        }
        RawCellInitializationParamCount::KnownI32 { value, ty } => {
            Some(PendingVariantRawByteRangeCount::KnownI32 {
                value: *value,
                ty: instantiate_summary_type(summary_type_params, type_args, *ty),
            })
        }
    }
}

pub(super) fn pending_variant_count_place(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    count: &PendingVariantRawByteRangeCount,
) -> Place {
    match count {
        PendingVariantRawByteRangeCount::ArgProjection { arg, suffix, ty } => {
            let arg = raw_aliases.canonicalize_scalar(arg);
            projected_place_with_concrete_type(types, &arg, suffix, *ty)
        }
        PendingVariantRawByteRangeCount::KnownI32 { value, ty } => Place::i32_constant(*value, *ty),
    }
}

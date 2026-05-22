extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::initialized::ResourceCheckEngine;
use super::model::{Place, PlaceProjection, ResourceLocal, ResourceOffset};
use super::place_utils::{place_suffix_after_prefix, projection_result_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SummaryPlace {
    pub(super) parameter_index: usize,
    pub(super) suffix: Vec<SummaryProjection>,
    pub(super) ty: crate::types::TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SummaryProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(SummaryOffset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SummaryOffset {
    Known(usize),
    Symbolic {
        place: Box<SummaryPlace>,
    },
    ScaledSymbolic {
        place: Box<SummaryPlace>,
        scale: usize,
    },
    Unknown,
}

pub(super) fn instantiate_summary_place(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_place_with_types(engine.types, args, target)
}

pub(super) fn instantiate_summary_place_with_types(
    types: &crate::types::TypeCtx,
    args: &[Place],
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_suffix_on_base_with_types(
        types,
        args,
        args.get(target.parameter_index)?,
        &target.suffix,
        target.ty,
    )
}

pub(super) fn instantiate_summary_suffix_on_base(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    base: &Place,
    suffix: &[SummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Place> {
    instantiate_summary_suffix_on_base_with_types(engine.types, args, base, suffix, fallback_ty)
}

pub(super) fn instantiate_summary_suffix_on_base_with_types(
    types: &crate::types::TypeCtx,
    args: &[Place],
    base: &Place,
    suffix: &[SummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Place> {
    let mut out = base.clone();
    let concrete_suffix =
        instantiate_summary_suffix_with_types(types, args, out.ty, suffix, fallback_ty)?;
    for projection in concrete_suffix {
        let current_ty = projection_result_type(types, out.ty, &projection).unwrap_or(fallback_ty);
        out.projections.push(projection);
        out.ty = current_ty;
    }
    Some(out)
}

pub(super) fn instantiate_summary_suffix_with_types(
    types: &crate::types::TypeCtx,
    args: &[Place],
    base_ty: crate::types::TypeId,
    suffix: &[SummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    for projection in suffix {
        let projection = instantiate_summary_projection(types, args, projection)?;
        current_ty = projection_result_type(types, current_ty, &projection).unwrap_or(fallback_ty);
        out.push(projection);
    }
    Some(out)
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<SummaryPlace> {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(target, &param.place) else {
            continue;
        };
        return Some(SummaryPlace {
            parameter_index,
            suffix: summary_suffix_for_params(params, &suffix)?,
            ty: target.ty,
        });
    }
    None
}

pub(super) fn summary_suffix_for_params(
    params: &[ResourceLocal],
    suffix: &[PlaceProjection],
) -> Option<Vec<SummaryProjection>> {
    let mut out = Vec::new();
    for projection in suffix {
        out.push(summary_projection_for_params(params, projection)?);
    }
    Some(out)
}

pub(super) fn translate_summary_suffix_for_params(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    suffix: &[SummaryProjection],
) -> Option<Vec<SummaryProjection>> {
    let mut out = Vec::new();
    for projection in suffix {
        let projection = instantiate_summary_projection(engine.types, args, projection)?;
        out.push(summary_projection_for_params(params, &projection)?);
    }
    Some(out)
}

pub(super) fn compose_translated_summary_suffix_for_params(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    prefix: &[PlaceProjection],
    suffix: &[SummaryProjection],
) -> Option<Vec<SummaryProjection>> {
    let mut out = summary_suffix_for_params(params, prefix)?;
    let mut translated_suffix = translate_summary_suffix_for_params(engine, args, params, suffix)?;
    out.append(&mut translated_suffix);
    Some(out)
}

fn summary_projection_for_params(
    params: &[ResourceLocal],
    projection: &PlaceProjection,
) -> Option<SummaryProjection> {
    Some(match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => SummaryProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => SummaryProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::EnumPayload { variant } => SummaryProjection::EnumPayload {
            variant: variant.clone(),
        },
        PlaceProjection::Deref => SummaryProjection::Deref,
        PlaceProjection::StorageOffset(offset) => {
            SummaryProjection::StorageOffset(summary_offset_for_params(params, offset)?)
        }
    })
}

fn summary_offset_for_params(
    params: &[ResourceLocal],
    offset: &ResourceOffset,
) -> Option<SummaryOffset> {
    Some(match offset {
        ResourceOffset::Known(value) => SummaryOffset::Known(*value),
        ResourceOffset::Symbolic { place } => SummaryOffset::Symbolic {
            place: Box::new(summary_place_for_params(params, place)?),
        },
        ResourceOffset::ScaledSymbolic { place, scale } => SummaryOffset::ScaledSymbolic {
            place: Box::new(summary_place_for_params(params, place)?),
            scale: *scale,
        },
        ResourceOffset::Unknown => SummaryOffset::Unknown,
    })
}

fn instantiate_summary_projection(
    types: &crate::types::TypeCtx,
    args: &[Place],
    projection: &SummaryProjection,
) -> Option<PlaceProjection> {
    Some(match projection {
        SummaryProjection::Field {
            index,
            offset_bytes,
        } => PlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::TupleField {
            index,
            offset_bytes,
        } => PlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::EnumPayload { variant } => PlaceProjection::EnumPayload {
            variant: variant.clone(),
        },
        SummaryProjection::Deref => PlaceProjection::Deref,
        SummaryProjection::StorageOffset(offset) => {
            PlaceProjection::StorageOffset(instantiate_summary_offset(types, args, offset)?)
        }
    })
}

fn instantiate_summary_offset(
    types: &crate::types::TypeCtx,
    args: &[Place],
    offset: &SummaryOffset,
) -> Option<ResourceOffset> {
    Some(match offset {
        SummaryOffset::Known(value) => ResourceOffset::Known(*value),
        SummaryOffset::Symbolic { place } => ResourceOffset::Symbolic {
            place: Box::new(instantiate_summary_place_with_types(types, args, place)?),
        },
        SummaryOffset::ScaledSymbolic { place, scale } => ResourceOffset::ScaledSymbolic {
            place: Box::new(instantiate_summary_place_with_types(types, args, place)?),
            scale: *scale,
        },
        SummaryOffset::Unknown => ResourceOffset::Unknown,
    })
}

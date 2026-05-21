extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, PlaceProjection, ResourceLocal, ResourceOffset};
use super::place_utils::{place_suffix_after_prefix, projection_result_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(CollectionSlotLifecycleSummaryOffset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryOffset {
    Known(usize),
    Symbolic {
        place: Box<CollectionSlotLifecycleSummaryPlace>,
    },
    ScaledSymbolic {
        place: Box<CollectionSlotLifecycleSummaryPlace>,
        scale: usize,
    },
    Unknown,
}

pub(super) fn instantiate_summary_place(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    instantiate_summary_suffix_on_base(
        engine,
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
    suffix: &[CollectionSlotLifecycleSummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Place> {
    let mut out = base.clone();
    let mut current_ty = out.ty;
    for projection in suffix {
        let projection = instantiate_summary_projection(engine, args, projection)?;
        current_ty =
            projection_result_type(engine.types, current_ty, &projection).unwrap_or(fallback_ty);
        out.projections.push(projection);
        out.ty = current_ty;
    }
    Some(out)
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(target, &param.place) else {
            continue;
        };
        return Some(CollectionSlotLifecycleSummaryPlace {
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
) -> Option<Vec<CollectionSlotLifecycleSummaryProjection>> {
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
    suffix: &[CollectionSlotLifecycleSummaryProjection],
) -> Option<Vec<CollectionSlotLifecycleSummaryProjection>> {
    let mut out = Vec::new();
    for projection in suffix {
        let projection = instantiate_summary_projection(engine, args, projection)?;
        out.push(summary_projection_for_params(params, &projection)?);
    }
    Some(out)
}

pub(super) fn compose_translated_summary_suffix_for_params(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    prefix: &[PlaceProjection],
    suffix: &[CollectionSlotLifecycleSummaryProjection],
) -> Option<Vec<CollectionSlotLifecycleSummaryProjection>> {
    let mut out = summary_suffix_for_params(params, prefix)?;
    let mut translated_suffix = translate_summary_suffix_for_params(engine, args, params, suffix)?;
    out.append(&mut translated_suffix);
    Some(out)
}

fn summary_projection_for_params(
    params: &[ResourceLocal],
    projection: &PlaceProjection,
) -> Option<CollectionSlotLifecycleSummaryProjection> {
    Some(match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => CollectionSlotLifecycleSummaryProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => CollectionSlotLifecycleSummaryProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::EnumPayload { variant } => {
            CollectionSlotLifecycleSummaryProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        PlaceProjection::Deref => CollectionSlotLifecycleSummaryProjection::Deref,
        PlaceProjection::StorageOffset(offset) => {
            CollectionSlotLifecycleSummaryProjection::StorageOffset(summary_offset_for_params(
                params, offset,
            )?)
        }
    })
}

fn summary_offset_for_params(
    params: &[ResourceLocal],
    offset: &ResourceOffset,
) -> Option<CollectionSlotLifecycleSummaryOffset> {
    Some(match offset {
        ResourceOffset::Known(value) => CollectionSlotLifecycleSummaryOffset::Known(*value),
        ResourceOffset::Symbolic { place } => CollectionSlotLifecycleSummaryOffset::Symbolic {
            place: Box::new(summary_place_for_params(params, place)?),
        },
        ResourceOffset::ScaledSymbolic { place, scale } => {
            CollectionSlotLifecycleSummaryOffset::ScaledSymbolic {
                place: Box::new(summary_place_for_params(params, place)?),
                scale: *scale,
            }
        }
        ResourceOffset::Unknown => CollectionSlotLifecycleSummaryOffset::Unknown,
    })
}

fn instantiate_summary_projection(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    projection: &CollectionSlotLifecycleSummaryProjection,
) -> Option<PlaceProjection> {
    Some(match projection {
        CollectionSlotLifecycleSummaryProjection::Field {
            index,
            offset_bytes,
        } => PlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        CollectionSlotLifecycleSummaryProjection::TupleField {
            index,
            offset_bytes,
        } => PlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        CollectionSlotLifecycleSummaryProjection::EnumPayload { variant } => {
            PlaceProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        CollectionSlotLifecycleSummaryProjection::Deref => PlaceProjection::Deref,
        CollectionSlotLifecycleSummaryProjection::StorageOffset(offset) => {
            PlaceProjection::StorageOffset(instantiate_summary_offset(engine, args, offset)?)
        }
    })
}

fn instantiate_summary_offset(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    offset: &CollectionSlotLifecycleSummaryOffset,
) -> Option<ResourceOffset> {
    Some(match offset {
        CollectionSlotLifecycleSummaryOffset::Known(value) => ResourceOffset::Known(*value),
        CollectionSlotLifecycleSummaryOffset::Symbolic { place } => ResourceOffset::Symbolic {
            place: Box::new(instantiate_summary_place(engine, args, place)?),
        },
        CollectionSlotLifecycleSummaryOffset::ScaledSymbolic { place, scale } => {
            ResourceOffset::ScaledSymbolic {
                place: Box::new(instantiate_summary_place(engine, args, place)?),
                scale: *scale,
            }
        }
        CollectionSlotLifecycleSummaryOffset::Unknown => ResourceOffset::Unknown,
    })
}

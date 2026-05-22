extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::types::TypeCtx;

use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
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
    Offset {
        place: Box<SummaryPlace>,
        offset: i64,
    },
    ScaledOffset {
        place: Box<SummaryPlace>,
        offset: i64,
        scale: usize,
    },
    Unknown,
}

#[cfg(test)]
pub(super) fn instantiate_summary_place(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_place_with_types(engine.types, args, target)
}

pub(super) fn instantiate_summary_place_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_place_with_types_and_aliases(engine.types, args, Some(raw_aliases), target)
}

#[cfg(test)]
pub(super) fn instantiate_summary_place_with_types(
    types: &crate::types::TypeCtx,
    args: &[Place],
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_place_with_types_and_aliases(types, args, None, target)
}

fn instantiate_summary_place_with_types_and_aliases(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &SummaryPlace,
) -> Option<Place> {
    instantiate_summary_suffix_on_base_with_types(
        types,
        args,
        raw_aliases,
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
    instantiate_summary_suffix_on_base_with_types(
        engine.types,
        args,
        None,
        base,
        suffix,
        fallback_ty,
    )
}

pub(super) fn instantiate_summary_suffix_on_base_with_types(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
    base: &Place,
    suffix: &[SummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Place> {
    let mut out = base.clone();
    let concrete_suffix = instantiate_summary_suffix_with_types_and_aliases(
        types,
        args,
        raw_aliases,
        out.ty,
        suffix,
        fallback_ty,
    )?;
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
    instantiate_summary_suffix_with_types_and_aliases(
        types,
        args,
        None,
        base_ty,
        suffix,
        fallback_ty,
    )
}

fn instantiate_summary_suffix_with_types_and_aliases(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
    base_ty: crate::types::TypeId,
    suffix: &[SummaryProjection],
    fallback_ty: crate::types::TypeId,
) -> Option<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    for projection in suffix {
        let projection = instantiate_summary_projection(types, args, raw_aliases, projection)?;
        current_ty = projection_result_type(types, current_ty, &projection).unwrap_or(fallback_ty);
        out.push(projection);
    }
    Some(out)
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<SummaryPlace> {
    summary_place_for_params_with_offset_aliases(params, None, None, target)
}

pub(super) fn summary_place_for_params_with_scalar_aliases(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
) -> Option<SummaryPlace> {
    summary_place_for_params_with_offset_aliases(params, None, Some(raw_aliases), target)
}

pub(super) fn summary_place_for_params_with_scalar_aliases_and_types(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
) -> Option<SummaryPlace> {
    summary_place_for_params_with_offset_aliases(params, Some(types), Some(raw_aliases), target)
}

fn summary_place_for_params_with_offset_aliases(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
) -> Option<SummaryPlace> {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(suffix) = place_suffix_after_prefix(target, &param.place) else {
            continue;
        };
        return Some(SummaryPlace {
            parameter_index,
            suffix: summary_suffix_for_params_with_offset_aliases(
                params,
                types,
                raw_aliases,
                &suffix,
            )?,
            ty: target.ty,
        });
    }
    None
}

pub(super) fn summary_suffix_for_params(
    params: &[ResourceLocal],
    suffix: &[PlaceProjection],
) -> Option<Vec<SummaryProjection>> {
    summary_suffix_for_params_with_offset_aliases(params, None, None, suffix)
}

fn summary_suffix_for_params_with_offset_aliases(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: Option<&RawCellAddressAliases>,
    suffix: &[PlaceProjection],
) -> Option<Vec<SummaryProjection>> {
    let mut out = Vec::new();
    for projection in suffix {
        out.push(summary_projection_for_params(
            params,
            types,
            raw_aliases,
            projection,
        )?);
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
        let projection = instantiate_summary_projection(engine.types, args, None, projection)?;
        out.push(summary_projection_for_params(
            params,
            None,
            None,
            &projection,
        )?);
    }
    Some(out)
}

pub(super) fn translate_summary_place_for_params_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    target: &SummaryPlace,
) -> Option<SummaryPlace> {
    let base = args.get(target.parameter_index)?;
    let mut out = summary_place_for_params_with_offset_aliases(
        params,
        Some(engine.types),
        Some(raw_aliases),
        base,
    )?;
    let mut suffix = translate_summary_suffix_for_params_with_aliases(
        engine,
        args,
        params,
        raw_aliases,
        &target.suffix,
    )?;
    out.suffix.append(&mut suffix);
    out.ty = target.ty;
    Some(out)
}

fn translate_summary_suffix_for_params_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    suffix: &[SummaryProjection],
) -> Option<Vec<SummaryProjection>> {
    let mut out = Vec::new();
    for projection in suffix {
        out.push(translate_summary_projection_for_params_with_aliases(
            engine,
            args,
            params,
            raw_aliases,
            projection,
        )?);
    }
    Some(out)
}

fn translate_summary_projection_for_params_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    projection: &SummaryProjection,
) -> Option<SummaryProjection> {
    Some(match projection {
        SummaryProjection::Field {
            index,
            offset_bytes,
        } => SummaryProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::TupleField {
            index,
            offset_bytes,
        } => SummaryProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::EnumPayload { variant } => SummaryProjection::EnumPayload {
            variant: variant.clone(),
        },
        SummaryProjection::Deref => SummaryProjection::Deref,
        SummaryProjection::StorageOffset(offset) => {
            SummaryProjection::StorageOffset(translate_summary_offset_for_params_with_aliases(
                engine,
                args,
                params,
                raw_aliases,
                offset,
            )?)
        }
    })
}

fn translate_summary_offset_for_params_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    offset: &SummaryOffset,
) -> Option<SummaryOffset> {
    match offset {
        SummaryOffset::Known(value) => Some(SummaryOffset::Known(*value)),
        SummaryOffset::Symbolic { place } => translate_summary_offset_place_with_shift(
            engine,
            args,
            params,
            raw_aliases,
            place,
            1,
            0,
        ),
        SummaryOffset::ScaledSymbolic { place, scale } => {
            translate_summary_offset_place_with_shift(
                engine,
                args,
                params,
                raw_aliases,
                place,
                *scale,
                0,
            )
        }
        SummaryOffset::Offset { place, offset } => translate_summary_offset_place_with_shift(
            engine,
            args,
            params,
            raw_aliases,
            place,
            1,
            *offset,
        ),
        SummaryOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => translate_summary_offset_place_with_shift(
            engine,
            args,
            params,
            raw_aliases,
            place,
            *scale,
            *offset,
        ),
        SummaryOffset::Unknown => Some(SummaryOffset::Unknown),
    }
}

fn translate_summary_offset_place_with_shift(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &SummaryPlace,
    scale: usize,
    offset: i64,
) -> Option<SummaryOffset> {
    if let Some(translated) =
        translate_summary_place_for_params_with_aliases(engine, args, params, raw_aliases, place)
    {
        return Some(summary_offset_from_place(translated, scale, offset));
    }
    let actual = instantiate_summary_place_with_aliases(engine, args, raw_aliases, place)?;
    summary_offset_for_scalar_with_shift(
        params,
        Some(engine.types),
        raw_aliases,
        &actual,
        scale,
        offset,
    )
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
    types: Option<&TypeCtx>,
    raw_aliases: Option<&RawCellAddressAliases>,
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
        PlaceProjection::StorageOffset(offset) => SummaryProjection::StorageOffset(
            summary_offset_for_params(params, types, raw_aliases, offset)?,
        ),
    })
}

fn summary_offset_for_params(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: Option<&RawCellAddressAliases>,
    offset: &ResourceOffset,
) -> Option<SummaryOffset> {
    Some(match offset {
        ResourceOffset::Known(value) => SummaryOffset::Known(*value),
        ResourceOffset::Symbolic { place } => {
            summary_offset_for_scalar(params, types, raw_aliases, place, 1)?
        }
        ResourceOffset::ScaledSymbolic { place, scale } => {
            summary_offset_for_scalar(params, types, raw_aliases, place, *scale)?
        }
        ResourceOffset::Offset { place, offset } => {
            let raw_aliases = raw_aliases?;
            summary_offset_for_scalar_with_shift(params, types, raw_aliases, place, 1, *offset)?
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => {
            let raw_aliases = raw_aliases?;
            summary_offset_for_scalar_with_shift(
                params,
                types,
                raw_aliases,
                place,
                *scale,
                *offset,
            )?
        }
        ResourceOffset::Unknown => SummaryOffset::Unknown,
    })
}

fn summary_offset_for_scalar(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: Option<&RawCellAddressAliases>,
    place: &Place,
    scale: usize,
) -> Option<SummaryOffset> {
    if let Some(summary) = summary_offset_operand_for_params(params, raw_aliases, place) {
        return Some(summary_offset_from_place(summary, scale, 0));
    }
    let raw_aliases = raw_aliases?;
    summary_offset_for_scalar_with_shift(params, types, raw_aliases, place, scale, 0)
}

fn summary_offset_for_scalar_with_shift(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    scale: usize,
    offset: i64,
) -> Option<SummaryOffset> {
    let mut visited = Vec::new();
    summary_offset_for_scalar_with_shift_inner(
        params,
        types,
        raw_aliases,
        place,
        scale,
        offset,
        &mut visited,
    )
}

fn summary_offset_for_scalar_with_shift_inner(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    scale: usize,
    offset: i64,
    visited: &mut Vec<Place>,
) -> Option<SummaryOffset> {
    if visited.iter().any(|seen| seen == place) {
        return None;
    }
    visited.push(place.clone());
    let result = (|| {
        if let Some(summary) = summary_offset_operand_for_params(params, Some(raw_aliases), place) {
            return Some(summary_offset_from_place(summary, scale, offset));
        };
        if let Some(value) = raw_aliases.i32_value(place) {
            return known_summary_offset_from_i64(i64::from(value).checked_add(offset)?, scale);
        }
        if let Some((source, source_scale)) = raw_aliases.i32_scaled_source(place) {
            let scale = scale.checked_mul(source_scale)?;
            if let Some(summary) = summary_offset_for_scalar_with_shift_inner(
                params,
                types,
                raw_aliases,
                &source,
                scale,
                offset,
                visited,
            ) {
                return Some(summary);
            }
        }
        if let (Some(types), Some((source, element_ty))) =
            (types, raw_aliases.i32_type_size_scaled_source(place))
        {
            let scale = scale.checked_mul(storage_size_bytes(types, element_ty))?;
            if let Some(summary) = summary_offset_for_scalar_with_shift_inner(
                params,
                Some(types),
                raw_aliases,
                &source,
                scale,
                offset,
                visited,
            ) {
                return Some(summary);
            }
        }
        for (source, source_offset) in raw_aliases.i32_offset_sources(place) {
            let combined_offset = source_offset.checked_add(offset)?;
            if let Some(summary) = summary_offset_for_scalar_with_shift_inner(
                params,
                types,
                raw_aliases,
                &source,
                scale,
                combined_offset,
                visited,
            ) {
                return Some(summary);
            }
        }
        for (minuend, subtrahend) in raw_aliases.i32_difference_sources(place) {
            let Some(subtrahend_value) = raw_aliases.i32_value(&subtrahend) else {
                continue;
            };
            let Some(combined_offset) = offset.checked_sub(i64::from(subtrahend_value)) else {
                continue;
            };
            if let Some(summary) = summary_offset_for_scalar_with_shift_inner(
                params,
                types,
                raw_aliases,
                &minuend,
                scale,
                combined_offset,
                visited,
            ) {
                return Some(summary);
            }
        }
        None
    })();
    visited.pop();
    result
}

fn summary_offset_operand_for_params(
    params: &[ResourceLocal],
    raw_aliases: Option<&RawCellAddressAliases>,
    place: &Place,
) -> Option<SummaryPlace> {
    if let Some(summary) = summary_place_for_params(params, place) {
        return Some(summary);
    }
    let Some(raw_aliases) = raw_aliases else {
        return None;
    };
    raw_aliases
        .scalar_aliases_for_value(place)
        .into_iter()
        .find_map(|alias| summary_place_for_params(params, &alias))
}

fn summary_offset_from_place(place: SummaryPlace, scale: usize, offset: i64) -> SummaryOffset {
    match (scale, offset) {
        (1, 0) => SummaryOffset::Symbolic {
            place: Box::new(place),
        },
        (_, 0) => SummaryOffset::ScaledSymbolic {
            place: Box::new(place),
            scale,
        },
        (1, _) => SummaryOffset::Offset {
            place: Box::new(place),
            offset,
        },
        _ => SummaryOffset::ScaledOffset {
            place: Box::new(place),
            offset,
            scale,
        },
    }
}

fn known_summary_offset_from_i64(value: i64, scale: usize) -> Option<SummaryOffset> {
    let value = value.checked_mul(i64::try_from(scale).ok()?)?;
    let value = usize::try_from(value).ok()?;
    Some(SummaryOffset::Known(value))
}

fn instantiate_summary_projection(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
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
        SummaryProjection::StorageOffset(offset) => PlaceProjection::StorageOffset(
            instantiate_summary_offset(types, args, raw_aliases, offset)?,
        ),
    })
}

fn instantiate_summary_offset(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
    offset: &SummaryOffset,
) -> Option<ResourceOffset> {
    Some(match offset {
        SummaryOffset::Known(value) => ResourceOffset::Known(*value),
        SummaryOffset::Symbolic { place } => {
            instantiate_symbolic_summary_offset(types, args, raw_aliases, place, 1, 0)?
        }
        SummaryOffset::ScaledSymbolic { place, scale } => {
            instantiate_symbolic_summary_offset(types, args, raw_aliases, place, *scale, 0)?
        }
        SummaryOffset::Offset { place, offset } => {
            instantiate_symbolic_summary_offset(types, args, raw_aliases, place, 1, *offset)?
        }
        SummaryOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => instantiate_symbolic_summary_offset(types, args, raw_aliases, place, *scale, *offset)?,
        SummaryOffset::Unknown => ResourceOffset::Unknown,
    })
}

fn instantiate_symbolic_summary_offset(
    types: &crate::types::TypeCtx,
    args: &[Place],
    raw_aliases: Option<&RawCellAddressAliases>,
    place: &SummaryPlace,
    scale: usize,
    offset: i64,
) -> Option<ResourceOffset> {
    let actual = instantiate_summary_place_with_types_and_aliases(types, args, raw_aliases, place)?;
    if let Some(raw_aliases) = raw_aliases {
        if let Some(resolved) =
            resource_offset_for_scalar_with_shift(types, raw_aliases, &actual, scale, offset)
        {
            return Some(resolved);
        }
    } else if offset != 0 {
        return Some(resource_offset_from_symbolic_shift(actual, scale, offset));
    }
    Some(resource_offset_from_symbolic(actual, scale))
}

fn resource_offset_for_scalar_with_shift(
    types: &crate::types::TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    scale: usize,
    offset: i64,
) -> Option<ResourceOffset> {
    let mut visited = Vec::new();
    resource_offset_for_scalar_with_shift_inner(
        types,
        raw_aliases,
        place,
        scale,
        offset,
        &mut visited,
    )
}

fn resource_offset_for_scalar_with_shift_inner(
    types: &crate::types::TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    scale: usize,
    offset: i64,
    visited: &mut Vec<Place>,
) -> Option<ResourceOffset> {
    if visited.iter().any(|seen| seen == place) {
        return None;
    }
    visited.push(place.clone());
    let result = (|| {
        if let Some(value) = raw_aliases.i32_value(place) {
            return resource_offset_from_shifted_scaled_i32(value, offset, scale);
        }
        if let Some((source, source_scale)) = raw_aliases.i32_scaled_source(place) {
            let scale = scale.checked_mul(source_scale)?;
            if let Some(resolved) = resource_offset_for_scalar_with_shift_inner(
                types,
                raw_aliases,
                &source,
                scale,
                offset,
                visited,
            ) {
                return Some(resolved);
            }
        }
        if let Some((source, element_ty)) = raw_aliases.i32_type_size_scaled_source(place) {
            let scale = scale.checked_mul(storage_size_bytes(types, element_ty))?;
            if let Some(resolved) = resource_offset_for_scalar_with_shift_inner(
                types,
                raw_aliases,
                &source,
                scale,
                offset,
                visited,
            ) {
                return Some(resolved);
            }
        }
        for (source, source_offset) in raw_aliases.i32_offset_sources(place) {
            let combined_offset = source_offset.checked_add(offset)?;
            if let Some(resolved) = resource_offset_for_scalar_with_shift_inner(
                types,
                raw_aliases,
                &source,
                scale,
                combined_offset,
                visited,
            ) {
                return Some(resolved);
            }
        }
        for (minuend, subtrahend) in raw_aliases.i32_difference_sources(place) {
            let Some(subtrahend_value) = raw_aliases.i32_value(&subtrahend) else {
                continue;
            };
            let Some(combined_offset) = offset.checked_sub(i64::from(subtrahend_value)) else {
                continue;
            };
            if let Some(resolved) = resource_offset_for_scalar_with_shift_inner(
                types,
                raw_aliases,
                &minuend,
                scale,
                combined_offset,
                visited,
            ) {
                return Some(resolved);
            }
        }
        Some(resource_offset_from_symbolic_shift(
            raw_aliases.canonicalize_scalar(place),
            scale,
            offset,
        ))
    })();
    visited.pop();
    result
}

fn resource_offset_from_symbolic(place: Place, scale: usize) -> ResourceOffset {
    if scale == 1 {
        ResourceOffset::Symbolic {
            place: Box::new(place),
        }
    } else {
        ResourceOffset::ScaledSymbolic {
            place: Box::new(place),
            scale,
        }
    }
}

fn resource_offset_from_symbolic_shift(place: Place, scale: usize, offset: i64) -> ResourceOffset {
    match (scale, offset) {
        (1, 0) => ResourceOffset::Symbolic {
            place: Box::new(place),
        },
        (_, 0) => ResourceOffset::ScaledSymbolic {
            place: Box::new(place),
            scale,
        },
        (1, _) => ResourceOffset::Offset {
            place: Box::new(place),
            offset,
        },
        _ => ResourceOffset::ScaledOffset {
            place: Box::new(place),
            offset,
            scale,
        },
    }
}

fn resource_offset_from_shifted_scaled_i32(
    value: i32,
    offset: i64,
    scale: usize,
) -> Option<ResourceOffset> {
    let shifted = i64::from(value).checked_add(offset)?;
    let shifted = usize::try_from(shifted).ok()?;
    shifted.checked_mul(scale).map(ResourceOffset::Known)
}

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceI32RelationOp};

pub(super) fn scalar_place_covers_count(
    raw_aliases: &RawCellAddressAliases,
    available: &Place,
    required: &Place,
) -> bool {
    let available = raw_aliases.canonicalize_scalar(available);
    let required = raw_aliases.canonicalize_scalar(required);
    if available == required {
        return true;
    }
    if let (Some(available), Some(required)) = (
        raw_aliases.i32_value(&available),
        raw_aliases.i32_value(&required),
    ) {
        return available >= required;
    }
    raw_aliases.i32_relation_truth(&available, ResourceI32RelationOp::Ge, &required) == Some(true)
        || raw_aliases.i32_relation_truth(&available, ResourceI32RelationOp::Gt, &required)
            == Some(true)
}

pub(super) fn place_covers_scaled_count(
    raw_aliases: &RawCellAddressAliases,
    available: &Place,
    required_count: &Place,
    stride: usize,
) -> bool {
    if stride == 1 {
        return scalar_place_covers_count(raw_aliases, available, required_count);
    }
    let required_count = raw_aliases.canonicalize_scalar(required_count);
    if raw_aliases
        .i32_scaled_source(available)
        .is_some_and(|(source, scale)| source == required_count && scale == stride)
    {
        return true;
    }
    raw_aliases
        .i32_scaled_targets(&required_count, stride)
        .iter()
        .any(|required_bytes| scalar_place_covers_count(raw_aliases, available, required_bytes))
        || scaled_count_value_is_covered_by_place(raw_aliases, &required_count, stride, available)
}

pub(super) fn copied_element_count_from_byte_count(
    raw_aliases: &RawCellAddressAliases,
    count: &Place,
    stride: usize,
) -> Option<Place> {
    if stride == 1 {
        return Some(raw_aliases.canonicalize_scalar(count));
    }
    if let Some((source, scale)) = raw_aliases.i32_scaled_source(count) {
        if scale == stride {
            return Some(source);
        }
    }
    let bytes = raw_aliases.i32_value(count)?;
    let bytes = usize::try_from(bytes).ok()?;
    if bytes % stride != 0 {
        return None;
    }
    let elements = bytes / stride;
    let elements = i32::try_from(elements).ok()?;
    Some(Place::i32_constant(elements, count.ty))
}

fn scaled_count_value_is_covered_by_place(
    raw_aliases: &RawCellAddressAliases,
    required_count: &Place,
    stride: usize,
    available: &Place,
) -> bool {
    let (Some(required_count), Some(available)) = (
        raw_aliases.i32_value(required_count),
        raw_aliases.i32_value(available),
    ) else {
        return false;
    };
    let (Ok(required_count), Ok(available)) =
        (usize::try_from(required_count), usize::try_from(available))
    else {
        return false;
    };
    required_count
        .checked_mul(stride)
        .is_some_and(|required_bytes| available >= required_bytes)
}

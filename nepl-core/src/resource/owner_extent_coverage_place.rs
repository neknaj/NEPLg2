use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceI32RelationOp};
use super::owner_extent::OwnerExtentProof;

pub(super) fn prove_scalar_place_covers(
    raw_aliases: &RawCellAddressAliases,
    available: &Place,
    required: &Place,
) -> OwnerExtentProof {
    let available = raw_aliases.canonicalize_scalar(available);
    let required = raw_aliases.canonicalize_scalar(required);
    if available == required {
        return OwnerExtentProof::Proven;
    }
    match (
        raw_aliases.i32_value(&available),
        raw_aliases.i32_value(&required),
    ) {
        (Some(left), Some(right)) if left >= right => return OwnerExtentProof::Proven,
        (Some(_), Some(_)) => return OwnerExtentProof::Mismatch,
        _ => {}
    }
    if raw_aliases.i32_relation_truth(&available, ResourceI32RelationOp::Ge, &required)
        == Some(true)
        || raw_aliases.i32_relation_truth(&available, ResourceI32RelationOp::Gt, &required)
            == Some(true)
    {
        return OwnerExtentProof::Proven;
    }
    if raw_aliases.i32_relation_truth(&available, ResourceI32RelationOp::Lt, &required)
        == Some(true)
    {
        return OwnerExtentProof::Mismatch;
    }
    OwnerExtentProof::Unknown
}

pub(super) fn prove_scaled_place_covers(
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    scale: usize,
    required: &Place,
) -> OwnerExtentProof {
    if scale == 1 {
        return prove_scalar_place_covers(raw_aliases, source, required);
    }
    if let Some((required_source, required_scale)) = raw_aliases.i32_scaled_source(required) {
        if required_scale == scale && raw_aliases.canonicalize_scalar(source) == required_source {
            return OwnerExtentProof::Proven;
        }
    }
    match (
        raw_aliases.i32_value(source),
        raw_aliases.i32_value(required),
    ) {
        (Some(source), Some(required)) => {
            let Some(scale) = i32::try_from(scale).ok() else {
                return OwnerExtentProof::Mismatch;
            };
            match source.checked_mul(scale) {
                Some(available) if available >= required => OwnerExtentProof::Proven,
                Some(_) | None => OwnerExtentProof::Mismatch,
            }
        }
        _ => OwnerExtentProof::Unknown,
    }
}

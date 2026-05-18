use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place, ResourceI32RelationOp};
use super::owner_extent::OwnerExtentProof;

pub(super) fn prove_owner_extent_covers_argument(
    raw_aliases: &RawCellAddressAliases,
    extent: &OwnerStorageExtent,
    required: &Place,
) -> OwnerExtentProof {
    match extent {
        OwnerStorageExtent::Unknown | OwnerStorageExtent::RegionTokenSize => {
            OwnerExtentProof::Unknown
        }
        OwnerStorageExtent::PayloadBytes { bytes } => {
            prove_scalar_place_covers(raw_aliases, bytes, required)
        }
    }
}

fn prove_scalar_place_covers(
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

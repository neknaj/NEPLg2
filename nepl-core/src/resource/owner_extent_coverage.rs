use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place};
use super::owner_extent::OwnerExtentProof;
use super::owner_extent_coverage_place::{prove_scalar_place_covers, prove_scaled_place_covers};

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
        OwnerStorageExtent::PayloadBytesScaled { source, scale } => {
            prove_scaled_place_covers(raw_aliases, source, *scale, required)
        }
    }
}

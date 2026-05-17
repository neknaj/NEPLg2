use super::compiler_memory_place::region_token_size_field_for_raw_owner;
use super::model::{OwnerStorageExtent, Place};

pub(super) fn comparable_owner_extent(
    owner: &Place,
    extent: OwnerStorageExtent,
) -> OwnerStorageExtent {
    match extent {
        OwnerStorageExtent::RegionTokenSize => region_token_size_for_raw_owner(owner)
            .map(|size| OwnerStorageExtent::payload_bytes(&size))
            .unwrap_or(OwnerStorageExtent::Unknown),
        other => other,
    }
}

pub(super) fn region_token_size_for_raw_owner(owner: &Place) -> Option<Place> {
    region_token_size_field_for_raw_owner(owner)
}

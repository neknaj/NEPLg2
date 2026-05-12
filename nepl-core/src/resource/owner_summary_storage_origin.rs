use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{PlaceProjection, StorageOrigin};
use super::summary::OwnerStorageOriginMarker;

pub(super) fn record_storage_origin_marker(
    out: &mut Vec<OwnerStorageOriginMarker>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    origin: StorageOrigin,
) {
    if out
        .iter()
        .any(|entry| entry.suffix == suffix && entry.ty == ty && entry.origin == origin)
    {
        return;
    }
    out.push(OwnerStorageOriginMarker { suffix, ty, origin });
}

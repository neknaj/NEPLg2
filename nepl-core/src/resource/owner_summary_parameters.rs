use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, ResourceFunction};
use super::owner_state::OwnerTable;
use super::owner_summary_i32_leaf::i32_leaf_places;
use super::owner_summary_leaf::owner_seed_leaf_places;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::storage_origin::StorageOriginTable;
use super::summary::OwnerProjectionSource;

pub(super) fn seed_owner_summary_parameters(
    types: &TypeCtx,
    function: &ResourceFunction,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
) -> (
    Vec<OwnerParameterStorageSource>,
    Vec<OwnerParameterConditionSource>,
) {
    let mut parameter_storage_sources = Vec::new();
    let mut parameter_condition_sources = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        for leaf in i32_leaf_places(types, &param.place) {
            parameter_condition_sources.push(OwnerParameterConditionSource {
                source: OwnerProjectionSource {
                    parameter_index: index,
                    suffix: leaf.suffix,
                    ty: leaf.place.ty,
                },
                place: leaf.place,
            });
        }
        for leaf in owner_seed_leaf_places(types, function, index, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
            if let Some(OwnerState::Live { storage, .. }) = owners.state(&leaf.place) {
                parameter_storage_sources.push(OwnerParameterStorageSource {
                    storage,
                    source: OwnerProjectionSource {
                        parameter_index: index,
                        suffix: leaf.suffix,
                        ty: leaf.place.ty,
                    },
                    place: leaf.place,
                });
            }
        }
    }
    (parameter_storage_sources, parameter_condition_sources)
}

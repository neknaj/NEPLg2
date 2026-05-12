use alloc::vec::Vec;

use super::model::OwnerState;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{
    push_unique_owner_projection_source, OwnerParameterStorageSource,
};
use super::place_utils::push_unique_usize;
use super::summary::OwnerProjectionSource;

pub(super) fn consumed_owner_parameters(
    owners: &OwnerTable,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &[OwnerProjectionSource],
) -> (Vec<usize>, Vec<OwnerProjectionSource>) {
    let mut indices = Vec::new();
    let mut sources = Vec::new();
    for entry in parameter_storage_sources {
        let source = &entry.source;
        if returned_sources.iter().any(|returned| returned == source) {
            continue;
        }
        match owners.state(&entry.place) {
            Some(OwnerState::Moved | OwnerState::Freed) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(OwnerState::NoFreeObligation) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(
                OwnerState::Live { .. }
                | OwnerState::Reserved { .. }
                | OwnerState::MaybeFreed { .. },
            )
            | None => {}
        }
    }
    (indices, sources)
}

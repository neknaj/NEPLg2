use alloc::vec::Vec;

use super::model::OwnerState;
use super::owner_projection_source::owner_projection_sources_overlap;
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
        if returned_sources
            .iter()
            .any(|returned| owner_projection_sources_overlap(returned, source))
        {
            continue;
        }
        if !state_consumes_parameter_owner(owners.state(&entry.place), entry) {
            continue;
        }
        if source.suffix.is_empty() {
            push_unique_usize(&mut indices, source.parameter_index);
        } else {
            push_unique_owner_projection_source(&mut sources, source);
        }
    }
    (indices, sources)
}

fn state_consumes_parameter_owner(
    state: Option<OwnerState>,
    source: &OwnerParameterStorageSource,
) -> bool {
    match state {
        Some(OwnerState::Moved | OwnerState::Freed | OwnerState::NoFreeObligation) => true,
        Some(OwnerState::MaybeFreed {
            storage: Some(storage),
        }) => storage == source.storage,
        Some(OwnerState::Live { .. } | OwnerState::Reserved { .. })
        | Some(OwnerState::MaybeFreed { storage: None })
        | None => false,
    }
}

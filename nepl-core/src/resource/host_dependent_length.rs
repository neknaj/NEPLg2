extern crate alloc;

use alloc::vec::Vec;

use super::host_size_contract::HostDependentLength;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::push_unique_place;

pub(super) fn dependent_host_length_candidates(
    raw_aliases: &RawCellAddressAliases,
    length: HostDependentLength,
) -> Vec<Place> {
    let mut out = Vec::new();
    match length {
        HostDependentLength::HostSize(kind) => {
            for place in raw_aliases.host_size_places(kind) {
                push_unique_place(&mut out, &place);
            }
        }
        HostDependentLength::HostSizeScaled {
            kind,
            bytes_per_item,
        } => {
            for source in raw_aliases.host_size_places(kind) {
                for target in raw_aliases.i32_scaled_targets(&source, bytes_per_item) {
                    push_unique_place(&mut out, &target);
                }
            }
        }
    }
    out
}

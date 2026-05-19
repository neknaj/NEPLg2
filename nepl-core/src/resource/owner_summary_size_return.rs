use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_summary_host_size_return::record_host_size_returns;
use super::owner_summary_type_size_return::record_type_size_returns;
use super::summary::{OwnerHostSizeReturn, OwnerTypeSizeReturn};

pub(super) fn record_size_returns(
    host_out: &mut Vec<OwnerHostSizeReturn>,
    type_out: &mut Vec<OwnerTypeSizeReturn>,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    resolved_value: &Place,
) {
    record_host_size_returns(host_out, raw_aliases, value);
    record_type_size_returns(type_out, raw_aliases, value);
    if resolved_value != value {
        record_host_size_returns(host_out, raw_aliases, resolved_value);
        record_type_size_returns(type_out, raw_aliases, resolved_value);
    }
}

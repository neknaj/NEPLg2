use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_call_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        span: Span,
    ) {
        if self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, arg) {
            self.push_unavailable(
                ResourceOwnerOperation::CallArgument,
                arg,
                OwnerState::NoFreeObligation,
                span,
            );
            return;
        }
        if self.has_transferable_owner(owners, raw_aliases, arg) {
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                arg,
                ResourceOwnerOperation::CallArgument,
                span,
            );
        } else if self.storage_origin_expects_owned(storage_origins, raw_aliases, arg) {
            self.push_unavailable(
                ResourceOwnerOperation::CallArgument,
                arg,
                OwnerState::NoFreeObligation,
                span,
            );
        }
    }
}

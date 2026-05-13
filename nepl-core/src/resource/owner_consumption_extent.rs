use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_call_argument_owner_with_extent(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        expected_extent: &OwnerStorageExtent,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        if !self.ensure_owner_extent_matches_summary(
            owners,
            raw_aliases,
            arg,
            expected_extent,
            operation,
            span,
        ) {
            self.push_unavailable(
                operation,
                arg,
                owners.state(arg).unwrap_or(OwnerState::NoFreeObligation),
                span,
            );
            return;
        }
        self.consume_call_argument_owner(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            arg,
            span,
        );
    }
}

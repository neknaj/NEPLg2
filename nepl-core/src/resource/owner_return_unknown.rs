use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_unknown_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        let mut returned_index = None;
        for (index, arg) in args
            .iter()
            .enumerate()
            .filter(|(_, arg)| self.types.same_type(arg.ty, output.ty))
        {
            if self.try_copy_unknown_indirect_non_owning_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                output,
            ) {
                returned_index = Some(index);
                break;
            }
        }
        if returned_index.is_none() {
            for (index, arg) in args
                .iter()
                .enumerate()
                .filter(|(_, arg)| self.types.same_type(arg.ty, output.ty))
            {
                if !self.unknown_indirect_arg_is_non_owning_raw_view(
                    owners,
                    raw_aliases,
                    raw_views,
                    arg,
                ) && self.has_transferable_owner(owners, raw_aliases, arg)
                {
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        arg,
                        output,
                        ResourceOwnerOperation::ReturnValue,
                        span,
                    );
                    returned_index = Some(index);
                    break;
                }
            }
        }
        for (index, arg) in args.iter().enumerate() {
            if returned_index == Some(index)
                || self.unknown_indirect_arg_is_non_owning_raw_view(
                    owners,
                    raw_aliases,
                    raw_views,
                    arg,
                )
            {
                continue;
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

    fn try_copy_unknown_indirect_non_owning_return(
        &self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        output: &Place,
    ) -> bool {
        if !self.unknown_indirect_arg_is_non_owning_raw_view(owners, raw_aliases, raw_views, source)
        {
            return false;
        }
        raw_aliases.clear(output);
        storage_origins.clear(output);
        if raw_views.contains_non_owning(source) {
            raw_views.copy_non_owning(source, output);
        } else {
            raw_views.mark_non_owning(output);
        }
        true
    }

    fn unknown_indirect_arg_is_non_owning_raw_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        arg: &Place,
    ) -> bool {
        raw_views.contains_non_owning(arg) && !owners.has_tracked_state_under(arg)
            || self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, arg)
    }
}

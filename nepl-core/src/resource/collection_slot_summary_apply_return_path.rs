extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_projection::instantiate_summary_suffix_on_base;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::i32_scalar_return_facts::apply_i32_scalar_return_facts;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};
use super::place_utils::{place_suffix_after_prefix, push_unique_place};
use super::summary_projection::SummaryProjection;

pub(super) struct CollectionSlotReturnPathState {
    pub(super) cells: CellTable,
    pub(super) collection_slots: CollectionSlotStateTable,
    pub(super) raw_aliases: RawCellAddressAliases,
}

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_return_transfers(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        output: &Place,
        transfers: &[CollectionSlotLifecycleReturnTransfer],
        span: crate::span::Span,
    ) {
        for transfer in transfers {
            let Some(source) =
                instantiate_summary_target_with_aliases(self, args, raw_aliases, &transfer.source)
            else {
                continue;
            };
            let source = raw_aliases.canonicalize_owner_cell_address(&source);
            let Some(target) = instantiate_summary_suffix_on_base(
                self,
                args,
                output,
                &transfer.target_suffix,
                transfer.target_ty,
            ) else {
                continue;
            };
            self.transfer_slot_state_with_aliases(
                collection_slots,
                &source,
                &target,
                raw_aliases,
                span,
            );
        }
    }

    pub(super) fn apply_collection_slot_return_paths(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        initial_cells: &CellTable,
        initial_collection_slots: &CollectionSlotStateTable,
        initial_raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        paths: &[CollectionSlotLifecycleReturnPath],
        span: crate::span::Span,
    ) -> Vec<CollectionSlotReturnPathState> {
        let mut path_slots = Vec::new();
        let mut path_cells = Vec::new();
        let mut path_aliases = Vec::new();
        let mut path_states = Vec::new();
        for path in paths {
            let mut cells = initial_cells.clone();
            let mut slots = initial_collection_slots.clone();
            let mut aliases = initial_raw_aliases.clone();
            self.apply_collection_slot_lifecycle_summary_ops_state_only(
                &mut cells, &mut slots, &aliases, args, &path.ops, span,
            );
            slots.clear_storage_prefix(output);
            self.apply_collection_slot_return_transfers(
                &mut slots,
                &aliases,
                args,
                output,
                &path.return_transfers,
                span,
            );
            self.apply_collection_slot_return_slots(&mut slots, args, output, &path.return_slots);
            self.clear_consumed_collection_slot_args(&mut slots, &aliases, args);
            apply_i32_scalar_return_facts(
                &mut aliases,
                output,
                args,
                &path.i32_scalar_facts,
                self.types,
            );
            path_cells.push(cells.clone());
            path_slots.push(slots.clone());
            path_aliases.push(aliases.clone());
            path_states.push(CollectionSlotReturnPathState {
                cells,
                collection_slots: slots,
                raw_aliases: aliases,
            });
        }
        *cells = CellTable::merge_paths(&path_cells);
        *collection_slots = merge_collection_slot_return_path_tables(output, paths, &path_slots);
        *raw_aliases = RawCellAddressAliases::merge_paths(&path_aliases);
        path_states
    }
}

fn merge_collection_slot_return_path_tables(
    output: &Place,
    paths: &[CollectionSlotLifecycleReturnPath],
    path_tables: &[CollectionSlotStateTable],
) -> CollectionSlotStateTable {
    let mut merged = CollectionSlotStateTable::merge_paths(path_tables);
    let mut slots = Vec::new();
    for table in path_tables {
        for entry in table.entries() {
            push_unique_place(&mut slots, &entry.slot);
        }
    }

    for slot in slots {
        let Some(slot_suffix) = place_suffix_after_prefix(&slot, output) else {
            continue;
        };
        let mut merged_state = None;
        for (path, table) in paths.iter().zip(path_tables) {
            let state = table.state(&slot);
            let path_does_not_return_slot = matches!(state, CollectionSlotState::Uninitialized)
                && !return_path_targets_slot(path, &slot_suffix);
            if path_does_not_return_slot {
                continue;
            }
            merged_state = Some(match merged_state {
                Some(existing) => merge_collection_slot_states(existing, state),
                None => state,
            });
        }
        if let Some(state) = merged_state {
            merged.set_slot_state(&slot, state);
        }
    }
    merged
}

fn return_path_targets_slot(
    path: &CollectionSlotLifecycleReturnPath,
    slot_suffix: &[PlaceProjection],
) -> bool {
    path.return_transfers
        .iter()
        .any(|transfer| summary_suffix_covers_place_suffix(&transfer.target_suffix, slot_suffix))
        || path
            .return_slots
            .iter()
            .any(|slot| summary_suffix_covers_place_suffix(&slot.suffix, slot_suffix))
}

fn summary_suffix_covers_place_suffix(
    summary_suffix: &[SummaryProjection],
    slot_suffix: &[PlaceProjection],
) -> bool {
    summary_suffix.len() <= slot_suffix.len()
        && summary_prefix_matches_place_prefix(summary_suffix, &slot_suffix[..summary_suffix.len()])
}

fn summary_prefix_matches_place_prefix(
    summary: &[SummaryProjection],
    place: &[PlaceProjection],
) -> bool {
    summary.len() == place.len()
        && summary
            .iter()
            .zip(place)
            .all(|(summary, place)| summary_projection_matches_place_projection(summary, place))
}

fn summary_projection_matches_place_projection(
    summary: &SummaryProjection,
    place: &PlaceProjection,
) -> bool {
    match (summary, place) {
        (
            SummaryProjection::Field {
                index: summary_index,
                offset_bytes: summary_offset,
            },
            PlaceProjection::Field {
                index: place_index,
                offset_bytes: place_offset,
            },
        )
        | (
            SummaryProjection::TupleField {
                index: summary_index,
                offset_bytes: summary_offset,
            },
            PlaceProjection::TupleField {
                index: place_index,
                offset_bytes: place_offset,
            },
        ) => summary_index == place_index && summary_offset == place_offset,
        (
            SummaryProjection::EnumPayload {
                variant: summary_variant,
            },
            PlaceProjection::EnumPayload {
                variant: place_variant,
            },
        ) => summary_variant == place_variant,
        (SummaryProjection::Deref, PlaceProjection::Deref) => true,
        (SummaryProjection::StorageOffset(_), PlaceProjection::StorageOffset(_)) => true,
        _ => false,
    }
}

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_traversal_summary_proof::summary_certified_drop_traversal_proof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_certified_collection_slot_drop_traversal_forall_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let drop_proof = summary_certified_drop_traversal_proof(expected_ty);
        let result = self.collection_slot_drop_traversal_result_with_drop_proof(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            &initialized_count,
            expected_ty,
            drop_proof,
        );
        if let Err(refutation) = result {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.into(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }
}

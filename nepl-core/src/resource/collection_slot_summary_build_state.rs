use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_seed::seed_summary_input_place;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::ResourceFunction;
use super::place_utils::reference_target_place;
use super::raw_realloc::PendingRawReallocs;

#[derive(Clone)]
pub(super) struct CollectionSlotSummaryBuildState {
    pub(super) cells: CellTable,
    pub(super) collection_slots: CollectionSlotStateTable,
    pub(super) raw_aliases: RawCellAddressAliases,
    pub(super) function_aliases: FunctionAliasTable,
    pub(super) pending_reallocs: PendingRawReallocs,
    pub(super) variant_initializations: PendingVariantRawCellInitializations,
}

impl CollectionSlotSummaryBuildState {
    pub(super) fn new(types: &TypeCtx, function: &ResourceFunction) -> Self {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        for param in &function.params {
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = reference_target_type(types, param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
            }
        }
        Self {
            cells,
            collection_slots: CollectionSlotStateTable::new(),
            raw_aliases,
            function_aliases: FunctionAliasTable::default(),
            pending_reallocs: PendingRawReallocs::default(),
            variant_initializations: PendingVariantRawCellInitializations::default(),
        }
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

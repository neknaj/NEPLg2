use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

#[derive(Clone, Copy)]
pub(super) struct CopyRawElementType {
    ty: TypeId,
    stride: usize,
}

impl CopyRawElementType {
    pub(super) fn new(ty: TypeId, types: &TypeCtx) -> Option<Self> {
        types.is_copy(ty).then(|| Self {
            ty,
            stride: storage_size_bytes(types, ty),
        })
    }

    fn ty(self) -> TypeId {
        self.ty
    }

    fn stride(self) -> usize {
        self.stride
    }
}

pub(super) enum RawCellLifecycleEvent<'a> {
    MoveOutLoadedCell {
        address: &'a Place,
        cell_ty: TypeId,
    },
    StoreValue {
        address: &'a Place,
        value: &'a Place,
        stored_ty: TypeId,
    },
    DiscardCellsUnderAddress {
        address: &'a Place,
    },
    ReleaseStorage {
        address: &'a Place,
    },
    ReallocSuccessTransfer {
        source: &'a Place,
        result: &'a Place,
    },
    BulkCopyInitializedRawState {
        source: &'a Place,
        destination: &'a Place,
        count: Option<&'a Place>,
    },
    FillBytes {
        address: &'a Place,
        count: &'a Place,
    },
    FillCopyElements {
        address: &'a Place,
        count: &'a Place,
        element_ty: CopyRawElementType,
    },
}

impl CellTable {
    pub(super) fn apply_raw_cell_lifecycle_event(
        &mut self,
        event: RawCellLifecycleEvent<'_>,
        raw_aliases: &mut RawCellAddressAliases,
        types: &TypeCtx,
    ) {
        match event {
            RawCellLifecycleEvent::MoveOutLoadedCell { address, cell_ty } => {
                if !types.is_copy(cell_ty) {
                    self.mark_raw_cell_moved(address, cell_ty);
                    self.record_raw_cell_value_flow_with_aliases(
                        raw_aliases,
                        address,
                        cell_ty,
                        RawCellValueFlowKind::MoveOutLoadedCell,
                    );
                }
            }
            RawCellLifecycleEvent::StoreValue {
                address,
                value,
                stored_ty,
            } => {
                let cell = raw_memory_cell_place(address, stored_ty);
                self.clear_raw_cells_overwritten_by_store(address, stored_ty, types);
                self.clear_initialized_raw_byte_ranges_through_value(&cell);
                self.mark_initialized(&cell);
                raw_aliases.clear(&cell);
                raw_aliases.copy_alias_if_tracked(value, &cell);
                self.copy_initialized_raw_byte_ranges_through_value_aliases(
                    value,
                    &cell,
                    raw_aliases,
                );
                if !types.is_copy(stored_ty) {
                    self.record_raw_cell_value_flow_with_aliases(
                        raw_aliases,
                        address,
                        stored_ty,
                        RawCellValueFlowKind::StoreValue,
                    );
                }
            }
            RawCellLifecycleEvent::DiscardCellsUnderAddress { address } => {
                self.clear_raw_cells_under(address);
            }
            RawCellLifecycleEvent::ReleaseStorage { address } => {
                self.clear_raw_cells_under(address);
                self.release_owned_raw_storage_under(address);
            }
            RawCellLifecycleEvent::ReallocSuccessTransfer { source, result } => {
                let source_owned = self.owns_raw_storage_under(source);
                let relocated = self.copy_initialized_copy_raw_cells(source, result, types);
                let relocated_ranges = self.copy_initialized_raw_byte_ranges_under(source, result);
                self.clear_raw_cells_under(source);
                self.clear_raw_cells_under(result);
                self.release_owned_raw_storage_under(source);
                self.mark_initialized(result);
                if source_owned {
                    self.mark_owned_raw_storage_root(result);
                }
                self.extend_entries(relocated);
                self.extend_initialized_raw_byte_ranges(relocated_ranges);
            }
            RawCellLifecycleEvent::BulkCopyInitializedRawState {
                source,
                destination,
                count,
            } => {
                let copied = count
                    .map(|count| {
                        self.copy_initialized_copy_raw_cells_covered_by_count(
                            source,
                            destination,
                            count,
                            raw_aliases,
                            types,
                        )
                    })
                    .unwrap_or_default();
                let copied_ranges = count
                    .map(|count| {
                        self.copy_initialized_raw_byte_ranges_for_bulk_copy(
                            source,
                            destination,
                            count,
                            raw_aliases,
                        )
                    })
                    .unwrap_or_default();
                self.clear_raw_cells_under(destination);
                self.extend_entries(copied);
                self.extend_initialized_raw_byte_ranges(copied_ranges);
            }
            RawCellLifecycleEvent::FillBytes { address, count } => {
                self.clear_raw_cells_under(address);
                self.mark_initialized_raw_byte_range_extending_appended_difference(
                    address,
                    count,
                    InitializedRawRangeUnit::Bytes,
                    types.u8(),
                    raw_aliases,
                );
            }
            RawCellLifecycleEvent::FillCopyElements {
                address,
                count,
                element_ty,
            } => {
                self.clear_raw_cells_under(address);
                self.mark_initialized_raw_byte_range(
                    address,
                    count,
                    InitializedRawRangeUnit::Elements {
                        stride: element_ty.stride(),
                    },
                    element_ty.ty(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::types::TypeKind;

    use super::*;

    #[test]
    fn copy_raw_element_type_requires_copy_evidence() {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.i32());
        let owned_ty = types.register_named(
            String::from("Owned"),
            TypeKind::Struct {
                name: String::from("Owned"),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        );

        assert!(CopyRawElementType::new(types.i32(), &types).is_some());
        assert!(CopyRawElementType::new(owned_ty, &types).is_none());
    }
}

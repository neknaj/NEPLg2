extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
};
use super::initialized_summary_apply_param::apply_param_initialization_summary;
use super::initialized_summary_byte_range_model::{
    RawCellInitializationParamByteRange, RawCellInitializationParamCount,
};
use super::model::{CellState, Place, PlaceProjection, ResourceOffset};
use super::summary_projection::{SummaryOffset, SummaryPlace, SummaryProjection};

#[test]
fn param_cell_summary_instantiates_symbolic_offset_with_caller_argument() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let storage = Place::local("caller_storage".to_string(), i32_ty);
    let index = Place::local("caller_index".to_string(), i32_ty);
    let summary = summary_with_param_cell(RawCellInitializationParamCell {
        param_index: 0,
        suffix: scaled_cell_suffix(i32_ty, 1),
        ty: i32_ty,
        holds_raw_address: false,
    });
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    apply_param_initialization_summary(
        &types,
        &mut cells,
        &mut raw_aliases,
        &[storage.clone(), index.clone()],
        &summary,
    );

    let expected = storage
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
                place: Box::new(index),
                scale: 4,
            }),
            i32_ty,
        )
        .with_projection(PlaceProjection::Deref, i32_ty);
    assert_eq!(
        cells.availability_state(&expected),
        CellState::Initialized(i32_ty)
    );
}

#[test]
fn param_byte_range_summary_instantiates_symbolic_address_offset_with_caller_argument() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let storage = Place::local("caller_storage".to_string(), i32_ty);
    let index = Place::local("caller_index".to_string(), i32_ty);
    let summary = summary_with_param_byte_range(RawCellInitializationParamByteRange {
        address_param_index: 0,
        address_suffix: scaled_address_suffix(i32_ty, 1),
        address_ty: i32_ty,
        count: RawCellInitializationParamCount::KnownI32 {
            value: 4,
            ty: i32_ty,
        },
        unit: InitializedRawRangeUnit::Bytes,
        ty: i32_ty,
    });
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    apply_param_initialization_summary(
        &types,
        &mut cells,
        &mut raw_aliases,
        &[storage.clone(), index.clone()],
        &summary,
    );

    let expected_address = storage.with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index),
            scale: 4,
        }),
        i32_ty,
    );
    assert!(cells.raw_cell_initialized_by_byte_range(
        &expected_address,
        i32_ty,
        &raw_aliases,
        &types
    ));
}

fn summary_with_param_cell(
    cell: RawCellInitializationParamCell,
) -> RawCellInitializationFunctionSummary {
    let mut summary = empty_summary();
    summary.param_cells.push(cell);
    summary
}

fn summary_with_param_byte_range(
    range: RawCellInitializationParamByteRange,
) -> RawCellInitializationFunctionSummary {
    let mut summary = empty_summary();
    summary.param_byte_ranges.push(range);
    summary
}

fn empty_summary() -> RawCellInitializationFunctionSummary {
    RawCellInitializationFunctionSummary {
        function: "test_summary".to_string(),
        return_cells: vec![],
        return_byte_ranges: vec![],
        param_cells: vec![],
        param_byte_ranges: vec![],
        param_release_requirements: vec![],
        variant_param_cells: vec![],
        variant_param_byte_ranges: vec![],
        variant_required_param_cells: vec![],
        variant_conditions: vec![],
    }
}

fn scaled_cell_suffix(ty: crate::types::TypeId, index_param: usize) -> Vec<SummaryProjection> {
    let mut suffix = scaled_address_suffix(ty, index_param);
    suffix.push(SummaryProjection::Deref);
    suffix
}

fn scaled_address_suffix(ty: crate::types::TypeId, index_param: usize) -> Vec<SummaryProjection> {
    vec![SummaryProjection::StorageOffset(
        SummaryOffset::ScaledSymbolic {
            place: Box::new(SummaryPlace {
                parameter_index: index_param,
                suffix: vec![],
                ty,
            }),
            scale: 4,
        },
    )]
}

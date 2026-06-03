use alloc::format;
use alloc::string::String;
use alloc::vec;

use crate::ast::Effect;
use crate::function_identity::FunctionValueIdentity;
use crate::resource::model::{CellState, Place, ResourceFunctionValueKind};
use crate::types::TypeCtx;
use crate::types::TypeId;

use super::*;

fn empty_resource_check_state() -> ResourceCheckState {
    ResourceCheckState::new(
        CellTable::default(),
        CollectionSlotStateTable::new(),
        RawCellAddressAliases::default(),
        FunctionAliasTable::default(),
        PendingRawReallocs::default(),
        PendingVariantRawCellInitializations::default(),
    )
}

fn resource_check_state_with_function_alias(index: usize) -> ResourceCheckState {
    let mut function_aliases = FunctionAliasTable::default();
    let name = format!("f{index}");
    function_aliases.set_alias(
        &Place::local(String::from("callback"), TypeId(0)),
        FunctionValueIdentity::new(String::from(name), None, TypeId(0), Effect::Pure, vec![]),
        ResourceFunctionValueKind::Plain,
    );
    ResourceCheckState::new(
        CellTable::default(),
        CollectionSlotStateTable::new(),
        RawCellAddressAliases::default(),
        function_aliases,
        PendingRawReallocs::default(),
        PendingVariantRawCellInitializations::default(),
    )
}

fn resource_check_state_with_result_variant(index: usize, result: &Place) -> ResourceCheckState {
    let mut state = empty_resource_check_state();
    let variant = if index % 2 == 0 {
        "Result::Ok"
    } else {
        "Result::Err"
    };
    state
        .variant_initializations
        .record_concrete_variant(result, variant);
    state
}

#[test]
fn path_alternatives_merge_to_single_state_after_precision_budget() {
    let alternatives = (0..=MAX_PATH_SENSITIVE_ALTERNATIVES)
        .map(resource_check_state_with_function_alias)
        .collect::<Vec<_>>();

    let ResourcePathAlternatives::Feasible(states) =
        ResourcePathAlternatives::from_states(alternatives)
    else {
        panic!("from_states should keep feasible path alternatives");
    };

    assert_eq!(states.len(), 1);
}

#[test]
fn path_alternatives_preserve_call_result_variants_when_budgeting() {
    let result = Place::local(String::from("result"), TypeId(0));
    let alternatives = (0..=MAX_PATH_SENSITIVE_ALTERNATIVES)
        .map(|index| resource_check_state_with_result_variant(index, &result))
        .collect::<Vec<_>>();

    let ResourcePathAlternatives::Feasible(states) =
        ResourcePathAlternatives::from_states_preserving_result_variants(alternatives, &result)
    else {
        panic!("from_states should keep feasible path alternatives");
    };

    assert_eq!(states.len(), 2);
    assert!(states
        .iter()
        .any(|state| { state.variant_initializations.concrete_variant(&result) == Some("Ok") }));
    assert!(states
        .iter()
        .any(|state| { state.variant_initializations.concrete_variant(&result) == Some("Err") }));
}

#[test]
fn path_alternatives_drop_duplicate_states_before_precision_budgeting() {
    let state = empty_resource_check_state();
    let alternatives = vec![state; MAX_PATH_SENSITIVE_ALTERNATIVES + 1];

    let ResourcePathAlternatives::Feasible(states) =
        ResourcePathAlternatives::from_states(alternatives)
    else {
        panic!("from_states should keep feasible path alternatives");
    };

    assert_eq!(states.len(), 1);
}

#[test]
fn path_replay_keeps_branch_local_cell_facts() {
    let value = Place::local(String::from("value"), TypeId(1));
    let mut initialized_path = empty_resource_check_state();
    initialized_path
        .cells
        .set_state(&value, CellState::Initialized(value.ty));
    let uninitialized_path = empty_resource_check_state();

    assert!(path_states_need_replay(&[
        initialized_path,
        uninitialized_path
    ]));
}

#[test]
fn path_replay_keeps_branch_local_raw_alias_facts() {
    let value = Place::local(String::from("value"), TypeId(1));
    let mut constant_path = empty_resource_check_state();
    constant_path.raw_aliases.set_i32_value(&value, 1);
    let unknown_path = empty_resource_check_state();

    assert!(path_states_need_replay(&[constant_path, unknown_path]));
}

#[test]
fn unit_control_output_drops_scalar_only_path_replay() {
    let types = TypeCtx::new();
    let value = Place::local(String::from("value"), TypeId(1));
    let output = Place::local(String::from("output"), types.unit());
    let mut left = empty_resource_check_state();
    left.cells.mark_initialized(&value);
    left.raw_aliases.set_i32_value(&value, 1);
    let mut right = empty_resource_check_state();
    right.cells.mark_initialized(&value);

    assert!(!control_path_states_need_replay(
        &types,
        &[left, right],
        &output
    ));
}

#[test]
fn copy_control_output_drops_concrete_variant_only_path_replay() {
    let types = TypeCtx::new();
    let value = Place::local(String::from("value"), TypeId(1));
    let output = Place::local(String::from("output"), types.unit());
    let left = resource_check_state_with_result_variant(0, &value);
    let right = resource_check_state_with_result_variant(1, &value);

    assert!(!control_path_states_need_replay(
        &types,
        &[left, right],
        &output
    ));
}

#[test]
fn non_copy_control_output_keeps_concrete_variant_path_replay() {
    let types = TypeCtx::new();
    let value = Place::local(String::from("value"), TypeId(1));
    let output = Place::local(String::from("output"), types.str());
    let left = resource_check_state_with_result_variant(0, &value);
    let right = resource_check_state_with_result_variant(1, &value);

    assert!(control_path_states_need_replay(
        &types,
        &[left, right],
        &output
    ));
}

#[test]
fn unit_control_output_keeps_paths_that_merge_to_maybe_moved() {
    let types = TypeCtx::new();
    let value = Place::local(String::from("value"), types.str());
    let output = Place::local(String::from("output"), types.unit());
    let mut initialized = empty_resource_check_state();
    initialized.cells.mark_initialized(&value);
    let uninitialized = empty_resource_check_state();

    assert!(control_path_states_need_replay(
        &types,
        &[initialized, uninitialized],
        &output
    ));
}

#[test]
fn unit_control_output_keeps_non_scalar_resource_path_differences() {
    let types = TypeCtx::new();
    let output = Place::local(String::from("output"), types.unit());
    let mut left = empty_resource_check_state();
    left.function_aliases.set_alias(
        &Place::local(String::from("callback"), TypeId(0)),
        FunctionValueIdentity::new(String::from("left"), None, TypeId(0), Effect::Pure, vec![]),
        ResourceFunctionValueKind::Plain,
    );
    let right = empty_resource_check_state();

    assert!(control_path_states_need_replay(
        &types,
        &[left, right],
        &output
    ));
}

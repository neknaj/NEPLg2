use alloc::format;
use alloc::string::String;
use alloc::vec;

use crate::ast::Effect;
use crate::function_identity::FunctionValueIdentity;
use crate::resource::model::Place;
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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use super::raw_place::{RawPlaceInfo, RawPlaceState};
use super::state::{BorrowBinding, FieldMove, ResourceStateSnapshot, VarState};
use super::MoveCheckContext;

pub(super) struct BranchStateSnapshot {
    pub(super) continues: bool,
    pub(super) state: ResourceStateSnapshot,
}

pub(super) fn snapshot_top_state(snapshot: &ResourceStateSnapshot, name: &str) -> Option<VarState> {
    snapshot
        .var_stacks
        .get(name)
        .and_then(|stack| stack.last().copied())
}

pub(super) fn changed_state_names(
    start: &ResourceStateSnapshot,
    end: &ResourceStateSnapshot,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for name in start.var_stacks.keys() {
        if snapshot_top_state(start, name) != snapshot_top_state(end, name) {
            names.insert(name.clone());
        }
    }
    for name in end.var_stacks.keys() {
        if snapshot_top_state(start, name) != snapshot_top_state(end, name) {
            names.insert(name.clone());
        }
    }
    for name in start.field_move_stacks.keys() {
        if start.field_move_stacks.get(name) != end.field_move_stacks.get(name) {
            names.insert(name.clone());
        }
    }
    for name in end.field_move_stacks.keys() {
        if start.field_move_stacks.get(name) != end.field_move_stacks.get(name) {
            names.insert(name.clone());
        }
    }
    names
}

fn push_unique_binding(out: &mut Vec<BorrowBinding>, binding: &BorrowBinding) {
    if !out.contains(binding) {
        out.push(binding.clone());
    }
}

fn merge_raw_place_state_pair(
    a: Option<RawPlaceInfo>,
    b: Option<RawPlaceInfo>,
) -> Option<RawPlaceInfo> {
    use RawPlaceState::*;
    let size = a
        .map(|info| info.size)
        .unwrap_or(0)
        .max(b.map(|info| info.size).unwrap_or(0));
    let state = match (a.map(|info| info.state), b.map(|info| info.state)) {
        (None, None) => return None,
        (Some(left), Some(right)) if left == right => left,
        (Some(_), Some(_)) => PossiblyMoved,
        (Some(Initialized), None) | (None, Some(Initialized)) => PossiblyMoved,
        (Some(Moved), None) | (None, Some(Moved)) => PossiblyMoved,
        (Some(PossiblyMoved), None) | (None, Some(PossiblyMoved)) => PossiblyMoved,
    };
    Some(RawPlaceInfo { state, size })
}

fn merge_raw_place_states(branches: &[&BranchStateSnapshot]) -> BTreeMap<String, RawPlaceInfo> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.raw_place_states.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let mut branch_iter = branches.iter();
        let Some(first_branch) = branch_iter.next() else {
            continue;
        };
        let mut state = first_branch
            .state
            .raw_place_states
            .get(name.as_str())
            .copied();
        for branch in branch_iter {
            let branch_state = branch.state.raw_place_states.get(name.as_str()).copied();
            state = merge_raw_place_state_pair(state, branch_state);
        }
        if let Some(state) = state {
            merged.insert(name, state);
        }
    }
    merged
}

fn merge_raw_addr_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<Option<String>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.raw_addr_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| branch.state.raw_addr_alias_stacks.get(name.as_str()))
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .raw_addr_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or(None)
            });
            let first = branch_values.next().unwrap_or(None);
            if branch_values.all(|alias| alias == first) {
                stack.push(first);
            } else {
                stack.push(None);
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_i32_const_stacks(branches: &[&BranchStateSnapshot]) -> BTreeMap<String, Vec<Option<i64>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.i32_const_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| branch.state.i32_const_stacks.get(name.as_str()))
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .i32_const_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .copied()
                    .unwrap_or(None)
            });
            let first = branch_values.next().unwrap_or(None);
            if branch_values.all(|value| value == first) {
                stack.push(first);
            } else {
                stack.push(None);
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_function_value_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeSet<String>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.function_value_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| branch.state.function_value_alias_stacks.get(name.as_str()))
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut aliases = BTreeSet::new();
            for branch in branches {
                branch
                    .state
                    .function_value_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .for_each(|alias| {
                        aliases.insert(alias);
                    });
            }
            stack.push(aliases);
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_enum_payload_raw_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<String, String>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.enum_payload_raw_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .enum_payload_raw_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .enum_payload_raw_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
            });
            let first = branch_values.next().unwrap_or_default();
            if branch_values.all(|aliases| aliases == first) {
                stack.push(first);
            } else {
                stack.push(BTreeMap::new());
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_enum_payload_function_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<String, BTreeSet<String>>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.enum_payload_function_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .enum_payload_function_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut merged_aliases = BTreeMap::<String, BTreeSet<String>>::new();
            for branch in branches {
                for (variant, aliases) in branch
                    .state
                    .enum_payload_function_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
                {
                    merged_aliases.entry(variant).or_default().extend(aliases);
                }
            }
            stack.push(merged_aliases);
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_aggregate_field_raw_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<usize, String>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.aggregate_field_raw_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .aggregate_field_raw_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .aggregate_field_raw_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
            });
            let first = branch_values.next().unwrap_or_default();
            if branch_values.all(|aliases| aliases == first) {
                stack.push(first);
            } else {
                stack.push(BTreeMap::new());
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_aggregate_field_function_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<usize, BTreeSet<String>>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch.state.aggregate_field_function_alias_stacks.keys() {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .aggregate_field_function_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut merged_aliases = BTreeMap::<usize, BTreeSet<String>>::new();
            for branch in branches {
                for (offset, aliases) in branch
                    .state
                    .aggregate_field_function_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
                {
                    merged_aliases.entry(offset).or_default().extend(aliases);
                }
            }
            stack.push(merged_aliases);
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_enum_payload_aggregate_field_raw_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, String>>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch
            .state
            .enum_payload_aggregate_field_raw_alias_stacks
            .keys()
        {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .enum_payload_aggregate_field_raw_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut branch_values = branches.iter().map(|branch| {
                branch
                    .state
                    .enum_payload_aggregate_field_raw_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
            });
            let first = branch_values.next().unwrap_or_default();
            if branch_values.all(|aliases| aliases == first) {
                stack.push(first);
            } else {
                stack.push(BTreeMap::new());
            }
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merge_enum_payload_aggregate_field_function_alias_stacks(
    branches: &[&BranchStateSnapshot],
) -> BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>>> {
    let mut names = BTreeSet::new();
    for branch in branches {
        for name in branch
            .state
            .enum_payload_aggregate_field_function_alias_stacks
            .keys()
        {
            names.insert(name.clone());
        }
    }

    let mut merged = BTreeMap::new();
    for name in names {
        let max_len = branches
            .iter()
            .filter_map(|branch| {
                branch
                    .state
                    .enum_payload_aggregate_field_function_alias_stacks
                    .get(name.as_str())
            })
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut stack = Vec::with_capacity(max_len);
        for index in 0..max_len {
            let mut merged_aliases = BTreeMap::<String, BTreeMap<usize, BTreeSet<String>>>::new();
            for branch in branches {
                for (variant, field_aliases) in branch
                    .state
                    .enum_payload_aggregate_field_function_alias_stacks
                    .get(name.as_str())
                    .and_then(|stack| stack.get(index))
                    .cloned()
                    .unwrap_or_default()
                {
                    let merged_field_aliases = merged_aliases.entry(variant).or_default();
                    for (offset, aliases) in field_aliases {
                        merged_field_aliases
                            .entry(offset)
                            .or_default()
                            .extend(aliases);
                    }
                }
            }
            stack.push(merged_aliases);
        }
        if !stack.is_empty() {
            merged.insert(name, stack);
        }
    }
    merged
}

fn merged_branch_borrow_stack(
    name: &str,
    active_len: usize,
    saved: &ResourceStateSnapshot,
    branches: &[&BranchStateSnapshot],
) -> Vec<Vec<BorrowBinding>> {
    let saved_stack = saved.borrow_stacks.get(name);
    let mut merged = Vec::with_capacity(active_len);
    for index in 0..active_len {
        let mut bindings = Vec::new();
        for branch in branches {
            let branch_bindings = branch
                .state
                .borrow_stacks
                .get(name)
                .and_then(|stack| stack.get(index))
                .or_else(|| saved_stack.and_then(|stack| stack.get(index)));
            if let Some(branch_bindings) = branch_bindings {
                for binding in branch_bindings {
                    push_unique_binding(&mut bindings, binding);
                }
            }
        }
        merged.push(bindings);
    }
    merged
}

fn snapshot_top_field_moves(snapshot: &ResourceStateSnapshot, name: &str) -> BTreeSet<FieldMove> {
    snapshot
        .field_move_stacks
        .get(name)
        .and_then(|stack| stack.last())
        .cloned()
        .unwrap_or_default()
}

pub(super) fn merge_continuing_branch_states(
    ctx: &mut MoveCheckContext,
    saved: &ResourceStateSnapshot,
    branches: &[BranchStateSnapshot],
) {
    let continuing: Vec<&BranchStateSnapshot> =
        branches.iter().filter(|branch| branch.continues).collect();
    if continuing.is_empty() {
        ctx.restore_resource_state(saved);
        return;
    }
    let merged_raw_place_states = merge_raw_place_states(&continuing);
    let merged_raw_addr_alias_stacks = merge_raw_addr_alias_stacks(&continuing);
    let merged_i32_const_stacks = merge_i32_const_stacks(&continuing);
    let merged_function_value_alias_stacks = merge_function_value_alias_stacks(&continuing);
    let merged_enum_payload_raw_alias_stacks = merge_enum_payload_raw_alias_stacks(&continuing);
    let merged_enum_payload_function_alias_stacks =
        merge_enum_payload_function_alias_stacks(&continuing);
    let merged_aggregate_field_raw_alias_stacks =
        merge_aggregate_field_raw_alias_stacks(&continuing);
    let merged_aggregate_field_function_alias_stacks =
        merge_aggregate_field_function_alias_stacks(&continuing);
    let merged_enum_payload_aggregate_field_raw_alias_stacks =
        merge_enum_payload_aggregate_field_raw_alias_stacks(&continuing);
    let merged_enum_payload_aggregate_field_function_alias_stacks =
        merge_enum_payload_aggregate_field_function_alias_stacks(&continuing);

    ctx.restore_resource_state(saved);

    let mut names = BTreeSet::new();
    for name in saved.var_stacks.keys() {
        names.insert(name.clone());
    }
    for branch in &continuing {
        for name in branch.state.var_stacks.keys() {
            names.insert(name.clone());
        }
    }

    for name in &names {
        let mut states = Vec::new();
        for branch in &continuing {
            let state = snapshot_top_state(&branch.state, name)
                .or_else(|| snapshot_top_state(saved, name))
                .unwrap_or(VarState::Valid);
            states.push(state);
        }
        if states.is_empty() {
            continue;
        }
        let merged = MoveCheckContext::merge_states(&states);
        ctx.set_state(name.as_str(), merged);

        let mut field_moves: Option<BTreeSet<FieldMove>> = None;
        let mut field_moves_match = true;
        for branch in &continuing {
            let branch_moves = snapshot_top_field_moves(&branch.state, name.as_str());
            match &field_moves {
                Some(existing) if *existing != branch_moves => {
                    field_moves_match = false;
                    break;
                }
                Some(_) => {}
                None => field_moves = Some(branch_moves),
            }
        }
        if field_moves_match {
            ctx.set_field_moves(name.as_str(), field_moves.unwrap_or_default());
        } else {
            ctx.clear_field_moves(name.as_str());
            ctx.set_state(name.as_str(), VarState::PossiblyMoved);
        }
    }

    let active_names: Vec<(String, usize)> = ctx
        .var_stacks
        .iter()
        .map(|(name, stack)| (name.clone(), stack.len()))
        .collect();
    ctx.borrow_stacks.clear();
    for (name, active_len) in active_names {
        let merged_stack =
            merged_branch_borrow_stack(name.as_str(), active_len, saved, &continuing);
        ctx.borrow_stacks.insert(name, merged_stack);
    }
    ctx.raw_addr_alias_stacks = merged_raw_addr_alias_stacks;
    ctx.i32_const_stacks = merged_i32_const_stacks;
    ctx.function_value_alias_stacks = merged_function_value_alias_stacks;
    ctx.enum_payload_raw_alias_stacks = merged_enum_payload_raw_alias_stacks;
    ctx.enum_payload_function_alias_stacks = merged_enum_payload_function_alias_stacks;
    ctx.aggregate_field_raw_alias_stacks = merged_aggregate_field_raw_alias_stacks;
    ctx.aggregate_field_function_alias_stacks = merged_aggregate_field_function_alias_stacks;
    ctx.enum_payload_aggregate_field_raw_alias_stacks =
        merged_enum_payload_aggregate_field_raw_alias_stacks;
    ctx.enum_payload_aggregate_field_function_alias_stacks =
        merged_enum_payload_aggregate_field_function_alias_stacks;
    ctx.raw_place_states = merged_raw_place_states;
    ctx.rebuild_borrow_counts_from_bindings();
    ctx.release_dead_borrows();
}

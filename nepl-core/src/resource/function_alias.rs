use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::model::{AggregateKind, Place, ResourceMatchArm, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};

#[derive(Debug, Clone, Default)]
pub(super) struct FunctionAliasTable {
    entries: Vec<FunctionAliasEntry>,
}

#[derive(Debug, Clone)]
struct FunctionAliasEntry {
    place: Place,
    functions: Vec<String>,
}

impl FunctionAliasTable {
    pub(super) fn functions(&self, place: &Place) -> &[String] {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.functions.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn set_alias(&mut self, place: &Place, function: String) {
        self.set_functions(place, vec![function]);
    }

    pub(super) fn copy_alias(&mut self, source: &Place, target: &Place) {
        let functions = self.functions(source).to_vec();
        if !functions.is_empty() {
            self.set_functions(target, functions);
        } else {
            self.clear_alias(target);
        }
    }

    pub(super) fn merge_paths(paths: &[FunctionAliasTable]) -> Self {
        let mut out = FunctionAliasTable::default();
        for path in paths {
            for entry in &path.entries {
                out.union_functions(&entry.place, entry.functions.iter().cloned());
            }
        }
        out
    }

    fn set_functions(&mut self, place: &Place, functions: Vec<String>) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.place == *place) {
            entry.functions = dedupe_functions(functions);
            return;
        }
        self.entries.push(FunctionAliasEntry {
            place: place.clone(),
            functions: dedupe_functions(functions),
        });
    }

    pub(super) fn clear_alias(&mut self, place: &Place) {
        self.entries.retain(|entry| entry.place != *place);
    }

    fn union_functions<I>(&mut self, place: &Place, functions: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut merged = self.functions(place).to_vec();
        for function in functions {
            if !merged.contains(&function) {
                merged.push(function);
            }
        }
        if !merged.is_empty() {
            self.set_functions(place, merged);
        }
    }
}

pub(super) fn construct_function_alias_fields(
    function_aliases: &mut FunctionAliasTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        function_aliases.copy_alias(input, &field);
    }
}

pub(super) fn function_aliases_after_ops(
    initial: &FunctionAliasTable,
    ops: &[ResourceOp],
) -> FunctionAliasTable {
    let mut function_aliases = initial.clone();
    apply_function_alias_ops(&mut function_aliases, ops);
    function_aliases
}

pub(super) fn function_aliases_for_match_arm(
    initial: &FunctionAliasTable,
    scrutinee: &Place,
    arm: &ResourceMatchArm,
) -> FunctionAliasTable {
    let mut function_aliases = initial.clone();
    if let Some(bind_local) = &arm.bind_local {
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            function_aliases.copy_alias(&source, bind_local);
        } else {
            function_aliases.clear_alias(bind_local);
        }
    }
    function_aliases
}

fn apply_function_alias_ops(function_aliases: &mut FunctionAliasTable, ops: &[ResourceOp]) {
    for op in ops {
        apply_function_alias_op(function_aliases, op);
    }
}

fn apply_function_alias_op(function_aliases: &mut FunctionAliasTable, op: &ResourceOp) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                function_aliases.copy_alias(initializer, place);
            } else {
                function_aliases.clear_alias(place);
            }
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            function_aliases.copy_alias(source, output);
        }
        ResourceOp::Assign { target, value, .. } => {
            function_aliases.copy_alias(value, target);
        }
        ResourceOp::Borrow { output, .. }
        | ResourceOp::Expr { output, .. }
        | ResourceOp::Call { output, .. }
        | ResourceOp::IndirectCall { output, .. }
        | ResourceOp::RawMemory { output, .. } => {
            function_aliases.clear_alias(output);
        }
        ResourceOp::Drop { place, .. } => {
            function_aliases.clear_alias(place);
        }
        ResourceOp::EndScope { locals, .. } => {
            for local in locals {
                function_aliases.clear_alias(local);
            }
        }
        ResourceOp::CallEffect { .. } => {}
        ResourceOp::FunctionValue { output, name, .. } => {
            function_aliases.set_alias(output, name.clone());
        }
        ResourceOp::RawAddressAlias { target, .. } | ResourceOp::RawAddressView { target, .. } => {
            function_aliases.clear_alias(target);
        }
        ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. } => {}
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            function_aliases.clear_alias(output);
            construct_function_alias_fields(function_aliases, output, kind, inputs);
        }
        ResourceOp::Branch {
            output,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            let mut then_aliases = function_aliases.clone();
            apply_function_alias_ops(&mut then_aliases, then_ops);
            then_aliases.copy_alias(then_value, output);
            let mut else_aliases = function_aliases.clone();
            apply_function_alias_ops(&mut else_aliases, else_ops);
            else_aliases.copy_alias(else_value, output);
            *function_aliases = FunctionAliasTable::merge_paths(&[then_aliases, else_aliases]);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_aliases = function_aliases.clone();
            apply_function_alias_ops(&mut condition_aliases, condition_ops);
            let mut body_aliases = condition_aliases.clone();
            apply_function_alias_ops(&mut body_aliases, body_ops);
            *function_aliases = FunctionAliasTable::merge_paths(&[condition_aliases, body_aliases]);
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            let mut arm_paths = Vec::new();
            for arm in arms {
                let mut arm_aliases =
                    function_aliases_for_match_arm(function_aliases, scrutinee, arm);
                apply_function_alias_ops(&mut arm_aliases, &arm.ops);
                arm_aliases.copy_alias(&arm.value, output);
                arm_paths.push(arm_aliases);
            }
            if arm_paths.is_empty() {
                function_aliases.clear_alias(output);
            } else {
                *function_aliases = FunctionAliasTable::merge_paths(&arm_paths);
            }
        }
    }
}

fn dedupe_functions(functions: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for function in functions {
        if !out.contains(&function) {
            out.push(function);
        }
    }
    out
}

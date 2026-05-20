extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::model::{ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp};

pub(super) fn build_function_summary_dependents(module: &ResourceModule) -> Vec<Vec<usize>> {
    let mut function_indices = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        function_indices.insert(function.name.as_str(), index);
    }

    let mut dependents = vec![Vec::new(); module.functions.len()];
    for (caller_index, function) in module.functions.iter().enumerate() {
        let mut dependencies = BTreeSet::new();
        collect_function_summary_dependencies(function, &mut dependencies);
        for dependency in dependencies {
            if let Some(dependency_index) = function_indices.get(dependency.as_str()) {
                dependents[*dependency_index].push(caller_index);
            }
        }
    }
    dependents
}

pub(super) fn build_function_summary_dependencies(module: &ResourceModule) -> Vec<Vec<usize>> {
    let mut function_indices = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        function_indices.insert(function.name.as_str(), index);
    }

    let mut dependencies = vec![Vec::new(); module.functions.len()];
    for (caller_index, function) in module.functions.iter().enumerate() {
        let mut dependency_names = BTreeSet::new();
        collect_function_summary_dependencies(function, &mut dependency_names);
        for dependency in dependency_names {
            if let Some(dependency_index) = function_indices.get(dependency.as_str()) {
                dependencies[caller_index].push(*dependency_index);
            }
        }
    }
    dependencies
}

fn collect_function_summary_dependencies(function: &ResourceFunction, out: &mut BTreeSet<String>) {
    for block in &function.blocks {
        collect_ops_summary_dependencies(&block.ops, out);
    }
}

fn collect_ops_summary_dependencies(ops: &[ResourceOp], out: &mut BTreeSet<String>) {
    for op in ops {
        collect_op_summary_dependencies(op, out);
    }
}

fn collect_op_summary_dependencies(op: &ResourceOp, out: &mut BTreeSet<String>) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            ..
        }
        | ResourceOp::FunctionValue { name, .. } => {
            out.insert(name.clone());
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_summary_dependencies(then_ops, out);
            collect_ops_summary_dependencies(else_ops, out);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_summary_dependencies(condition_ops, out);
            collect_ops_summary_dependencies(body_ops, out);
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_summary_dependencies(&arm.ops, out);
            }
        }
        ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::Construct { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeId;

    use super::*;
    use crate::resource::model::{
        Place, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceTerminator,
    };

    #[test]
    fn summary_dependents_cover_nested_calls_function_values_and_self_recursion() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "caller",
                    vec![
                        call("callee"),
                        ResourceOp::Branch {
                            output: place("branch_out"),
                            condition: place("cond"),
                            condition_fact: None,
                            then_ops: vec![function_value("callback")],
                            then_value: place("then_value"),
                            else_ops: vec![],
                            else_value: place("else_value"),
                            span: Span::dummy(),
                        },
                    ],
                ),
                function_with_ops("callee", vec![]),
                function_with_ops("callback", vec![]),
                function_with_ops("recursive", vec![call("recursive")]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let dependents = build_function_summary_dependents(&module);

        assert_eq!(dependents[0], Vec::<usize>::new());
        assert_eq!(dependents[1], vec![0]);
        assert_eq!(dependents[2], vec![0]);
        assert_eq!(dependents[3], vec![3]);
    }

    fn function_with_ops(name: &str, ops: Vec<ResourceOp>) -> ResourceFunction {
        ResourceFunction {
            name: name.to_string(),
            origin_name: name.to_string(),
            type_params: vec![],
            params: vec![],
            result: TypeId(0),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn call(name: &str) -> ResourceOp {
        ResourceOp::Call {
            output: place("call_out"),
            target: ResourceCallTarget::User {
                name: name.to_string(),
                type_args: vec![],
            },
            args: vec![],
            effect: super::super::model::EffectOp::Pure,
            span: Span::dummy(),
        }
    }

    fn function_value(name: &str) -> ResourceOp {
        ResourceOp::FunctionValue {
            output: place("function_value"),
            name: name.to_string(),
            effect: super::super::model::EffectOp::Pure,
            span: Span::dummy(),
        }
    }

    fn place(name: &str) -> Place {
        Place {
            root: PlaceRoot::Local(name.to_string()),
            projections: vec![],
            ty: TypeId(0),
        }
    }
}

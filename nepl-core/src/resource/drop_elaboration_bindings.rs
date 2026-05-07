use alloc::string::String;
use alloc::vec::Vec;

use super::model::{Place, PlaceRoot, ResourceFunction, ResourceOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceDropSourceBinding {
    pub place: Place,
    pub source_name: String,
}

pub(super) fn function_source_bindings(
    function: &ResourceFunction,
) -> Vec<ResourceDropSourceBinding> {
    let mut bindings = function
        .params
        .iter()
        .map(|param| ResourceDropSourceBinding {
            place: param.place.clone(),
            source_name: param.name.clone(),
        })
        .collect::<Vec<_>>();
    for block in &function.blocks {
        collect_source_bindings_from_ops(&block.ops, &mut bindings);
    }
    bindings
}

pub(super) fn source_name_for_place(
    bindings: &[ResourceDropSourceBinding],
    place: &Place,
) -> Option<String> {
    bindings
        .iter()
        .rev()
        .find(|binding| binding.place == *place)
        .map(|binding| binding.source_name.clone())
}

fn collect_source_bindings_from_ops(
    ops: &[ResourceOp],
    bindings: &mut Vec<ResourceDropSourceBinding>,
) {
    for op in ops {
        match op {
            ResourceOp::DeclareLocal {
                place, source_name, ..
            } => bindings.push(ResourceDropSourceBinding {
                place: place.clone(),
                source_name: source_name.clone(),
            }),
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_source_bindings_from_ops(then_ops, bindings);
                collect_source_bindings_from_ops(else_ops, bindings);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_source_bindings_from_ops(condition_ops, bindings);
                collect_source_bindings_from_ops(body_ops, bindings);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    if let Some(place) = &arm.bind_local {
                        let source_name =
                            arm.bind_source_name.as_ref().or_else(|| match &place.root {
                                PlaceRoot::Local(name) => Some(name),
                                PlaceRoot::Temporary(_)
                                | PlaceRoot::I32Constant(_)
                                | PlaceRoot::Return
                                | PlaceRoot::Storage(_)
                                | PlaceRoot::Unknown => None,
                            });
                        if let Some(source_name) = source_name {
                            bindings.push(ResourceDropSourceBinding {
                                place: place.clone(),
                                source_name: source_name.clone(),
                            });
                        }
                    }
                    collect_source_bindings_from_ops(&arm.ops, bindings);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::{EffectOp, Place, ResourceFunction, ResourceModule, ResourceOp};
use super::owner_summary_leaf::owner_leaf_places;
use super::summary_dependency::ResourceSummaryDependencyGraph;

/// Owner obligation の本検査が必要な関数を保守的に抽出する。
///
/// owner checker は所有権を運ぶ型、raw memory boundary、storage origin、non-pure call
/// に対する obligation を検査する。pure scalar helper のようにそれらを全く含まない
/// 関数では、summary も final owner table も空になるため、cache がない cold compile で
/// op 列全体を owner engine に再投入する必要がない。
///
/// この判定は過小評価を避けるため、raw address / storage / indirect call / non-pure call
/// を常に relevant とし、関数 signature または body の place が owner leaf を含む場合も
/// relevant に残す。false になるのは、owner checker が観測できる入力を持たない
/// scalar-only 関数だけである。
pub(super) fn owner_obligation_relevant_functions(
    module: &ResourceModule,
    types: &TypeCtx,
    dependency_graph: &ResourceSummaryDependencyGraph,
) -> Vec<bool> {
    module
        .functions
        .iter()
        .enumerate()
        .map(|(function_index, function)| {
            owner_obligation_relevant_function(types, function, dependency_graph, function_index)
        })
        .collect()
}

fn owner_obligation_relevant_function(
    types: &TypeCtx,
    function: &ResourceFunction,
    dependency_graph: &ResourceSummaryDependencyGraph,
    function_index: usize,
) -> bool {
    function
        .params
        .iter()
        .any(|param| place_has_owner_obligation_leaf(types, &param.place))
        || place_has_owner_obligation_leaf(types, &Place::local("__return".into(), function.result))
        || dependency_graph.has_direct_owner_summary_op(function_index)
        || function
            .blocks
            .iter()
            .any(|block| ops_have_owner_obligation_relevance(types, &block.ops))
}

fn ops_have_owner_obligation_relevance(types: &TypeCtx, ops: &[ResourceOp]) -> bool {
    ops.iter()
        .any(|op| op_has_owner_obligation_relevance(types, op))
}

fn op_has_owner_obligation_relevance(types: &TypeCtx, op: &ResourceOp) -> bool {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            place_has_owner_obligation_leaf(types, place)
                || initializer
                    .as_ref()
                    .is_some_and(|place| place_has_owner_obligation_leaf(types, place))
        }
        ResourceOp::Read { source, output, .. }
        | ResourceOp::Move { source, output, .. }
        | ResourceOp::Borrow { source, output, .. } => {
            place_has_owner_obligation_leaf(types, source)
                || place_has_owner_obligation_leaf(types, output)
        }
        ResourceOp::Assign { target, value, .. } => {
            place_has_owner_obligation_leaf(types, target)
                || place_has_owner_obligation_leaf(types, value)
        }
        ResourceOp::Drop { place, .. } => place_has_owner_obligation_leaf(types, place),
        ResourceOp::EndScope { locals, result, .. } => {
            locals
                .iter()
                .any(|place| place_has_owner_obligation_leaf(types, place))
                || result
                    .as_ref()
                    .is_some_and(|place| place_has_owner_obligation_leaf(types, place))
        }
        ResourceOp::FunctionValue { output, .. } | ResourceOp::Expr { output, .. } => {
            place_has_owner_obligation_leaf(types, output)
        }
        ResourceOp::Call {
            output,
            args,
            effect,
            ..
        } => {
            !matches!(effect, EffectOp::Pure)
                || place_has_owner_obligation_leaf(types, output)
                || args
                    .iter()
                    .any(|arg| place_has_owner_obligation_leaf(types, arg))
        }
        ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. } => true,
        ResourceOp::Construct { output, inputs, .. } => {
            place_has_owner_obligation_leaf(types, output)
                || inputs
                    .iter()
                    .any(|input| place_has_owner_obligation_leaf(types, input))
        }
        ResourceOp::Branch {
            output,
            condition,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            place_has_owner_obligation_leaf(types, output)
                || place_has_owner_obligation_leaf(types, condition)
                || place_has_owner_obligation_leaf(types, then_value)
                || place_has_owner_obligation_leaf(types, else_value)
                || ops_have_owner_obligation_relevance(types, then_ops)
                || ops_have_owner_obligation_relevance(types, else_ops)
        }
        ResourceOp::Loop {
            condition_ops,
            condition,
            body_ops,
            ..
        } => {
            place_has_owner_obligation_leaf(types, condition)
                || ops_have_owner_obligation_relevance(types, condition_ops)
                || ops_have_owner_obligation_relevance(types, body_ops)
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            place_has_owner_obligation_leaf(types, output)
                || place_has_owner_obligation_leaf(types, scrutinee)
                || arms.iter().any(|arm| {
                    arm.bind_local
                        .as_ref()
                        .is_some_and(|place| place_has_owner_obligation_leaf(types, place))
                        || place_has_owner_obligation_leaf(types, &arm.value)
                        || ops_have_owner_obligation_relevance(types, &arm.ops)
                })
        }
        ResourceOp::CallEffect { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => false,
    }
}

fn place_has_owner_obligation_leaf(types: &TypeCtx, place: &Place) -> bool {
    !owner_leaf_places(types, place).is_empty()
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeCtx;

    use super::*;
    use crate::resource::model::{
        RawAddressAliasKind, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceLocal,
        ResourceTerminator,
    };

    #[test]
    fn owner_obligation_relevance_skips_scalar_pure_function() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let module = ResourceModule {
            functions: vec![function(
                "scalar",
                vec![local("value", i32_ty)],
                vec![ResourceOp::Read {
                    source: place("value", i32_ty),
                    output: place("out", i32_ty),
                    span: Span::dummy(),
                }],
                ResourceTerminator::Return {
                    value: Some(place("out", i32_ty)),
                    span: Span::dummy(),
                },
                i32_ty,
            )],
            entry: None,
            string_literals: Vec::new(),
        };
        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(
            owner_obligation_relevant_functions(&module, &types, &graph),
            vec![false]
        );
    }

    #[test]
    fn owner_obligation_relevance_keeps_owner_signature_and_non_pure_call() {
        let types = TypeCtx::new();
        let str_ty = types.str();
        let i32_ty = types.i32();
        let module = ResourceModule {
            functions: vec![
                function(
                    "str_identity",
                    vec![local("value", str_ty)],
                    Vec::new(),
                    ResourceTerminator::Return {
                        value: Some(place("value", str_ty)),
                        span: Span::dummy(),
                    },
                    str_ty,
                ),
                function(
                    "impure_scalar_call",
                    Vec::new(),
                    vec![ResourceOp::Call {
                        output: place("out", i32_ty),
                        target: ResourceCallTarget::User {
                            name: "callee".into(),
                            type_args: Vec::new(),
                        },
                        args: Vec::new(),
                        effect: EffectOp::UserCall {
                            name: "callee".into(),
                            effect: Effect::Impure,
                        },
                        span: Span::dummy(),
                    }],
                    ResourceTerminator::Return {
                        value: Some(place("out", i32_ty)),
                        span: Span::dummy(),
                    },
                    i32_ty,
                ),
            ],
            entry: None,
            string_literals: Vec::new(),
        };
        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(
            owner_obligation_relevant_functions(&module, &types, &graph),
            vec![true, true]
        );
    }

    #[test]
    fn owner_obligation_relevance_keeps_raw_address_alias_without_owner_type_leaf() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let module = ResourceModule {
            functions: vec![function(
                "raw_i32_alias",
                vec![local("raw", i32_ty)],
                vec![ResourceOp::RawAddressAlias {
                    source: place("raw", i32_ty),
                    target: place("raw_alias", i32_ty),
                    kind: RawAddressAliasKind::Transparent,
                    span: Span::dummy(),
                }],
                ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                types.unit(),
            )],
            entry: None,
            string_literals: Vec::new(),
        };
        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(
            owner_obligation_relevant_functions(&module, &types, &graph),
            vec![true]
        );
    }

    #[test]
    fn owner_obligation_relevance_distinguishes_pure_builtin_and_pure_user_calls() {
        let types = TypeCtx::new();
        let i32_ty = types.i32();
        let module = ResourceModule {
            functions: vec![
                function(
                    "pure_builtin_scalar_call",
                    Vec::new(),
                    vec![ResourceOp::Call {
                        output: place("out", i32_ty),
                        target: ResourceCallTarget::Builtin {
                            name: "scalar_builtin".into(),
                        },
                        args: Vec::new(),
                        effect: EffectOp::Pure,
                        span: Span::dummy(),
                    }],
                    ResourceTerminator::Return {
                        value: Some(place("out", i32_ty)),
                        span: Span::dummy(),
                    },
                    i32_ty,
                ),
                function(
                    "pure_user_scalar_call",
                    Vec::new(),
                    vec![ResourceOp::Call {
                        output: place("out", i32_ty),
                        target: ResourceCallTarget::User {
                            name: "callee".into(),
                            type_args: Vec::new(),
                        },
                        args: Vec::new(),
                        effect: EffectOp::UserCall {
                            name: "callee".into(),
                            effect: Effect::Pure,
                        },
                        span: Span::dummy(),
                    }],
                    ResourceTerminator::Return {
                        value: Some(place("out", i32_ty)),
                        span: Span::dummy(),
                    },
                    i32_ty,
                ),
            ],
            entry: None,
            string_literals: Vec::new(),
        };
        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(
            owner_obligation_relevant_functions(&module, &types, &graph),
            vec![false, true]
        );
    }

    fn function(
        name: &str,
        params: Vec<ResourceLocal>,
        ops: Vec<ResourceOp>,
        terminator: ResourceTerminator,
        result: crate::types::TypeId,
    ) -> ResourceFunction {
        ResourceFunction {
            name: name.into(),
            origin_name: name.into(),
            type_params: Vec::new(),
            params,
            result,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator,
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn local(name: &str, ty: crate::types::TypeId) -> ResourceLocal {
        ResourceLocal {
            name: name.into(),
            ty,
            mutable: false,
            place: place(name, ty),
        }
    }

    fn place(name: &str, ty: crate::types::TypeId) -> Place {
        Place::local(String::from(name), ty)
    }
}

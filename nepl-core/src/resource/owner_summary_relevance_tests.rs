use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::TypeCtx;

use super::*;
use crate::resource::model::{
    EffectOp, Place, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceCallTarget,
    ResourceFunction, ResourceLocal, ResourceOp, ResourceTerminator,
};
use crate::resource::summary_dependency::ResourceSummaryDependencyGraph;

#[test]
fn owner_summary_relevance_keeps_owner_carrier_signatures() {
    let types = TypeCtx::new();
    let str_ty = types.str();
    let i32_ty = types.i32();
    let module = ResourceModule {
        functions: vec![
            identity_function("str_id", str_ty),
            identity_function("i32_id", i32_ty),
        ],
        entry: None,
        string_literals: vec![],
    };

    let graph = ResourceSummaryDependencyGraph::build(&module);
    let relevant = owner_summary_relevant_functions(&module, &types, &graph);

    assert_eq!(relevant, vec![true, false]);
}

#[test]
fn owner_summary_relevance_keeps_raw_memory_even_with_scalar_signature() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let output = place("ptr", i32_ty);
    let size = place("size", i32_ty);
    let module = ResourceModule {
        functions: vec![function(
            "alloc_wrapper",
            vec![local("size", i32_ty)],
            vec![ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output,
                args: vec![size],
                span: Span::dummy(),
            }],
            ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            i32_ty,
        )],
        entry: None,
        string_literals: vec![],
    };

    let graph = ResourceSummaryDependencyGraph::build(&module);
    let relevant = owner_summary_relevant_functions(&module, &types, &graph);

    assert_eq!(relevant, vec![true]);
}

#[test]
fn owner_summary_relevance_reuses_nested_dependency_inventory() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let module = ResourceModule {
        functions: vec![
            function(
                "nested_raw_memory",
                vec![],
                vec![ResourceOp::Branch {
                    output: place("branch_out", i32_ty),
                    condition: place("cond", i32_ty),
                    condition_fact: None,
                    then_ops: vec![ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: place("ptr", i32_ty),
                        args: vec![place("size", i32_ty)],
                        span: Span::dummy(),
                    }],
                    then_value: place("then_value", i32_ty),
                    else_ops: vec![],
                    else_value: place("else_value", i32_ty),
                    span: Span::dummy(),
                }],
                ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                i32_ty,
            ),
            function(
                "scalar_impure_call",
                vec![],
                vec![ResourceOp::Call {
                    output: place("out", i32_ty),
                    target: ResourceCallTarget::User {
                        name: "callee".into(),
                        type_args: vec![],
                    },
                    args: vec![],
                    effect: EffectOp::UserCall {
                        name: "callee".into(),
                        effect: Effect::Impure,
                    },
                    span: Span::dummy(),
                }],
                ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                i32_ty,
            ),
            function(
                "scalar_plain",
                vec![],
                vec![],
                ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                i32_ty,
            ),
        ],
        entry: None,
        string_literals: vec![],
    };

    let graph = ResourceSummaryDependencyGraph::build(&module);
    let relevant = owner_summary_relevant_functions(&module, &types, &graph);

    assert_eq!(relevant, vec![true, true, false]);
}

fn identity_function(name: &str, ty: crate::types::TypeId) -> ResourceFunction {
    let input = place("x", ty);
    function(
        name,
        vec![local("x", ty)],
        vec![],
        ResourceTerminator::Return {
            value: Some(input),
            span: Span::dummy(),
        },
        ty,
    )
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

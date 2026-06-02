use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::TypeCtx;

use super::*;
use crate::resource::model::{
    Place, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal,
    ResourceModule, ResourceOp, ResourceTerminator,
};

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

    let relevant = owner_summary_relevant_functions(&module, &types);

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

    let relevant = owner_summary_relevant_functions(&module, &types);

    assert_eq!(relevant, vec![true]);
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

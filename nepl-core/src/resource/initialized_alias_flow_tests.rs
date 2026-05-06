use alloc::string::{String, ToString};
use alloc::vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::*;
use crate::resource::model::{
    EffectOp, Place, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceFunction,
    ResourceLocal, ResourceModule, ResourceOp, ResourceTerminator,
};

#[test]
fn raw_alias_return_summaries_revisit_dependents_when_callee_changes() {
    let mut types = TypeCtx::new();
    let ty = mem_ptr_type(&mut types);
    let module = ResourceModule {
        functions: vec![
            wrapper_function("wrapper", "id", ty),
            identity_function("id", ty),
        ],
        entry: None,
        string_literals: vec![],
    };

    let summaries = compute_raw_cell_address_return_summaries(&module, &types);

    let id = summary(&summaries, "id");
    assert_eq!(id.aliases.len(), 1);
    assert_eq!(id.aliases[0].parameter_index, 0);
    let wrapper = summary(&summaries, "wrapper");
    assert_eq!(wrapper.aliases.len(), 1);
    assert_eq!(wrapper.aliases[0].parameter_index, 0);
}

#[test]
fn raw_alias_return_summaries_do_not_seed_plain_scalar_parameters() {
    let types = TypeCtx::new();
    let ty = types.i32();
    let module = ResourceModule {
        functions: vec![identity_function("id_i32", ty)],
        entry: None,
        string_literals: vec![],
    };

    let summaries = compute_raw_cell_address_return_summaries(&module, &types);

    assert!(summaries.is_empty());
}

fn summary<'a>(
    summaries: &'a [RawCellAddressReturnSummary],
    function: &str,
) -> &'a RawCellAddressReturnSummary {
    summaries
        .iter()
        .find(|summary| summary.function == function)
        .expect("summary should exist")
}

fn identity_function(name: &str, ty: TypeId) -> ResourceFunction {
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

fn wrapper_function(name: &str, callee: &str, ty: TypeId) -> ResourceFunction {
    let input = place("x", ty);
    let output = place("out", ty);
    function(
        name,
        vec![local("x", ty)],
        vec![ResourceOp::Call {
            output: output.clone(),
            target: ResourceCallTarget::User {
                name: callee.to_string(),
                type_args: vec![],
            },
            args: vec![input],
            effect: EffectOp::Pure,
            span: Span::dummy(),
        }],
        ResourceTerminator::Return {
            value: Some(output),
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
    result: TypeId,
) -> ResourceFunction {
    ResourceFunction {
        name: name.to_string(),
        origin_name: name.to_string(),
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

fn local(name: &str, ty: TypeId) -> ResourceLocal {
    ResourceLocal {
        name: name.to_string(),
        ty,
        mutable: false,
        place: place(name, ty),
    }
}

fn place(name: &str, ty: TypeId) -> Place {
    Place::local(String::from(name), ty)
}

fn mem_ptr_type(types: &mut TypeCtx) -> TypeId {
    let raw = types.i32();
    types.register_named(
        String::from("MemPtr"),
        TypeKind::Struct {
            doc: None,
            name: String::from("MemPtr"),
            type_params: vec![],
            fields: vec![raw],
            field_names: vec![String::from("raw")],
        },
    )
}

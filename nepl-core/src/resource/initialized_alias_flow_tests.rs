use alloc::string::{String, ToString};
use alloc::vec;

use crate::ast::Effect;
use crate::span::{FileId, Span};
use crate::types::{NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeId, TypeKind};

use super::*;
use crate::resource::model::{
    EffectOp, Place, PlaceProjection, RawAddressViewKind, ResourceBlock, ResourceBlockId,
    ResourceCallTarget, ResourceFunction, ResourceLocal, ResourceModule, ResourceOffset,
    ResourceOp, ResourceTerminator,
};
use crate::resource::summary_dependency::ResourceSummaryDependencyGraph;
use crate::resource::{ResourceSummaryValueCache, ResourceSummaryValueCacheContext};
use crate::source_map::CompilerMemoryType;

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

#[test]
fn raw_alias_return_summary_value_cache_preseeds_non_empty_and_empty_functions() {
    let mut types = TypeCtx::new();
    let ty = mem_ptr_type(&mut types);
    types.mark_compiler_memory_type(ty, CompilerMemoryType::RawPointer);
    let module = ResourceModule {
        functions: vec![
            wrapper_function("wrapper", "id", ty),
            identity_function("id", ty),
            identity_function("bool_id", types.bool()),
        ],
        entry: None,
        string_literals: vec![],
    };
    let mut cache = ResourceSummaryValueCache::new();
    let mut context = ResourceSummaryValueCacheContext::new(7);
    context.insert_source_policy_hash(FileId(0), 100);
    let dependency_graph = ResourceSummaryDependencyGraph::build(&module);

    let (first_summaries, first_recomputations) =
        compute_raw_cell_address_return_summaries_with_recomputations(
            &module,
            &types,
            &dependency_graph,
            Some(&mut cache),
            Some(&context),
        );
    let (second_summaries, second_recomputations) =
        compute_raw_cell_address_return_summaries_with_recomputations(
            &module,
            &types,
            &dependency_graph,
            Some(&mut cache),
            Some(&context),
        );

    assert!(first_recomputations >= module.functions.len());
    assert_eq!(second_recomputations, 0);
    assert_eq!(
        summary(&first_summaries, "id"),
        summary(&second_summaries, "id")
    );
    assert_eq!(
        summary(&first_summaries, "wrapper"),
        summary(&second_summaries, "wrapper")
    );
    assert!(summary(&second_summaries, "id").aliases.len() == 1);
    assert!(second_summaries
        .iter()
        .all(|summary| summary.function != "bool_id"));

    let stats = cache.stats();
    assert_eq!(stats.resource_summary_value_raw_alias_return_entry_hits, 0);
    assert_eq!(stats.resource_summary_value_replay_hits, 2);
    assert_eq!(
        stats.resource_summary_value_raw_alias_return_entry_recomputed_ops,
        2
    );
}

#[test]
fn raw_alias_return_summaries_widen_recursive_storage_offsets() {
    let mut types = TypeCtx::new();
    let ty = mem_ptr_type(&mut types);
    types.mark_compiler_memory_type(ty, CompilerMemoryType::RawPointer);
    let module = ResourceModule {
        functions: vec![recursive_offset_function(
            "recursive_offset",
            ty,
            types.i32(),
        )],
        entry: None,
        string_literals: vec![],
    };

    let summaries = compute_raw_cell_address_return_summaries(&module, &types);

    let summary = summary(&summaries, "recursive_offset");
    assert!(summary
        .aliases
        .iter()
        .any(|alias| alias.parameter_projection
            == vec![PlaceProjection::StorageOffset(ResourceOffset::Unknown)]
            && alias.return_projection.is_empty()));
    assert!(summary.aliases.iter().all(|alias| alias
        .parameter_projection
        .len()
        .max(alias.return_projection.len())
        <= 2));
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

fn recursive_offset_function(name: &str, ty: TypeId, bool_ty: TypeId) -> ResourceFunction {
    let input = place("x", ty);
    let next = place("next", ty);
    let recursive = place("recursive", ty);
    let output = place("out", ty);
    let offset_input = input
        .clone()
        .with_projection(PlaceProjection::StorageOffset(ResourceOffset::Known(1)), ty);
    function(
        name,
        vec![local("x", ty)],
        vec![ResourceOp::Branch {
            output: output.clone(),
            condition: place("cond", bool_ty),
            condition_fact: None,
            then_ops: vec![ResourceOp::RawAddressView {
                source: offset_input.clone(),
                target: next.clone(),
                kind: RawAddressViewKind::Offset,
                span: Span::dummy(),
            }],
            then_value: next,
            else_ops: vec![
                ResourceOp::RawAddressView {
                    source: offset_input,
                    target: recursive.clone(),
                    kind: RawAddressViewKind::Offset,
                    span: Span::dummy(),
                },
                ResourceOp::Call {
                    output: recursive.clone(),
                    target: ResourceCallTarget::User {
                        name: name.to_string(),
                        type_args: vec![],
                    },
                    args: vec![recursive.clone()],
                    effect: EffectOp::Pure,
                    span: Span::dummy(),
                },
            ],
            else_value: recursive,
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
        type_params: Vec::new(),
        params,
        result,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops,
            terminator,
            span: Span::new(FileId(0), 0, 1),
        }],
        span: Span::new(FileId(0), 0, 1),
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
    types.register_named_with_stable_identity(
        String::from("MemPtr"),
        TypeKind::Struct {
            name: String::from("MemPtr"),
            type_params: vec![],
            fields: vec![raw],
            field_names: vec![String::from("raw")],
        },
        NominalStableTypeIdentity::new(
            NominalStableTypeKind::Struct,
            String::from("/test/mem.nepl"),
            String::from("MemPtr"),
            0,
            1,
        ),
    )
}

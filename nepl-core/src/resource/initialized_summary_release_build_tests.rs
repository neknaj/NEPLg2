use alloc::{string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::effects::RawMemoryOp;
use crate::source_map::CompilerMemoryType;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_summary::RawCellReleaseRequirementKind;
use super::initialized_summary_build::compute_raw_cell_initialization_function_summaries;
use super::model::{
    Place, PlaceProjection, RawAddressViewKind, ResourceBlock, ResourceBlockId, ResourceFunction,
    ResourceId, ResourceLocal, ResourceModule, ResourceOffset, ResourceOp, ResourceTerminator,
};

#[test]
fn release_summary_does_not_seed_plain_string_view_parameters() {
    let types = TypeCtx::new();
    let source = Place::local("source".to_string(), types.str());
    let raw_source = source.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        types.i32(),
    );
    let function = raw_copy_from_view_function("copy_from_str", source, raw_source, types.i32());
    let summaries = compute_raw_cell_initialization_function_summaries(
        &module(function),
        &types,
        &[],
        &[],
        None,
        None,
    );

    assert!(summaries
        .iter()
        .all(|summary| summary.param_release_requirements.is_empty()));
}

#[test]
fn release_summary_keeps_requirements_for_registered_raw_pointer_fields() {
    let mut types = TypeCtx::new();
    let mem_ptr = memory_struct(&mut types, CompilerMemoryType::RawPointer);
    let source = Place::local("source".to_string(), mem_ptr);
    let raw_source = source.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        types.i32(),
    );
    let function =
        raw_copy_from_view_function("copy_from_mem_ptr", source, raw_source, types.i32());
    let summaries = compute_raw_cell_initialization_function_summaries(
        &module(function),
        &types,
        &[],
        &[],
        None,
        None,
    );
    let summary = summaries
        .iter()
        .find(|summary| summary.function == "copy_from_mem_ptr")
        .expect("registered raw pointer source should keep a release summary");

    assert!(summary
        .param_release_requirements
        .iter()
        .any(|requirement| {
            requirement.param_index == 0
                && requirement.kind == RawCellReleaseRequirementKind::BulkSource
                && matches!(
                    requirement.suffix.as_slice(),
                    [PlaceProjection::Field {
                        index: 0,
                        offset_bytes: 0
                    }]
                )
        }));
}

fn raw_copy_from_view_function(
    name: &str,
    source: Place,
    raw_source: Place,
    raw_ty: TypeId,
) -> ResourceFunction {
    let raw_view = Place::temporary(ResourceId(0), raw_ty);
    let raw_destination = Place::temporary(ResourceId(1), raw_ty);
    let output = Place::temporary(ResourceId(2), raw_ty);
    let source_ty = source.ty;
    ResourceFunction {
        name: name.to_string(),
        origin_name: name.to_string(),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: "source".to_string(),
            ty: source_ty,
            mutable: false,
            place: source,
        }],
        result: raw_ty,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::RawAddressView {
                    source: raw_source,
                    target: raw_view.clone(),
                    kind: RawAddressViewKind::NonOwningProjection,
                    span: Span::dummy(),
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::BulkCopy,
                    output: output.clone(),
                    args: vec![raw_destination, raw_view],
                    span: Span::dummy(),
                },
            ],
            terminator: ResourceTerminator::Return {
                value: Some(output),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }
}

fn module(function: ResourceFunction) -> ResourceModule {
    ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: Vec::new(),
    }
}

fn memory_struct(types: &mut TypeCtx, memory_type: CompilerMemoryType) -> TypeId {
    let i32_ty = types.i32();
    let ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    types.mark_compiler_memory_type(ty, memory_type);
    ty
}

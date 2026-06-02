use alloc::{string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::effects::RawMemoryOp;
use crate::source_map::CompilerMemoryType;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellReleaseRequirementKind;
use super::initialized_summary_build::compute_raw_cell_initialization_function_summaries;
use super::initialized_summary_release_build::collect_address_release_requirements;
use super::model::{
    Place, PlaceProjection, RawAddressViewKind, ResourceBlock, ResourceBlockId, ResourceExprKind,
    ResourceFunction, ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern,
    ResourceModule, ResourceOffset, ResourceOp, ResourceTerminator,
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

#[test]
fn branch_release_summary_uses_merged_branch_output_alias() {
    let mut types = TypeCtx::new();
    types.register_copy_impl_target(types.bool());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.unit());
    let mem_ptr = memory_struct(&mut types, CompilerMemoryType::RawPointer);
    let source = Place::local("source".to_string(), mem_ptr);
    let condition = Place::local("condition".to_string(), types.bool());
    let size = Place::local("size".to_string(), types.i32());
    let raw_source = source.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        types.i32(),
    );
    let then_raw = Place::temporary(ResourceId(0), types.i32());
    let else_raw = Place::temporary(ResourceId(1), types.i32());
    let branch_raw = Place::temporary(ResourceId(2), types.i32());
    let raw_release = Place::temporary(ResourceId(3), types.unit());
    let function = ResourceFunction {
        name: "branch_release".to_string(),
        origin_name: "branch_release".to_string(),
        type_params: Vec::new(),
        params: vec![
            ResourceLocal {
                name: "source".to_string(),
                ty: source.ty,
                mutable: false,
                place: source,
            },
            ResourceLocal {
                name: "condition".to_string(),
                ty: condition.ty,
                mutable: false,
                place: condition.clone(),
            },
            ResourceLocal {
                name: "size".to_string(),
                ty: size.ty,
                mutable: false,
                place: size.clone(),
            },
        ],
        result: types.unit(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::Branch {
                    output: branch_raw.clone(),
                    condition,
                    condition_fact: None,
                    then_ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::LiteralI32(0),
                            output: then_raw.clone(),
                            ty: types.i32(),
                            span: Span::dummy(),
                        },
                        ResourceOp::RawAddressView {
                            source: raw_source.clone(),
                            target: then_raw.clone(),
                            kind: RawAddressViewKind::NonOwningProjection,
                            span: Span::dummy(),
                        },
                    ],
                    then_value: then_raw,
                    else_ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::LiteralI32(0),
                            output: else_raw.clone(),
                            ty: types.i32(),
                            span: Span::dummy(),
                        },
                        ResourceOp::RawAddressView {
                            source: raw_source,
                            target: else_raw.clone(),
                            kind: RawAddressViewKind::NonOwningProjection,
                            span: Span::dummy(),
                        },
                    ],
                    else_value: else_raw,
                    span: Span::dummy(),
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: raw_release,
                    args: vec![branch_raw, size],
                    span: Span::dummy(),
                },
            ],
            terminator: ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    };
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
        .find(|summary| summary.function == "branch_release")
        .expect("branch output alias should make the release summary relevant");

    assert!(summary
        .param_release_requirements
        .iter()
        .any(|requirement| {
            requirement.param_index == 0
                && requirement.kind == RawCellReleaseRequirementKind::Dealloc
                && matches!(
                    requirement.suffix.as_slice(),
                    [PlaceProjection::Field {
                        index: 0,
                        offset_bytes: 0
                    }]
                )
        }));
}

#[test]
fn match_release_summary_uses_merged_match_output_alias() {
    let mut types = TypeCtx::new();
    types.register_copy_impl_target(types.bool());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.unit());
    let mem_ptr = memory_struct(&mut types, CompilerMemoryType::RawPointer);
    let source = Place::local("source".to_string(), mem_ptr);
    let scrutinee = Place::local("scrutinee".to_string(), types.bool());
    let size = Place::local("size".to_string(), types.i32());
    let raw_source = source.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        types.i32(),
    );
    let true_raw = Place::temporary(ResourceId(0), types.i32());
    let false_raw = Place::temporary(ResourceId(1), types.i32());
    let match_raw = Place::temporary(ResourceId(2), types.i32());
    let raw_release = Place::temporary(ResourceId(3), types.unit());
    let function = ResourceFunction {
        name: "match_release".to_string(),
        origin_name: "match_release".to_string(),
        type_params: Vec::new(),
        params: vec![
            ResourceLocal {
                name: "source".to_string(),
                ty: source.ty,
                mutable: false,
                place: source,
            },
            ResourceLocal {
                name: "scrutinee".to_string(),
                ty: scrutinee.ty,
                mutable: false,
                place: scrutinee.clone(),
            },
            ResourceLocal {
                name: "size".to_string(),
                ty: size.ty,
                mutable: false,
                place: size.clone(),
            },
        ],
        result: types.unit(),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::Match {
                    output: match_raw.clone(),
                    scrutinee,
                    scrutinee_is_borrow_target: false,
                    arms: vec![
                        ResourceMatchArm {
                            pattern: ResourceMatchPattern::BoolLiteral(true),
                            bind_local: None,
                            bind_source_name: None,
                            bind_mode: None,
                            ops: vec![
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::LiteralI32(0),
                                    output: true_raw.clone(),
                                    ty: types.i32(),
                                    span: Span::dummy(),
                                },
                                ResourceOp::RawAddressView {
                                    source: raw_source.clone(),
                                    target: true_raw.clone(),
                                    kind: RawAddressViewKind::NonOwningProjection,
                                    span: Span::dummy(),
                                },
                            ],
                            value: true_raw,
                            span: Span::dummy(),
                        },
                        ResourceMatchArm {
                            pattern: ResourceMatchPattern::BoolLiteral(false),
                            bind_local: None,
                            bind_source_name: None,
                            bind_mode: None,
                            ops: vec![
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::LiteralI32(0),
                                    output: false_raw.clone(),
                                    ty: types.i32(),
                                    span: Span::dummy(),
                                },
                                ResourceOp::RawAddressView {
                                    source: raw_source,
                                    target: false_raw.clone(),
                                    kind: RawAddressViewKind::NonOwningProjection,
                                    span: Span::dummy(),
                                },
                            ],
                            value: false_raw,
                            span: Span::dummy(),
                        },
                    ],
                    span: Span::dummy(),
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: raw_release,
                    args: vec![match_raw, size],
                    span: Span::dummy(),
                },
            ],
            terminator: ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    };
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
        .find(|summary| summary.function == "match_release")
        .expect("match output alias should make the release summary relevant");

    assert!(summary
        .param_release_requirements
        .iter()
        .any(|requirement| {
            requirement.param_index == 0
                && requirement.kind == RawCellReleaseRequirementKind::Dealloc
                && matches!(
                    requirement.suffix.as_slice(),
                    [PlaceProjection::Field {
                        index: 0,
                        offset_bytes: 0
                    }]
                )
        }));
}

#[test]
fn release_requirement_param_alias_index_deduplicates_equivalent_alias_pairs() {
    let mut types = TypeCtx::new();
    let mem_ptr = memory_struct(&mut types, CompilerMemoryType::RawPointer);
    let param_place = Place::local("source".to_string(), mem_ptr);
    let alias_place = Place::local("source_alias".to_string(), mem_ptr);
    let raw_address = param_place.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        types.i32(),
    );
    let params = vec![ResourceLocal {
        name: "source".to_string(),
        ty: mem_ptr,
        mutable: false,
        place: param_place.clone(),
    }];
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut requirements = Vec::new();

    raw_aliases.copy_explicit_raw_address_alias(&param_place, &alias_place);
    collect_address_release_requirements(
        &mut requirements,
        &types,
        &raw_address,
        RawCellReleaseRequirementKind::Dealloc,
        &raw_aliases,
        &params,
    );

    assert_eq!(
        requirements.len(),
        1,
        "param alias index は同じ param / suffix / kind に畳める alias pair を重複保存しない"
    );
    let requirement = &requirements[0];
    assert_eq!(requirement.param_index, 0);
    assert_eq!(requirement.kind, RawCellReleaseRequirementKind::Dealloc);
    assert!(matches!(
        requirement.suffix.as_slice(),
        [PlaceProjection::Field {
            index: 0,
            offset_bytes: 0
        }]
    ));
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

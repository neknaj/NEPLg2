use alloc::string::{String, ToString};
use alloc::vec;

use crate::source_map::CompilerMemoryType;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::super::model::Place;
use super::*;

fn empty_function(name: &str, params: Vec<(String, TypeId)>, result: TypeId) -> ResourceFunction {
    ResourceFunction {
        name: name.to_string(),
        origin_name: name.to_string(),
        type_params: Vec::new(),
        params: params
            .into_iter()
            .map(|(name, ty)| super::super::model::ResourceLocal {
                name: name.clone(),
                ty,
                mutable: false,
                place: Place::local(name, ty),
            })
            .collect(),
        result,
        effect: crate::ast::Effect::Pure,
        entry_block: super::super::model::ResourceBlockId(0),
        blocks: vec![super::super::model::ResourceBlock {
            id: super::super::model::ResourceBlockId(0),
            ops: Vec::new(),
            terminator: super::super::model::ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }
}

fn collection_storage_marker_calling_function(
    name: &str,
    callee: &str,
    storage_ty: TypeId,
    value_ty: TypeId,
) -> ResourceFunction {
    let mut function = empty_function(name, Vec::new(), value_ty);
    let storage = Place::local("storage".to_string(), storage_ty);
    function.blocks[0]
        .ops
        .push(ResourceOp::CollectionSlotLifecycle {
            target: storage.clone(),
            event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty },
            span: Span::dummy(),
        });
    function.blocks[0].ops.push(ResourceOp::Call {
        output: Place::local("storage_out".to_string(), storage_ty),
        target: ResourceCallTarget::User {
            name: callee.to_string(),
            type_args: Vec::new(),
        },
        args: vec![storage],
        effect: super::super::model::EffectOp::Pure,
        span: Span::dummy(),
    });
    function
}

fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
    types.register_named(
        name.to_string(),
        TypeKind::Struct {
            name: name.to_string(),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    )
}

fn register_region_token(types: &mut TypeCtx) -> TypeId {
    let raw_ty = types.i32();
    let value_ty = types.fresh_var(Some("T".to_string()));
    let region_token_ty = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![value_ty],
            fields: vec![raw_ty, raw_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
    region_token_ty
}

#[test]
fn owner_token_with_non_copy_payload_keeps_summary_for_slot_storage_transfer() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let payload_ty = register_empty_struct(&mut types, "OwnedPayload");
    let region_token = register_region_token(&mut types);
    let storage_ty = types.apply(region_token, vec![payload_ty]);
    let module = ResourceModule {
        functions: vec![
            collection_storage_marker_calling_function(
                "mark_collection_storage",
                "identity_storage",
                storage_ty,
                payload_ty,
            ),
            empty_function(
                "identity_storage",
                vec![("storage".to_string(), storage_ty)],
                storage_ty,
            ),
        ],
        entry: None,
        string_literals: vec![],
    };

    assert_eq!(
        collection_slot_summary_relevant_functions(&module, &types),
        vec![true, true]
    );
}

#[test]
fn owner_token_with_copy_payload_does_not_force_collection_slot_callee() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let u8_ty = types.u8();
    types.register_copy_impl_target(u8_ty);
    let region_token = register_region_token(&mut types);
    let storage_ty = types.apply(region_token, vec![u8_ty]);
    let module = ResourceModule {
        functions: vec![
            collection_storage_marker_calling_function(
                "mark_collection_storage",
                "identity_storage",
                storage_ty,
                u8_ty,
            ),
            empty_function(
                "identity_storage",
                vec![("storage".to_string(), storage_ty)],
                storage_ty,
            ),
        ],
        entry: None,
        string_literals: vec![],
    };

    assert_eq!(
        collection_slot_summary_relevant_functions(&module, &types),
        vec![false, false]
    );
}

#[test]
fn copy_scalar_signature_does_not_force_collection_slot_summary() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let module = ResourceModule {
        functions: vec![empty_function(
            "identity_i32",
            vec![("value".to_string(), types.i32())],
            types.i32(),
        )],
        entry: None,
        string_literals: vec![],
    };

    assert_eq!(
        collection_slot_summary_relevant_functions(&module, &types),
        vec![false]
    );
}

#[test]
fn direct_slot_payload_i32_does_not_make_i32_helper_relevant() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let module = ResourceModule {
        functions: vec![
            collection_storage_marker_calling_function(
                "mark_collection_storage",
                "identity_i32",
                types.i32(),
                types.i32(),
            ),
            empty_function(
                "identity_i32",
                vec![("value".to_string(), types.i32())],
                types.i32(),
            ),
        ],
        entry: None,
        string_literals: vec![],
    };

    assert_eq!(
        collection_slot_summary_relevant_functions(&module, &types),
        vec![false, false]
    );
}

#[test]
fn string_owner_signature_does_not_force_collection_slot_summary() {
    let types = TypeCtx::new();
    let module = ResourceModule {
        functions: vec![empty_function(
            "identity_str",
            vec![("value".to_string(), types.str())],
            types.str(),
        )],
        entry: None,
        string_literals: vec![],
    };

    assert_eq!(
        collection_slot_summary_relevant_functions(&module, &types),
        vec![false]
    );
}

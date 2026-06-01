use nepl_core::ast::Effect;
use nepl_core::types::{NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeKind};

#[test]
fn checkpoint_rolls_back_bindings_and_temporary_arena_entries() {
    let mut ctx = TypeCtx::new();
    let var = ctx.fresh_var(None);
    let checkpoint = ctx.checkpoint();

    let temp = ctx.fresh_var(None);
    let i32_ty = ctx.i32();
    let fn_ty = ctx.function(Vec::new(), vec![temp], i32_ty, Effect::Pure);
    assert!(ctx.unify(var, fn_ty).is_ok());
    assert_ne!(ctx.resolve_id(var), var);

    ctx.rollback(checkpoint);
    assert_eq!(ctx.resolve_id(var), var);

    let next = ctx.fresh_var(None);
    assert_eq!(next, temp);
}

#[test]
fn checkpoint_commit_keeps_bindings_and_closes_snapshot_scope() {
    let mut ctx = TypeCtx::new();
    let var = ctx.fresh_var(None);
    let i32_ty = ctx.i32();
    let checkpoint = ctx.checkpoint();

    assert!(ctx.unify(var, i32_ty).is_ok());
    ctx.commit(checkpoint);

    assert!(ctx.same_type(var, i32_ty));

    let rollback_after_commit = ctx.checkpoint();
    let temp = ctx.fresh_var(None);
    let fn_ty = ctx.function(Vec::new(), vec![temp], i32_ty, Effect::Pure);
    let speculative = ctx.fresh_var(None);
    assert!(ctx.unify(speculative, fn_ty).is_ok());
    ctx.rollback(rollback_after_commit);

    assert!(ctx.same_type(var, i32_ty));
    assert_eq!(ctx.fresh_var(None), temp);
}

#[test]
fn checkpoint_rolls_back_named_and_trait_model_state() {
    let mut ctx = TypeCtx::new();
    let checkpoint = ctx.checkpoint();

    let i32_ty = ctx.i32();
    ctx.register_named(
        "Temporary".to_string(),
        TypeKind::Tuple {
            items: vec![i32_ty],
        },
    );
    ctx.set_copy_trait_enabled(true);
    ctx.register_copy_impl_target(i32_ty);
    ctx.register_drop_impl_target(i32_ty);

    assert!(ctx.lookup_named("Temporary").is_some());
    assert!(ctx.is_copy(i32_ty));
    assert!(ctx.has_drop_impl_target(i32_ty));

    ctx.rollback(checkpoint);

    assert!(ctx.lookup_named("Temporary").is_none());
    assert!(!ctx.is_copy(i32_ty));
    assert!(!ctx.has_drop_impl_target(i32_ty));
}

#[test]
fn checkpoint_rolls_back_nominal_stable_identity_state() {
    let mut ctx = TypeCtx::new();
    let checkpoint = ctx.checkpoint();

    let i32_ty = ctx.i32();
    let temporary = ctx.register_named_with_stable_identity(
        "Temporary".to_string(),
        TypeKind::Struct {
            name: "Temporary".to_string(),
            type_params: Vec::new(),
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
        NominalStableTypeIdentity::new(
            NominalStableTypeKind::Struct,
            "/user/types.nepl".to_string(),
            "Temporary".to_string(),
            0,
            1,
        ),
    );

    assert!(ctx.nominal_stable_identity(temporary).is_some());

    ctx.rollback(checkpoint);

    assert!(ctx.lookup_named("Temporary").is_none());
}

#[test]
fn str_and_i32_do_not_unify() {
    let mut ctx = TypeCtx::new();
    let str_ty = ctx.str();
    let i32_ty = ctx.i32();

    assert!(ctx.unify(str_ty, i32_ty).is_err());
    assert!(ctx.unify(i32_ty, str_ty).is_err());
}

#[test]
fn mutable_references_are_not_copy() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let shared = ctx.reference(i32_ty, false);
    let unique = ctx.reference(i32_ty, true);

    assert!(ctx.is_copy(shared));
    assert!(!ctx.is_copy(unique));
    assert!(ctx.is_copy_eligible(shared));
    assert!(!ctx.is_copy_eligible(unique));

    ctx.set_copy_trait_enabled(true);
    assert!(ctx.is_copy(shared));
    assert!(!ctx.is_copy(unique));
}

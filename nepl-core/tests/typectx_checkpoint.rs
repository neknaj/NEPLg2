use nepl_core::ast::Effect;
use nepl_core::types::{TypeCtx, TypeKind};

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

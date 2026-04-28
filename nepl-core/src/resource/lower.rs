extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::effects::{
    intrinsic_internal_effect, intrinsic_is_raw_memory_effect, raw_callee_is_raw_memory_effect,
    raw_memory_callee_internal_effect, InternalEffect,
};
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirMatchPattern, HirModule,
    HirParam,
};
use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::runtime_helpers::{
    helper_base_name, ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI,
};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{
    AggregateKind, BorrowKind, EffectOp, Place, PlaceProjection, RawBodyKind, RawMemoryOp,
    ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceExprKind, ResourceFunction,
    ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern, ResourceModule,
    ResourceOffset, ResourceOp, ResourceTerminator,
};
use super::place_utils::raw_memory_cell_place;
use super::type_pattern::field_type_matches_result;

pub fn lower_hir_module_skeleton(module: &HirModule) -> ResourceModule {
    let types = TypeCtx::new();
    lower_hir_module(module, &types)
}

pub fn lower_hir_module(module: &HirModule, types: &TypeCtx) -> ResourceModule {
    let env = LoweringEnvironment::new(module, types);
    ResourceModule {
        functions: module
            .functions
            .iter()
            .map(|function| lower_hir_function_skeleton(function, &env))
            .collect(),
        entry: module.entry.clone(),
        string_literals: module.string_literals.clone(),
    }
}

struct LoweringEnvironment<'a> {
    function_effects: BTreeMap<String, Effect>,
    functions: BTreeMap<String, &'a HirFunction>,
    types: &'a TypeCtx,
    string_literals: &'a [String],
}

impl<'a> LoweringEnvironment<'a> {
    fn new(module: &'a HirModule, types: &'a TypeCtx) -> Self {
        let mut function_effects = BTreeMap::new();
        let mut functions = BTreeMap::new();
        for function in &module.functions {
            insert_effect(
                &mut function_effects,
                function.name.clone(),
                function.effect,
            );
            functions.insert(function.name.clone(), function);
        }
        for extern_fn in &module.externs {
            insert_effect(
                &mut function_effects,
                extern_fn.local_name.clone(),
                extern_fn.effect,
            );
        }
        Self {
            function_effects,
            functions,
            types,
            string_literals: &module.string_literals,
        }
    }

    fn function_effect(&self, name: &str) -> Effect {
        self.function_effects
            .get(name)
            .copied()
            .unwrap_or(Effect::Pure)
    }

    fn function(&self, name: &str) -> Option<&'a HirFunction> {
        self.functions.get(name).copied()
    }
}

fn insert_effect(function_effects: &mut BTreeMap<String, Effect>, name: String, effect: Effect) {
    let merged = match (function_effects.get(name.as_str()).copied(), effect) {
        (Some(Effect::Impure), _) | (_, Effect::Impure) => Effect::Impure,
        _ => Effect::Pure,
    };
    function_effects.insert(name, merged);
}

struct LoweringContext {
    next_resource: usize,
    local_scopes: Vec<BTreeMap<String, TypeId>>,
}

impl LoweringContext {
    fn new(params: &[ResourceLocal]) -> Self {
        let mut root_scope = BTreeMap::new();
        for param in params {
            root_scope.insert(param.name.clone(), param.ty);
        }
        Self {
            next_resource: 0,
            local_scopes: alloc::vec![root_scope],
        }
    }

    fn temporary(&mut self, ty: TypeId) -> Place {
        let id = ResourceId(self.next_resource);
        self.next_resource += 1;
        Place::temporary(id, ty)
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        let _ = self.local_scopes.pop();
    }

    fn declare_local(&mut self, name: String, ty: TypeId) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn local_place(&self, name: &str, fallback_ty: TypeId) -> Place {
        let ty = self
            .local_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .unwrap_or(fallback_ty);
        Place::local(String::from(name), ty)
    }

    fn snapshot_locals(&self) -> Vec<BTreeMap<String, TypeId>> {
        self.local_scopes.clone()
    }

    fn restore_locals(&mut self, local_scopes: Vec<BTreeMap<String, TypeId>>) {
        self.local_scopes = local_scopes;
    }
}

fn lower_hir_function_skeleton(
    function: &HirFunction,
    env: &LoweringEnvironment,
) -> ResourceFunction {
    let params = function
        .params
        .iter()
        .map(lower_param_skeleton)
        .collect::<Vec<_>>();
    let mut ctx = LoweringContext::new(&params);
    let mut ops = Vec::new();
    let terminator = match &function.body {
        HirBody::Block(block) => {
            let value = lower_block_skeleton(block, &mut ops, &mut ctx, env);
            ResourceTerminator::Return {
                value: Some(value),
                span: block.span,
            }
        }
        HirBody::Wasm(_) => ResourceTerminator::RawBody {
            kind: RawBodyKind::Wasm,
            span: function.span,
        },
        HirBody::LlvmIr(_) => ResourceTerminator::RawBody {
            kind: RawBodyKind::LlvmIr,
            span: function.span,
        },
    };
    let mut blocks = Vec::new();
    blocks.push(ResourceBlock {
        id: ResourceBlockId(0),
        ops,
        terminator,
        span: function.span,
    });
    ResourceFunction {
        name: function.name.clone(),
        params,
        result: function.result,
        effect: function.effect,
        entry_block: ResourceBlockId(0),
        blocks,
        span: function.span,
    }
}

fn lower_param_skeleton(param: &HirParam) -> ResourceLocal {
    ResourceLocal {
        name: param.name.clone(),
        ty: param.ty,
        mutable: param.mutable,
        place: Place::local(param.name.clone(), param.ty),
    }
}

fn lower_block_skeleton(
    block: &HirBlock,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Place {
    let block_output = ctx.temporary(block.ty);
    ops.push(ResourceOp::Expr {
        kind: ResourceExprKind::Block,
        output: block_output.clone(),
        ty: block.ty,
        span: block.span,
    });
    ctx.push_scope();
    let mut last = block_output;
    for line in &block.lines {
        let value = lower_expr_skeleton(&line.expr, ops, ctx, env);
        if !line.drop_result {
            last = value;
        }
    }
    ctx.pop_scope();
    last
}

fn lower_expr_skeleton(
    expr: &HirExpr,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Place {
    match &expr.kind {
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => push_expr(ops, ResourceExprKind::Literal, expr, ctx),
        HirExprKind::Var(name) => {
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Read {
                source: ctx.local_place(name, expr.ty),
                output: output.clone(),
                span: expr.span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::LocalRead,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::FnValue(name) => {
            let output = ctx.temporary(expr.ty);
            let effect = function_value_effect(name, env);
            ops.push(ResourceOp::FunctionValue {
                output: output.clone(),
                name: name.clone(),
                effect,
                span: expr.span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::FunctionValue,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::Call { callee, args } => {
            if let Some(source) = lower_field_get_call_source(callee, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Read {
                    source,
                    output: output.clone(),
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Call,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
                return output;
            }
            let arg_places = lower_args_skeleton(args, ops, ctx, env);
            let effect = call_effect_skeleton(callee, env);
            ops.push(ResourceOp::CallEffect {
                effect: effect.clone(),
                span: expr.span,
            });
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Call {
                output: output.clone(),
                target: lower_call_target(callee),
                args: arg_places.clone(),
                effect: effect.clone(),
                span: expr.span,
            });
            push_core_mem_wrapper_semantics(
                callee,
                args,
                &arg_places,
                &output,
                ops,
                env,
                expr.span,
            );
            push_user_raw_address_return_semantics(
                callee,
                args,
                &arg_places,
                &output,
                ops,
                env,
                expr.span,
            );
            if let Some(name) = func_ref_base_name(callee) {
                push_named_raw_address_semantics(
                    name,
                    args,
                    &arg_places,
                    &output,
                    ops,
                    env,
                    expr.span,
                );
            }
            if let Some(operation) = raw_memory_op_from_callee(callee) {
                ops.push(ResourceOp::RawMemory {
                    operation,
                    output: output.clone(),
                    args: arg_places,
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Call,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
            }
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::Call,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            let callee = lower_expr_skeleton(callee, ops, ctx, env);
            let arg_places = lower_args_skeleton(args, ops, ctx, env);
            let effect = EffectOp::Unknown {
                reason: String::from("indirect call"),
            };
            ops.push(ResourceOp::CallEffect {
                effect: effect.clone(),
                span: expr.span,
            });
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::IndirectCall {
                output: output.clone(),
                callee,
                params: params.clone(),
                result: *result,
                args: arg_places,
                effect,
                span: expr.span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::IndirectCall,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let condition = lower_expr_skeleton(cond, ops, ctx, env);
            let branch_locals = ctx.snapshot_locals();
            let mut then_ops = Vec::new();
            let then_value = lower_expr_skeleton(then_branch, &mut then_ops, ctx, env);
            ctx.restore_locals(branch_locals.clone());
            let mut else_ops = Vec::new();
            let else_value = lower_expr_skeleton(else_branch, &mut else_ops, ctx, env);
            ctx.restore_locals(branch_locals);
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Branch {
                output: output.clone(),
                condition,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span: expr.span,
            });
            output
        }
        HirExprKind::While { cond, body } => {
            let loop_locals = ctx.snapshot_locals();
            let mut condition_ops = Vec::new();
            let condition = lower_expr_skeleton(cond, &mut condition_ops, ctx, env);
            ctx.restore_locals(loop_locals.clone());
            let mut body_ops = Vec::new();
            lower_expr_skeleton(body, &mut body_ops, ctx, env);
            ctx.restore_locals(loop_locals);
            ops.push(ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Loop, expr, ctx)
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee = lower_expr_skeleton(scrutinee, ops, ctx, env);
            let mut resource_arms = Vec::new();
            let match_locals = ctx.snapshot_locals();
            for arm in arms {
                ctx.restore_locals(match_locals.clone());
                let mut arm_ops = Vec::new();
                let bind_local = arm
                    .bind_local
                    .as_ref()
                    .zip(arm.bind_ty)
                    .map(|(name, ty)| Place::local(name.clone(), ty));
                if let Some(place) = &bind_local {
                    if let super::model::PlaceRoot::Local(name) = &place.root {
                        ctx.declare_local(name.clone(), place.ty);
                    }
                }
                let value = lower_expr_skeleton(&arm.body, &mut arm_ops, ctx, env);
                resource_arms.push(ResourceMatchArm {
                    pattern: lower_match_pattern(&arm.pattern),
                    bind_local,
                    ops: arm_ops,
                    value,
                    span: arm.body.span,
                });
            }
            ctx.restore_locals(match_locals);
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Match {
                output: output.clone(),
                scrutinee,
                arms: resource_arms,
                span: expr.span,
            });
            output
        }
        HirExprKind::EnumConstruct {
            name,
            variant,
            payload,
            ..
        } => {
            let mut inputs = Vec::new();
            if let Some(payload) = payload {
                inputs.push(lower_expr_skeleton(payload, ops, ctx, env));
            }
            push_construct(
                ops,
                AggregateKind::Enum {
                    name: name.clone(),
                    variant: variant.clone(),
                },
                inputs,
                expr,
                ctx,
            )
        }
        HirExprKind::StructConstruct { name, fields, .. } => {
            let inputs = fields
                .iter()
                .map(|field| lower_expr_skeleton(field, ops, ctx, env))
                .collect();
            push_construct(
                ops,
                AggregateKind::Struct { name: name.clone() },
                inputs,
                expr,
                ctx,
            )
        }
        HirExprKind::TupleConstruct { items } => {
            let inputs = items
                .iter()
                .map(|item| lower_expr_skeleton(item, ops, ctx, env))
                .collect();
            push_construct(ops, AggregateKind::Tuple, inputs, expr, ctx)
        }
        HirExprKind::Block(block) => lower_block_skeleton(block, ops, ctx, env),
        HirExprKind::Let {
            name,
            mutable,
            value,
        } => {
            let initializer = lower_expr_skeleton(value, ops, ctx, env);
            let place = Place::local(name.clone(), value.ty);
            ctx.declare_local(name.clone(), value.ty);
            ops.push(ResourceOp::DeclareLocal {
                place: place.clone(),
                mutable: *mutable,
                initializer: Some(initializer),
                span: expr.span,
            });
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::Let,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::Set { name, value } => {
            let value_place = lower_expr_skeleton(value, ops, ctx, env);
            let target = ctx.local_place(name, value.ty);
            ops.push(ResourceOp::Assign {
                target: target.clone(),
                value: value_place,
                span: expr.span,
            });
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::Set,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            if let Some(source) =
                lower_get_field_intrinsic_source(name, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Read {
                    source,
                    output: output.clone(),
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Intrinsic,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
                return output;
            }
            if let Some(source) =
                lower_compiler_field_load_source(name, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Read {
                    source,
                    output: output.clone(),
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Intrinsic,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
                return output;
            }
            let arg_places = lower_args_skeleton(args, ops, ctx, env);
            let raw_operation = raw_memory_op_from_intrinsic(name);
            let internal_effect = intrinsic_internal_effect(name);
            if !matches!(internal_effect, InternalEffect::Pure) {
                ops.push(ResourceOp::CallEffect {
                    effect: resource_effect_from_internal(internal_effect),
                    span: expr.span,
                });
            }
            if let Some(operation) = raw_operation {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::RawMemory {
                    operation,
                    output: output.clone(),
                    args: arg_places,
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Intrinsic,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
                output
            } else {
                let output = push_expr(ops, ResourceExprKind::Intrinsic, expr, ctx);
                push_named_raw_address_semantics(
                    helper_base_name(name),
                    args,
                    &arg_places,
                    &output,
                    ops,
                    env,
                    expr.span,
                );
                output
            }
        }
        HirExprKind::AddrOf(inner) => {
            let mut source = place_from_expr_skeleton(inner, ctx);
            if matches!(&source.root, super::model::PlaceRoot::Unknown) {
                source = lower_expr_skeleton(inner, ops, ctx, env);
            }
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Borrow {
                source,
                output: output.clone(),
                kind: BorrowKind::Shared,
                span: expr.span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::Borrow,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::Deref(inner) => {
            let mut source = place_from_expr_skeleton(inner, ctx);
            if matches!(&source.root, super::model::PlaceRoot::Unknown) {
                source = lower_expr_skeleton(inner, ops, ctx, env);
            }
            let source = source.with_projection(super::model::PlaceProjection::Deref, expr.ty);
            let output = ctx.temporary(expr.ty);
            ops.push(ResourceOp::Read {
                source,
                output: output.clone(),
                span: expr.span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::Deref,
                output: output.clone(),
                ty: expr.ty,
                span: expr.span,
            });
            output
        }
        HirExprKind::Drop { name } => {
            ops.push(ResourceOp::Drop {
                place: ctx.local_place(name, expr.ty),
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Drop, expr, ctx)
        }
    }
}

fn push_expr(
    ops: &mut Vec<ResourceOp>,
    kind: ResourceExprKind,
    expr: &HirExpr,
    ctx: &mut LoweringContext,
) -> Place {
    let output = ctx.temporary(expr.ty);
    ops.push(ResourceOp::Expr {
        kind,
        output: output.clone(),
        ty: expr.ty,
        span: expr.span,
    });
    output
}

fn push_construct(
    ops: &mut Vec<ResourceOp>,
    kind: AggregateKind,
    inputs: Vec<Place>,
    expr: &HirExpr,
    ctx: &mut LoweringContext,
) -> Place {
    let output = ctx.temporary(expr.ty);
    ops.push(ResourceOp::Construct {
        output: output.clone(),
        kind,
        inputs,
        span: expr.span,
    });
    output
}

fn lower_args_skeleton(
    args: &[HirExpr],
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Vec<Place> {
    args.iter()
        .map(|arg| lower_expr_skeleton(arg, ops, ctx, env))
        .collect()
}

fn lower_match_pattern(pattern: &HirMatchPattern) -> ResourceMatchPattern {
    match pattern {
        HirMatchPattern::Variant(name) => ResourceMatchPattern::Variant(name.clone()),
        HirMatchPattern::IntLiteral(value) => ResourceMatchPattern::IntLiteral(*value),
        HirMatchPattern::BoolLiteral(value) => ResourceMatchPattern::BoolLiteral(*value),
        HirMatchPattern::Wildcard => ResourceMatchPattern::Wildcard,
    }
}

fn call_effect_skeleton(callee: &FuncRef, env: &LoweringEnvironment) -> EffectOp {
    match callee {
        FuncRef::Builtin(name) => {
            if let Some(effect) = raw_memory_callee_internal_effect(name.as_str()) {
                resource_effect_from_internal(effect)
            } else {
                EffectOp::UserCall {
                    name: name.clone(),
                    effect: Effect::Pure,
                }
            }
        }
        FuncRef::User(name, _, _) => {
            if let Some(effect) = raw_memory_callee_internal_effect(name.as_str()) {
                resource_effect_from_internal(effect)
            } else {
                EffectOp::UserCall {
                    name: name.clone(),
                    effect: env.function_effect(name),
                }
            }
        }
        FuncRef::Trait {
            trait_name, method, ..
        } => EffectOp::UserCall {
            name: alloc::format!("{}::{}", trait_name, method),
            effect: Effect::Pure,
        },
    }
}

fn function_value_effect(name: &str, env: &LoweringEnvironment) -> EffectOp {
    if let Some(effect) = raw_memory_callee_internal_effect(name) {
        resource_effect_from_internal(effect)
    } else {
        EffectOp::UserCall {
            name: String::from(name),
            effect: env.function_effect(name),
        }
    }
}

fn resource_effect_from_internal(effect: InternalEffect) -> EffectOp {
    match effect {
        InternalEffect::Pure => EffectOp::Pure,
        InternalEffect::InternalAlloc { .. } => EffectOp::InternalAlloc,
        InternalEffect::UnsafeMemory { operation } => EffectOp::UnsafeMemory { operation },
        InternalEffect::ExternalIo { operation } => EffectOp::ExternalIo { operation },
        InternalEffect::Nondet { operation } => EffectOp::Nondet { operation },
    }
}

fn lower_call_target(callee: &FuncRef) -> ResourceCallTarget {
    match callee {
        FuncRef::Builtin(name) => ResourceCallTarget::Builtin { name: name.clone() },
        FuncRef::User(name, type_args, _) => ResourceCallTarget::User {
            name: name.clone(),
            type_args: type_args.clone(),
        },
        FuncRef::Trait {
            trait_name,
            trait_args,
            method,
            self_ty,
        } => ResourceCallTarget::Trait {
            trait_name: trait_name.clone(),
            trait_args: trait_args.clone(),
            method: method.clone(),
            self_ty: *self_ty,
        },
    }
}

fn push_core_mem_wrapper_semantics(
    callee: &FuncRef,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: crate::span::Span,
) {
    match func_ref_base_name(callee) {
        Some("mem_ptr_wrap") => {
            let Some(raw) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: raw.clone(),
                target: mem_ptr_raw_field_place(output, env.types.i32()),
                span,
            });
        }
        Some("mem_ptr_addr") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(ptr, output.ty),
                target: output.clone(),
                span,
            });
        }
        Some("mem_ptr_add") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            let mut raw = mem_ptr_raw_field_place(ptr, env.types.i32());
            match hir_args.get(1).and_then(non_negative_i32_literal_bytes) {
                Some(0) => {}
                Some(bytes) => {
                    raw = raw.with_projection(
                        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(bytes) }),
                        env.types.i32(),
                    );
                }
                None => {
                    raw = raw.with_projection(
                        PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                        env.types.i32(),
                    );
                }
            }
            ops.push(ResourceOp::RawAddressAlias {
                source: raw,
                target: mem_ptr_raw_field_place(output, env.types.i32()),
                span,
            });
        }
        Some("region_new") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(ptr, env.types.i32()),
                target: region_token_raw_field_place(output, env.types.i32()),
                span,
            });
        }
        _ => {}
    }
}

fn push_user_raw_address_return_semantics(
    callee: &FuncRef,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: crate::span::Span,
) {
    let FuncRef::User(name, _, _) = callee else {
        return;
    };
    let Some(function) = env.function(name) else {
        return;
    };
    if function.params.len() != hir_args.len() || hir_args.len() != arg_places.len() {
        return;
    }
    let Some(return_expr) = function_return_expr(function) else {
        return;
    };
    let Some(source) =
        raw_address_source_from_return_expr(return_expr, function, hir_args, arg_places, env)
    else {
        return;
    };
    ops.push(ResourceOp::RawAddressAlias {
        source: source.place(env.types.i32()),
        target: raw_address_alias_target(output, env),
        span,
    });
}

fn push_named_raw_address_semantics(
    name: &str,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: crate::span::Span,
) {
    let Some(source) = raw_address_source_from_actual_named_expr(name, hir_args, arg_places, env)
    else {
        return;
    };
    ops.push(ResourceOp::RawAddressAlias {
        source: source.place(env.types.i32()),
        target: raw_address_alias_target(output, env),
        span,
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawAddressSource {
    base: Place,
    offset: RawAddressOffset,
}

impl RawAddressSource {
    fn place(self, raw_ty: TypeId) -> Place {
        match self.offset {
            RawAddressOffset::Known(0) => self.base,
            RawAddressOffset::Known(bytes) if bytes > 0 => match usize::try_from(bytes) {
                Ok(bytes) => self.base.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(bytes) }),
                    raw_ty,
                ),
                Err(_) => self.base.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                    raw_ty,
                ),
            },
            RawAddressOffset::Known(_) | RawAddressOffset::Unknown => self.base.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                raw_ty,
            ),
        }
    }

    fn with_added_offset(mut self, offset: Option<i64>) -> Self {
        self.offset = self.offset.add(offset);
        self
    }

    fn with_subtracted_offset(mut self, offset: Option<i64>) -> Self {
        self.offset = self.offset.sub(offset);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawAddressOffset {
    Known(i64),
    Unknown,
}

impl RawAddressOffset {
    fn add(self, rhs: Option<i64>) -> Self {
        match (self, rhs) {
            (RawAddressOffset::Known(lhs), Some(rhs)) => lhs
                .checked_add(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }

    fn sub(self, rhs: Option<i64>) -> Self {
        match (self, rhs) {
            (RawAddressOffset::Known(lhs), Some(rhs)) => lhs
                .checked_sub(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }
}

fn function_return_expr(function: &HirFunction) -> Option<&HirExpr> {
    let HirBody::Block(block) = &function.body else {
        return None;
    };
    block
        .lines
        .iter()
        .rev()
        .find(|line| !line.drop_result)
        .map(|line| &line.expr)
}

fn raw_address_source_from_return_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            Some(RawAddressSource {
                base: raw_address_place_from_argument(arg_places.get(index)?, env),
                offset: RawAddressOffset::Known(0),
            })
        }
        HirExprKind::Call { callee, args } => raw_address_source_from_named_call(
            func_ref_base_name(callee)?,
            args,
            expr.ty,
            function,
            hir_args,
            arg_places,
            env,
        ),
        HirExprKind::Intrinsic { name, args, .. } => raw_address_source_from_named_call(
            helper_base_name(name),
            args,
            expr.ty,
            function,
            hir_args,
            arg_places,
            env,
        ),
        HirExprKind::StructConstruct { name, fields, .. } if name == "MemPtr" => {
            raw_address_source_from_return_expr(
                fields.first()?,
                function,
                hir_args,
                arg_places,
                env,
            )
        }
        _ => None,
    }
}

fn raw_address_source_from_named_call(
    name: &str,
    args: &[HirExpr],
    return_ty: TypeId,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match name {
        "add" if args.len() == 2 => {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
                .map(|source| {
                    source.with_added_offset(i32_const_from_return_expr(
                        &args[1], function, hir_args, env,
                    ))
                })
                .or_else(|| {
                    let offset = i32_const_from_return_expr(&args[0], function, hir_args, env)?;
                    raw_address_source_from_return_expr(
                        &args[1], function, hir_args, arg_places, env,
                    )
                    .map(|source| source.with_added_offset(Some(offset)))
                })
        }
        "sub" if args.len() == 2 => raw_address_source_from_return_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(|source| {
            source.with_subtracted_offset(i32_const_from_return_expr(
                &args[1], function, hir_args, env,
            ))
        }),
        "mem_ptr_addr" if args.len() == 1 => {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
        }
        "mem_ptr_wrap" if args.len() == 1 => {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
        }
        "mem_ptr_add" if args.len() >= 2 => raw_address_source_from_return_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(|source| {
            source.with_added_offset(i32_const_from_return_expr(
                &args[1], function, hir_args, env,
            ))
        }),
        "get" | "get_field"
            if args.len() >= 2
                && literal_field_name(env, &args[1]) == Some("raw")
                && is_named_struct_type(env.types, args[0].ty, "MemPtr") =>
        {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
        }
        "get" | "get_field"
            if args.len() >= 2
                && is_named_struct_type(env.types, args[0].ty, "RegionToken")
                && is_named_struct_type(env.types, return_ty, "MemPtr")
                && literal_field_name(env, &args[1])
                    .map(|field_name| field_name == "ptr")
                    .unwrap_or(true) =>
        {
            raw_address_source_from_region_token_ptr_expr(
                &args[0], return_ty, function, hir_args, arg_places, env,
            )
        }
        other
            if matches!(raw_memory_op_from_name(other), Some(RawMemoryOp::Load))
                && args.len() == 1
                && is_named_struct_type(env.types, args[0].ty, "RegionToken")
                && is_named_struct_type(env.types, return_ty, "MemPtr") =>
        {
            raw_address_source_from_region_token_ptr_expr(
                &args[0], return_ty, function, hir_args, arg_places, env,
            )
        }
        "str_addr" if args.len() == 1 => {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
        }
        "region_new" if args.len() >= 2 => {
            raw_address_source_from_return_expr(&args[0], function, hir_args, arg_places, env)
        }
        _ => None,
    }
}

fn i32_const_from_return_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            i32_const_from_actual_arg(hir_args.get(index)?, env)
        }
        HirExprKind::Call { callee, args } => i32_const_from_return_named_expr(
            func_ref_base_name(callee)?,
            args,
            function,
            hir_args,
            env,
        )
        .or_else(|| i32_const_from_size_of_call(callee, env)),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                i32_const_from_return_named_expr(
                    helper_base_name(name),
                    args,
                    function,
                    hir_args,
                    env,
                )
            }
        }
        _ => None,
    }
}

fn raw_address_source_from_region_token_ptr_expr(
    expr: &HirExpr,
    _ptr_ty: TypeId,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            let token = arg_places.get(index)?;
            Some(RawAddressSource {
                base: region_token_raw_field_place(token, env.types.i32()),
                offset: RawAddressOffset::Known(0),
            })
        }
        HirExprKind::Call { callee, args }
            if matches!(func_ref_base_name(callee), Some("region_new")) =>
        {
            raw_address_source_from_return_expr(args.first()?, function, hir_args, arg_places, env)
        }
        HirExprKind::Intrinsic { name, args, .. } if helper_base_name(name) == "region_new" => {
            raw_address_source_from_return_expr(args.first()?, function, hir_args, arg_places, env)
        }
        _ => None,
    }
}

fn i32_const_from_actual_arg(expr: &HirExpr, env: &LoweringEnvironment) -> Option<i64> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        HirExprKind::Call { callee, args } => {
            i32_const_from_actual_named_expr(func_ref_base_name(callee)?, args, env)
                .or_else(|| i32_const_from_size_of_call(callee, env))
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                i32_const_from_actual_named_expr(helper_base_name(name), args, env)
            }
        }
        _ => None,
    }
}

fn raw_address_source_from_actual_named_expr(
    name: &str,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match helper_base_name(name) {
        "add" if args.len() == 2 && arg_places.len() == 2 => {
            if i32_const_from_actual_arg(&args[0], env).is_some()
                && i32_const_from_actual_arg(&args[1], env).is_none()
            {
                raw_address_source_from_actual_arg(1, arg_places, env).map(|source| {
                    source.with_added_offset(i32_const_from_actual_arg(&args[0], env))
                })
            } else {
                raw_address_source_from_actual_arg(0, arg_places, env).map(|source| {
                    source.with_added_offset(i32_const_from_actual_arg(&args[1], env))
                })
            }
        }
        "sub" if args.len() == 2 && arg_places.len() == 2 => {
            raw_address_source_from_actual_arg(0, arg_places, env).map(|source| {
                source.with_subtracted_offset(i32_const_from_actual_arg(&args[1], env))
            })
        }
        "mem_ptr_addr" | "mem_ptr_wrap" | "str_addr"
            if args.len() == 1 && arg_places.len() == 1 =>
        {
            raw_address_source_from_actual_arg(0, arg_places, env)
        }
        "mem_ptr_add" if args.len() >= 2 && arg_places.len() >= 2 => {
            raw_address_source_from_actual_arg(0, arg_places, env)
                .map(|source| source.with_added_offset(i32_const_from_actual_arg(&args[1], env)))
        }
        "region_new" if args.len() >= 2 && !arg_places.is_empty() => {
            raw_address_source_from_actual_arg(0, arg_places, env)
        }
        _ => None,
    }
}

fn raw_address_source_from_actual_arg(
    index: usize,
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    Some(RawAddressSource {
        base: raw_address_place_from_argument(arg_places.get(index)?, env),
        offset: RawAddressOffset::Known(0),
    })
}

fn i32_const_from_return_named_expr(
    name: &str,
    args: &[HirExpr],
    function: &HirFunction,
    hir_args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    if args.len() != 2 {
        return None;
    }
    match name {
        "add" => i32_const_from_return_expr(&args[0], function, hir_args, env)?.checked_add(
            i32_const_from_return_expr(&args[1], function, hir_args, env)?,
        ),
        "sub" => i32_const_from_return_expr(&args[0], function, hir_args, env)?.checked_sub(
            i32_const_from_return_expr(&args[1], function, hir_args, env)?,
        ),
        "mul" => {
            let left = i32_const_from_return_expr(&args[0], function, hir_args, env);
            if matches!(left, Some(0)) {
                return Some(0);
            }
            let right = i32_const_from_return_expr(&args[1], function, hir_args, env);
            if matches!(right, Some(0)) {
                return Some(0);
            }
            left?.checked_mul(right?)
        }
        _ => None,
    }
}

fn i32_const_from_actual_named_expr(
    name: &str,
    args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    if args.len() != 2 {
        return None;
    }
    match name {
        "add" => i32_const_from_actual_arg(&args[0], env)?
            .checked_add(i32_const_from_actual_arg(&args[1], env)?),
        "sub" => i32_const_from_actual_arg(&args[0], env)?
            .checked_sub(i32_const_from_actual_arg(&args[1], env)?),
        "mul" => {
            let left = i32_const_from_actual_arg(&args[0], env);
            if matches!(left, Some(0)) {
                return Some(0);
            }
            let right = i32_const_from_actual_arg(&args[1], env);
            if matches!(right, Some(0)) {
                return Some(0);
            }
            left?.checked_mul(right?)
        }
        _ => None,
    }
}

fn i32_const_from_size_of_call(callee: &FuncRef, env: &LoweringEnvironment) -> Option<i64> {
    match callee {
        FuncRef::User(name, type_args, _)
            if helper_base_name(name) == "size_of" && type_args.len() == 1 =>
        {
            i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
        }
        FuncRef::User(name, _, _) if helper_base_name(name) == "size_of" => {
            let function = env.function(name)?;
            let HirBody::Block(block) = &function.body else {
                return None;
            };
            if block.lines.len() != 1 {
                return None;
            }
            let HirExprKind::Intrinsic {
                name, type_args, ..
            } = &block.lines[0].expr.kind
            else {
                return None;
            };
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn function_param_index(function: &HirFunction, name: &str) -> Option<usize> {
    function.params.iter().position(|param| param.name == name)
}

fn raw_address_place_from_argument(place: &Place, env: &LoweringEnvironment) -> Place {
    if is_named_struct_type(env.types, place.ty, "MemPtr") {
        mem_ptr_raw_field_place(place, env.types.i32())
    } else {
        place.clone()
    }
}

fn raw_address_alias_target(output: &Place, env: &LoweringEnvironment) -> Place {
    if is_named_struct_type(env.types, output.ty, "MemPtr") {
        mem_ptr_raw_field_place(output, env.types.i32())
    } else if is_named_struct_type(env.types, output.ty, "RegionToken") {
        region_token_raw_field_place(output, env.types.i32())
    } else {
        output.clone()
    }
}

fn is_named_struct_type(types: &TypeCtx, ty: TypeId, expected: &str) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == expected,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == expected)
        }
        _ => false,
    }
}

fn mem_ptr_raw_field_place(ptr: &Place, raw_ty: TypeId) -> Place {
    ptr.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        raw_ty,
    )
}

fn region_token_ptr_field_place(token: &Place, ptr_ty: TypeId) -> Place {
    token.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        ptr_ty,
    )
}

fn region_token_raw_field_place(token: &Place, raw_ty: TypeId) -> Place {
    mem_ptr_raw_field_place(&region_token_ptr_field_place(token, token.ty), raw_ty)
}

fn non_negative_i32_literal_bytes(expr: &HirExpr) -> Option<usize> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        _ => None,
    }
}

fn raw_memory_op_from_callee(callee: &FuncRef) -> Option<RawMemoryOp> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => raw_memory_op_from_name(name),
        FuncRef::Trait { .. } => None,
    }
}

fn raw_memory_op_from_intrinsic(name: &str) -> Option<RawMemoryOp> {
    if intrinsic_is_raw_memory_effect(name) {
        raw_memory_op_from_name(name)
    } else {
        None
    }
}

fn raw_memory_op_from_name(name: &str) -> Option<RawMemoryOp> {
    if !raw_callee_is_raw_memory_effect(name) && !intrinsic_is_raw_memory_effect(name) {
        return None;
    }
    let base = helper_base_name(name);
    let operation = match base {
        ALLOC_RUNTIME_ABI | "alloc_raw" | "alloc" => RawMemoryOp::Alloc,
        DEALLOC_RUNTIME_ABI | "dealloc_raw" | "dealloc" => RawMemoryOp::Dealloc,
        REALLOC_RUNTIME_ABI | "realloc_raw" | "realloc" => RawMemoryOp::Realloc,
        "load" => RawMemoryOp::Load,
        "store" => RawMemoryOp::Store,
        "mem_copy" => RawMemoryOp::BulkCopy,
        "mem_move" => RawMemoryOp::BulkMove,
        "mem_size" => RawMemoryOp::MemorySize,
        "mem_grow" => RawMemoryOp::MemoryGrow,
        "mem_fill" => RawMemoryOp::Fill,
        other if other.starts_with("load_") => RawMemoryOp::Load,
        other if other.starts_with("store_") => RawMemoryOp::Store,
        other => RawMemoryOp::Other {
            name: String::from(other),
        },
    };
    Some(operation)
}

fn lower_compiler_field_load_source(
    name: &str,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if !matches!(raw_memory_op_from_name(name), Some(RawMemoryOp::Load)) {
        return None;
    }
    let address = args.first()?;
    let (base_expr, offset_bytes) = compiler_field_address_base_and_offset(address)?;
    let projection = aggregate_field_projection(env.types, base_expr.ty, offset_bytes, field_ty)?;
    if let Some(source) =
        lower_raw_aggregate_field_source(base_expr, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(base_expr, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(base_expr, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

fn lower_field_get_call_source(
    callee: &FuncRef,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if func_ref_base_name(callee)? != "get" {
        return None;
    }
    let owner = args.first()?;
    let field_name = literal_field_name(env, args.get(1)?)?;
    let projection = aggregate_field_projection_by_name(env.types, owner.ty, field_name, field_ty)?;
    if let Some(source) =
        lower_raw_aggregate_field_source(owner, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(owner, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

fn lower_get_field_intrinsic_source(
    name: &str,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if helper_base_name(name) != "get_field" {
        return None;
    }
    let owner = args.first()?;
    let projection =
        if let Some(field_name) = args.get(1).and_then(|arg| literal_field_name(env, arg)) {
            aggregate_field_projection_by_name(env.types, owner.ty, field_name, field_ty)?
        } else if is_named_struct_type(env.types, owner.ty, "RegionToken")
            && is_named_struct_type(env.types, field_ty, "MemPtr")
        {
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }
        } else {
            return None;
        };
    if let Some(source) =
        lower_raw_aggregate_field_source(owner, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(owner, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

fn func_ref_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

fn literal_field_name<'a>(env: &'a LoweringEnvironment, expr: &HirExpr) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => {
            env.string_literals.get(*index as usize).map(String::as_str)
        }
        _ => None,
    }
}

fn lower_raw_aggregate_field_source(
    base_expr: &HirExpr,
    projection: PlaceProjection,
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    let address = raw_load_address_expr(base_expr)?;
    let mut address_place = place_from_expr_skeleton(address, ctx);
    if matches!(&address_place.root, super::model::PlaceRoot::Unknown) {
        address_place = lower_expr_skeleton(address, ops, ctx, env);
    }
    Some(raw_memory_cell_place(&address_place, base_expr.ty).with_projection(projection, field_ty))
}

fn raw_load_address_expr(expr: &HirExpr) -> Option<&HirExpr> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. }
            if matches!(raw_memory_op_from_intrinsic(name), Some(RawMemoryOp::Load)) =>
        {
            args.first()
        }
        HirExprKind::Call { callee, args }
            if matches!(raw_memory_op_from_callee(callee), Some(RawMemoryOp::Load)) =>
        {
            args.first()
        }
        _ => None,
    }
}

fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() == 2 => {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if matches!(func_ref_base_name(callee), Some("add")) && args.len() == 2 =>
        {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

fn aggregate_field_projection(
    types: &TypeCtx,
    owner_ty: TypeId,
    offset_bytes: usize,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    let fields = aggregate_fields_with_offsets(types, owner_ty);
    let (index, _) = fields.iter().enumerate().find(|(_, field)| {
        field.offset == offset_bytes && field_type_matches_result(types, field.ty, field_ty)
    })?;
    Some(match kind {
        AggregateProjectionKind::Struct => PlaceProjection::Field {
            index,
            offset_bytes,
        },
        AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
            index,
            offset_bytes,
        },
    })
}

fn aggregate_field_projection_by_name(
    types: &TypeCtx,
    owner_ty: TypeId,
    field_name: &str,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    let fields = aggregate_fields_with_offsets(types, owner_ty);
    let index = match kind {
        AggregateProjectionKind::Struct => aggregate_struct_field_names(types, owner_ty)?
            .iter()
            .position(|name| name == field_name)?,
        AggregateProjectionKind::Tuple => field_name.parse::<usize>().ok()?,
    };
    let field = fields.get(index)?;
    if !field_type_matches_result(types, field.ty, field_ty) {
        return None;
    }
    Some(match kind {
        AggregateProjectionKind::Struct => PlaceProjection::Field {
            index,
            offset_bytes: field.offset,
        },
        AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
            index,
            offset_bytes: field.offset,
        },
    })
}

fn aggregate_struct_field_names(types: &TypeCtx, ty: TypeId) -> Option<&Vec<String>> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => Some(field_names),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => Some(field_names),
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AggregateProjectionKind {
    Struct,
    Tuple,
}

fn aggregate_projection_kind(types: &TypeCtx, ty: TypeId) -> Option<AggregateProjectionKind> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } => Some(AggregateProjectionKind::Struct),
        TypeKind::Tuple { .. } => Some(AggregateProjectionKind::Tuple),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { .. } => Some(AggregateProjectionKind::Struct),
                TypeKind::Tuple { .. } => Some(AggregateProjectionKind::Tuple),
                _ => None,
            }
        }
        _ => None,
    }
}

fn place_from_expr_skeleton(expr: &HirExpr, ctx: &LoweringContext) -> Place {
    match &expr.kind {
        HirExprKind::Var(name) => ctx.local_place(name, expr.ty),
        HirExprKind::Deref(inner) => {
            let source = place_from_expr_skeleton(inner, ctx);
            if matches!(&source.root, super::model::PlaceRoot::Unknown) {
                Place::unknown(expr.ty)
            } else {
                source.with_projection(super::model::PlaceProjection::Deref, expr.ty)
            }
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            let source = place_from_expr_skeleton(&args[0], ctx);
            if matches!(&source.root, super::model::PlaceRoot::Unknown) {
                return Place::unknown(expr.ty);
            }
            let bytes = args.get(1).and_then(|offset| match &offset.kind {
                HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
                _ => None,
            });
            source.with_projection(
                super::model::PlaceProjection::StorageOffset(super::model::ResourceOffset {
                    bytes,
                }),
                expr.ty,
            )
        }
        _ => Place::unknown(expr.ty),
    }
}

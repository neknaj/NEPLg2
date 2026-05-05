extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::effects::{intrinsic_internal_effect, raw_callee_internal_effect, InternalEffect};
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirMatchPattern, HirModule,
    HirParam,
};
use crate::layout::aggregate_fields_with_offsets;
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::lower_condition::{place_from_expr_skeleton, resource_condition_fact};
use super::lower_raw_address::{
    is_named_struct_type, push_core_mem_wrapper_semantics, push_named_raw_address_semantics,
    push_user_raw_address_return_semantics,
};
use super::lower_raw_memory::{
    raw_memory_op_from_callee, raw_memory_op_from_intrinsic, raw_memory_op_from_name,
};
use super::model::{
    AggregateKind, BorrowKind, EffectOp, Place, PlaceProjection, RawBodyKind, RawMemoryOp,
    ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceExprKind, ResourceFunction,
    ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern, ResourceModule, ResourceOp,
    ResourceTerminator,
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

pub(super) struct LoweringEnvironment<'a> {
    function_effects: BTreeMap<String, Effect>,
    functions: BTreeMap<String, &'a HirFunction>,
    pub(super) types: &'a TypeCtx,
    pub(super) string_literals: &'a [String],
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

    pub(super) fn function(&self, name: &str) -> Option<&'a HirFunction> {
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

pub(super) struct LoweringContext {
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

    pub(super) fn local_place(&self, name: &str, fallback_ty: TypeId) -> Place {
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
    if matches!(expr.kind, HirExprKind::Call { .. }) {
        if let Some(output) = try_lower_simple_direct_call_tree(expr, ops, ctx, env) {
            return output;
        }
    }
    match &expr.kind {
        HirExprKind::LiteralI32(value) => {
            push_expr(ops, ResourceExprKind::LiteralI32(*value), expr, ctx)
        }
        HirExprKind::LiteralF32(_)
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
            push_direct_call_skeleton(expr, callee, args, arg_places, ops, ctx, env)
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            effect,
            args,
        } => {
            let callee = lower_expr_skeleton(callee, ops, ctx, env);
            let arg_places = lower_args_skeleton(args, ops, ctx, env);
            let effect = EffectOp::IndirectCall { effect: *effect };
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
            let condition_fact = resource_condition_fact(cond, ctx);
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
                condition_fact,
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
                AggregateKind::Struct {
                    name: name.clone(),
                    field_offsets: aggregate_construct_field_offsets(env.types, expr.ty),
                },
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
            push_construct(
                ops,
                AggregateKind::Tuple {
                    field_offsets: aggregate_construct_field_offsets(env.types, expr.ty),
                },
                inputs,
                expr,
                ctx,
            )
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
                lower_get_field_ref_intrinsic_source(name, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Borrow {
                    source,
                    output: output.clone(),
                    kind: BorrowKind::Shared,
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

fn try_lower_simple_direct_call_tree(
    expr: &HirExpr,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if !is_simple_direct_call_tree(expr) {
        return None;
    }

    enum Frame<'a> {
        Expr(&'a HirExpr),
        Call {
            expr: &'a HirExpr,
            callee: &'a FuncRef,
            args: &'a [HirExpr],
        },
    }

    let mut frames = alloc::vec![Frame::Expr(expr)];
    let mut values = Vec::new();
    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Expr(expr) => match &expr.kind {
                HirExprKind::LiteralI32(_)
                | HirExprKind::LiteralF32(_)
                | HirExprKind::LiteralBool(_)
                | HirExprKind::LiteralStr(_)
                | HirExprKind::Unit
                | HirExprKind::Var(_)
                | HirExprKind::FnValue(_) => {
                    values.push(lower_expr_skeleton(expr, ops, ctx, env));
                }
                HirExprKind::Call { callee, args } => {
                    frames.push(Frame::Call { expr, callee, args });
                    for arg in args.iter().rev() {
                        frames.push(Frame::Expr(arg));
                    }
                }
                _ => return None,
            },
            Frame::Call { expr, callee, args } => {
                let start = values.len().checked_sub(args.len())?;
                let arg_places = values.split_off(start);
                values.push(push_direct_call_skeleton(
                    expr, callee, args, arg_places, ops, ctx, env,
                ));
            }
        }
    }

    if values.len() == 1 {
        values.pop()
    } else {
        None
    }
}

fn is_simple_direct_call_tree(expr: &HirExpr) -> bool {
    let mut stack = alloc::vec![expr];
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Var(_)
            | HirExprKind::FnValue(_) => {}
            HirExprKind::Call { callee, args } => {
                if direct_call_needs_recursive_lowering(&callee) {
                    return false;
                }
                for arg in args {
                    stack.push(arg);
                }
            }
            _ => return false,
        }
    }
    true
}

fn direct_call_needs_recursive_lowering(callee: &FuncRef) -> bool {
    matches!(func_ref_base_name(callee), Some("get") | Some("get_field"))
}

fn push_direct_call_skeleton(
    expr: &HirExpr,
    callee: &FuncRef,
    args: &[HirExpr],
    arg_places: Vec<Place>,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Place {
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
    push_core_mem_wrapper_semantics(callee, args, &arg_places, &output, ops, env, expr.span);
    push_user_raw_address_return_semantics(callee, args, &arg_places, &output, ops, env, expr.span);
    if let Some(name) = func_ref_base_name(callee) {
        push_named_raw_address_semantics(name, args, &arg_places, &output, ops, env, expr.span);
    }
    if let Some(operation) = raw_memory_op_from_callee(callee)
        .filter(|operation| should_lower_raw_memory_call(operation, args, env))
    {
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

fn should_lower_raw_memory_call(
    operation: &RawMemoryOp,
    args: &[HirExpr],
    env: &LoweringEnvironment,
) -> bool {
    match operation {
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::Fill { .. }
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove => args
            .first()
            .map(|arg| !is_named_struct_type(env.types, arg.ty, "MemPtr"))
            .unwrap_or(true),
        RawMemoryOp::Alloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Other { .. } => true,
    }
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
            if let Some(effect) = raw_callee_internal_effect(name.as_str()) {
                resource_effect_from_internal(effect)
            } else {
                EffectOp::UserCall {
                    name: name.clone(),
                    effect: Effect::Pure,
                }
            }
        }
        FuncRef::User(name, _, _) => {
            if let Some(effect) = raw_callee_internal_effect(name.as_str()) {
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
    if let Some(effect) = raw_callee_internal_effect(name) {
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
    let selector = literal_field_selector(env, args.get(1)?)?;
    let projection =
        aggregate_field_projection_by_selector(env.types, owner.ty, selector, field_ty)?;
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
        if let Some(selector) = args.get(1).and_then(|arg| literal_field_selector(env, arg)) {
            aggregate_field_projection_by_selector(env.types, owner.ty, selector, field_ty)?
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

fn lower_get_field_ref_intrinsic_source(
    name: &str,
    args: &[HirExpr],
    ref_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if helper_base_name(name) != "get_field_ref" {
        return None;
    }
    let owner_ref = args.first()?;
    let selector = literal_field_selector(env, args.get(1)?)?;
    let owner_ty = reference_inner_type(env.types, owner_ref.ty)?;
    let field_ty = reference_inner_type(env.types, ref_ty)?;
    let projection =
        aggregate_field_projection_by_selector(env.types, owner_ty, selector, field_ty)?;
    let mut base = place_from_expr_skeleton(owner_ref, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner_ref, ops, ctx, env);
    }
    let deref_base = base.with_projection(super::model::PlaceProjection::Deref, owner_ty);
    Some(deref_base.with_projection(projection, field_ty))
}

pub(super) fn func_ref_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

fn reference_inner_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_id(ty);
    match types.get_ref(resolved) {
        TypeKind::Reference(inner, _) => Some(*inner),
        _ => None,
    }
}

fn aggregate_construct_field_offsets(types: &TypeCtx, ty: TypeId) -> Vec<usize> {
    aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .map(|field| field.offset)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldSelector<'a> {
    Name(&'a str),
    Index(usize),
}

fn literal_field_selector<'a>(
    env: &'a LoweringEnvironment,
    expr: &HirExpr,
) -> Option<FieldSelector<'a>> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => env
            .string_literals
            .get(*index as usize)
            .map(String::as_str)
            .map(FieldSelector::Name),
        HirExprKind::LiteralI32(index) if *index >= 0 => {
            Some(FieldSelector::Index(*index as usize))
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

fn aggregate_field_projection_by_selector(
    types: &TypeCtx,
    owner_ty: TypeId,
    selector: FieldSelector<'_>,
    field_ty: TypeId,
) -> Option<PlaceProjection> {
    let kind = aggregate_projection_kind(types, owner_ty)?;
    let fields = aggregate_fields_with_offsets(types, owner_ty);
    let index = match kind {
        AggregateProjectionKind::Struct => match selector {
            FieldSelector::Name(field_name) => aggregate_struct_field_names(types, owner_ty)?
                .iter()
                .position(|name| name == field_name)?,
            FieldSelector::Index(_) => return None,
        },
        AggregateProjectionKind::Tuple => match selector {
            FieldSelector::Name(field_name) => field_name.parse::<usize>().ok()?,
            FieldSelector::Index(index) => index,
        },
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

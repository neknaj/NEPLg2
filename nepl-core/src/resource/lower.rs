extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::effects::raw_callee_internal_effect;
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirMatchBindMode,
    HirMatchPattern, HirModule, HirParam,
};
use crate::layout::aggregate_fields_with_offsets;
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::address_projection::storage_offset_base_and_offset;
use super::lower_aggregate::{
    lower_compiler_field_load_source, lower_get_field_intrinsic_source,
    lower_get_field_ref_intrinsic_source, lower_reference_address_projection_source,
};
use super::lower_call::{
    call_effect_skeleton, func_ref_base_name, function_value_effect, intrinsic_effect_skeleton,
    lower_call_target, resource_effect_from_internal,
};
use super::lower_condition::resource_condition_fact;
use super::lower_layout_intrinsic::{
    layout_intrinsic_i32_value, layout_intrinsic_i32_value_from_callee,
};
use super::lower_match::{
    borrowed_match_payload_source, resource_match_scrutinee_place, type_is_reference_to_enum,
};
use super::lower_raw_address::{
    push_core_mem_owner_storage_origin, push_core_mem_wrapper_semantics,
    push_named_raw_address_semantics, push_transparent_raw_address_return_projection,
};
use super::lower_raw_memory::{
    raw_memory_call_uses_direct_raw_address, raw_memory_op_from_callee,
    raw_memory_op_from_intrinsic,
};
use super::lower_temporary_scope::push_line_copy_state_only_temporary_scope;
use super::model::{
    AggregateKind, BorrowKind, EffectOp, Place, RawBodyKind, ResourceBlock, ResourceBlockId,
    ResourceExprKind, ResourceFunction, ResourceId, ResourceLocal, ResourceMatchArm,
    ResourceMatchBindMode, ResourceMatchPattern, ResourceModule, ResourceOp, ResourceTerminator,
};

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

    pub(super) fn function_effect(&self, name: &str) -> Effect {
        self.function_effects
            .get(name)
            .copied()
            .unwrap_or(Effect::Pure)
    }

    pub(super) fn known_function_value_effect(&self, name: &str) -> Option<EffectOp> {
        if let Some(effect) = raw_callee_internal_effect(name) {
            Some(resource_effect_from_internal(effect))
        } else if self.function_effects.contains_key(name) {
            Some(EffectOp::UserCall {
                name: String::from(name),
                effect: self.function_effect(name),
            })
        } else {
            None
        }
    }

    fn known_function_value(&self, name: &str, ty: TypeId) -> Option<(String, EffectOp)> {
        if self.function_effects.contains_key(name) || raw_callee_internal_effect(name).is_some() {
            return Some((String::from(name), function_value_effect(name, self)));
        }
        let mut matches = self
            .functions
            .values()
            .filter(|function| function.origin_name == name && function.func_ty == ty);
        let function = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        let function_name = function.name.clone();
        let effect = function_value_effect(&function_name, self);
        Some((function_name, effect))
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
    next_local: usize,
    local_scopes: Vec<LocalScope>,
}

#[derive(Clone, Default)]
struct LocalScope {
    by_name: BTreeMap<String, Place>,
    declarations: Vec<Place>,
}

impl LocalScope {
    fn with_params(params: &[ResourceLocal]) -> Self {
        let mut scope = Self::default();
        for param in params {
            scope
                .by_name
                .insert(param.name.clone(), param.place.clone());
            scope.declarations.push(param.place.clone());
        }
        scope
    }

    fn declare(&mut self, name: String, place: Place) {
        self.by_name.insert(name, place.clone());
        self.declarations.push(place);
    }

    fn place_for_name(&self, name: &str) -> Option<Place> {
        self.by_name.get(name).cloned()
    }

    fn into_declarations(self) -> Vec<Place> {
        self.declarations
    }
}

impl LoweringContext {
    fn new(params: &[ResourceLocal]) -> Self {
        Self {
            next_resource: 0,
            next_local: 0,
            local_scopes: alloc::vec![LocalScope::with_params(params)],
        }
    }

    fn temporary(&mut self, ty: TypeId) -> Place {
        let id = ResourceId(self.next_resource);
        self.next_resource += 1;
        Place::temporary(id, ty)
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(LocalScope::default());
    }

    fn pop_scope(&mut self) -> Vec<Place> {
        self.local_scopes
            .pop()
            .unwrap_or_default()
            .into_declarations()
    }

    fn declare_local(&mut self, name: String, ty: TypeId) -> Place {
        let local_name = if self.local_place_for_name(&name).is_some() {
            let id = self.next_local;
            self.next_local += 1;
            alloc::format!("{name}#{id}")
        } else {
            name.clone()
        };
        let place = Place::local(local_name, ty);
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.declare(name, place.clone());
        }
        place
    }

    fn local_place(&self, name: &str, fallback_ty: TypeId) -> Place {
        self.local_place_for_name(name)
            .unwrap_or_else(|| Place::local(String::from(name), fallback_ty))
    }

    fn local_place_for_name(&self, name: &str) -> Option<Place> {
        self.local_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.place_for_name(name))
    }

    fn snapshot_locals(&self) -> Vec<LocalScope> {
        self.local_scopes.clone()
    }

    fn restore_locals(&mut self, local_scopes: Vec<LocalScope>) {
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
            let param_drop_locals = params
                .iter()
                .filter(|param| !env.types.is_copy(param.ty))
                .map(|param| param.place.clone())
                .collect::<Vec<_>>();
            if !param_drop_locals.is_empty() {
                ops.push(ResourceOp::EndScope {
                    locals: param_drop_locals,
                    result: Some(value.clone()),
                    span: block.span,
                });
            }
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
        origin_name: function.origin_name.clone(),
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
        let op_start = ops.len();
        let value = lower_expr_skeleton(&line.expr, ops, ctx, env);
        let result = (!line.drop_result).then(|| value.clone());
        push_line_copy_state_only_temporary_scope(env.types, ops, op_start, result, line.expr.span);
        if !line.drop_result {
            last = value;
        }
    }
    let locals = ctx.pop_scope();
    ops.push(ResourceOp::EndScope {
        locals,
        result: Some(last.clone()),
        span: block.span,
    });
    last
}

pub(super) fn lower_expr_skeleton(
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
            if ctx.local_place_for_name(name).is_none() {
                if let Some((function_name, effect)) = env.known_function_value(name, expr.ty) {
                    let output = ctx.temporary(expr.ty);
                    ops.push(ResourceOp::FunctionValue {
                        output: output.clone(),
                        name: function_name,
                        effect,
                        span: expr.span,
                    });
                    ops.push(ResourceOp::Expr {
                        kind: ResourceExprKind::FunctionValue,
                        output: output.clone(),
                        ty: expr.ty,
                        span: expr.span,
                    });
                    return output;
                }
            }
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
            let (function_name, effect) = env
                .known_function_value(name, expr.ty)
                .unwrap_or_else(|| (name.clone(), function_value_effect(name, env)));
            ops.push(ResourceOp::FunctionValue {
                output: output.clone(),
                name: function_name,
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
            let condition_fact = resource_condition_fact(cond, ctx);
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
                condition_fact,
                body_ops,
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Loop, expr, ctx)
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee = if type_is_reference_to_enum(env.types, scrutinee.ty) {
                let place = place_from_expr_skeleton(scrutinee, ctx);
                if matches!(&place.root, super::model::PlaceRoot::Unknown) {
                    lower_expr_skeleton(scrutinee, ops, ctx, env)
                } else {
                    place
                }
            } else {
                lower_expr_skeleton(scrutinee, ops, ctx, env)
            };
            let (scrutinee, scrutinee_is_borrow_target) =
                resource_match_scrutinee_place(env.types, &scrutinee);
            let mut resource_arms = Vec::new();
            let match_locals = ctx.snapshot_locals();
            for arm in arms {
                ctx.restore_locals(match_locals.clone());
                let mut arm_ops = Vec::new();
                let bind_source_name = arm.bind_local.clone();
                let bind_local = arm
                    .bind_local
                    .as_ref()
                    .zip(arm.bind_ty)
                    .map(|(name, ty)| ctx.declare_local(name.clone(), ty));
                let bind_mode = resource_match_bind_mode(arm.bind_mode);
                if matches!(bind_mode, Some(ResourceMatchBindMode::Borrowed { .. })) {
                    if let Some(bind_source) = borrowed_match_payload_source(
                        env.types,
                        &scrutinee,
                        arm,
                        bind_local.as_ref(),
                    ) {
                        let bind_local = bind_local.as_ref().expect("borrowed bind local exists");
                        arm_ops.push(ResourceOp::Borrow {
                            source: bind_source,
                            output: bind_local.clone(),
                            kind: borrow_kind_for_reference_type(env.types, bind_local.ty),
                            synthetic: true,
                            span: arm.body.span,
                        });
                    }
                }
                let value = lower_expr_skeleton(&arm.body, &mut arm_ops, ctx, env);
                if let Some(place) = &bind_local {
                    arm_ops.push(ResourceOp::EndScope {
                        locals: alloc::vec![place.clone()],
                        result: Some(value.clone()),
                        span: arm.body.span,
                    });
                }
                resource_arms.push(ResourceMatchArm {
                    pattern: lower_match_pattern(&arm.pattern),
                    bind_local,
                    bind_source_name,
                    bind_mode,
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
                scrutinee_is_borrow_target,
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
            let place = ctx.declare_local(name.clone(), value.ty);
            ops.push(ResourceOp::DeclareLocal {
                place: place.clone(),
                source_name: name.clone(),
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
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if let Some(source) =
                lower_reference_address_projection_source(name, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Borrow {
                    source,
                    output: output.clone(),
                    kind: borrow_kind_for_reference_type(env.types, expr.ty),
                    synthetic: false,
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Borrow,
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
                    kind: ResourceExprKind::Borrow,
                    output: output.clone(),
                    ty: expr.ty,
                    span: expr.span,
                });
                return output;
            }
            if let Some(source) =
                lower_get_field_ref_intrinsic_source(name, args, expr.ty, ops, ctx, env)
            {
                let output = ctx.temporary(expr.ty);
                ops.push(ResourceOp::Borrow {
                    source,
                    output: output.clone(),
                    kind: borrow_kind_for_reference_type(env.types, expr.ty),
                    synthetic: false,
                    span: expr.span,
                });
                ops.push(ResourceOp::Expr {
                    kind: ResourceExprKind::Borrow,
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
            if let Some(effect) = intrinsic_effect_skeleton(name) {
                ops.push(ResourceOp::CallEffect {
                    effect,
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
                let kind = layout_intrinsic_i32_value(name, type_args, env)
                    .map(ResourceExprKind::LiteralI32)
                    .unwrap_or(ResourceExprKind::Intrinsic);
                let output = push_expr(ops, kind, expr, ctx);
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
                kind: borrow_kind_for_reference_type(env.types, expr.ty),
                synthetic: false,
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
            HirExprKind::Call { args, .. } => {
                for arg in args {
                    stack.push(arg);
                }
            }
            _ => return false,
        }
    }
    true
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
    let lowered_core_mem_wrapper =
        push_core_mem_wrapper_semantics(callee, args, &arg_places, &output, ops, env, expr.span);
    if !lowered_core_mem_wrapper {
        if let Some(name) = func_ref_base_name(callee) {
            push_named_raw_address_semantics(name, args, &arg_places, &output, ops, env, expr.span);
        }
    }
    push_transparent_raw_address_return_projection(
        callee,
        args,
        &arg_places,
        &output,
        ops,
        env,
        expr.span,
    );
    push_core_mem_owner_storage_origin(callee, &output, ops, env, expr.span);
    if let Some(operation) = raw_memory_op_from_callee(callee)
        .filter(|operation| raw_memory_call_uses_direct_raw_address(operation, args, env.types))
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
    let kind = layout_intrinsic_i32_value_from_callee(callee, env)
        .map(ResourceExprKind::LiteralI32)
        .unwrap_or(ResourceExprKind::Call);
    ops.push(ResourceOp::Expr {
        kind,
        output: output.clone(),
        ty: expr.ty,
        span: expr.span,
    });
    output
}

fn lower_match_pattern(pattern: &HirMatchPattern) -> ResourceMatchPattern {
    match pattern {
        HirMatchPattern::Variant(name) => ResourceMatchPattern::Variant(name.clone()),
        HirMatchPattern::IntLiteral(value) => ResourceMatchPattern::IntLiteral(*value),
        HirMatchPattern::BoolLiteral(value) => ResourceMatchPattern::BoolLiteral(*value),
        HirMatchPattern::Wildcard => ResourceMatchPattern::Wildcard,
    }
}

fn resource_match_bind_mode(mode: Option<HirMatchBindMode>) -> Option<ResourceMatchBindMode> {
    match mode {
        Some(HirMatchBindMode::Owned) => Some(ResourceMatchBindMode::Owned),
        Some(HirMatchBindMode::Borrowed { is_mut }) => {
            Some(ResourceMatchBindMode::Borrowed { is_mut })
        }
        None => None,
    }
}

fn borrow_kind_for_reference_type(types: &TypeCtx, ty: TypeId) -> BorrowKind {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(_, true) => BorrowKind::Unique,
        TypeKind::Reference(_, false) => BorrowKind::Shared,
        _ => BorrowKind::Shared,
    }
}

fn aggregate_construct_field_offsets(types: &TypeCtx, ty: TypeId) -> Vec<usize> {
    aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .map(|field| field.offset)
        .collect()
}

pub(super) fn place_from_expr_skeleton(expr: &HirExpr, ctx: &LoweringContext) -> Place {
    if let Some((base, offset)) = storage_offset_base_and_offset(expr) {
        let source = place_from_expr_skeleton(base, ctx);
        if matches!(&source.root, super::model::PlaceRoot::Unknown) {
            return Place::unknown(expr.ty);
        }
        return source.with_projection(
            super::model::PlaceProjection::StorageOffset(offset),
            expr.ty,
        );
    }

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
        _ => Place::unknown(expr.ty),
    }
}

#[cfg(test)]
#[path = "lower_tests.rs"]
mod tests;

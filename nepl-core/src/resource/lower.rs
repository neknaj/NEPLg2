extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::effects::{intrinsic_is_raw_memory_effect, raw_callee_is_raw_memory_effect};
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirMatchPattern, HirModule,
    HirParam,
};
use crate::runtime_helpers::{
    helper_base_name, ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI,
};
use crate::types::TypeId;

use super::model::{
    AggregateKind, BorrowKind, EffectOp, Place, RawBodyKind, RawMemoryOp, ResourceBlock,
    ResourceBlockId, ResourceExprKind, ResourceFunction, ResourceId, ResourceLocal,
    ResourceMatchArm, ResourceMatchPattern, ResourceModule, ResourceOp, ResourceTerminator,
};

pub fn lower_hir_module_skeleton(module: &HirModule) -> ResourceModule {
    ResourceModule {
        functions: module
            .functions
            .iter()
            .map(lower_hir_function_skeleton)
            .collect(),
        entry: module.entry.clone(),
        string_literals: module.string_literals.clone(),
    }
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

fn lower_hir_function_skeleton(function: &HirFunction) -> ResourceFunction {
    let params = function
        .params
        .iter()
        .map(lower_param_skeleton)
        .collect::<Vec<_>>();
    let mut ctx = LoweringContext::new(&params);
    let mut ops = Vec::new();
    let terminator = match &function.body {
        HirBody::Block(block) => {
            let value = lower_block_skeleton(block, &mut ops, &mut ctx);
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
        let value = lower_expr_skeleton(&line.expr, ops, ctx);
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
        HirExprKind::FnValue(_) => push_expr(ops, ResourceExprKind::FunctionValue, expr, ctx),
        HirExprKind::Call { callee, args } => {
            let arg_places = lower_args_skeleton(args, ops, ctx);
            ops.push(ResourceOp::CallEffect {
                effect: call_effect_skeleton(callee),
                span: expr.span,
            });
            if let Some(operation) = raw_memory_op_from_callee(callee) {
                let output = ctx.temporary(expr.ty);
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
                output
            } else {
                push_expr(ops, ResourceExprKind::Call, expr, ctx)
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            lower_expr_skeleton(callee, ops, ctx);
            for arg in args {
                lower_expr_skeleton(arg, ops, ctx);
            }
            ops.push(ResourceOp::CallEffect {
                effect: EffectOp::Unknown {
                    reason: String::from("indirect call"),
                },
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::IndirectCall, expr, ctx)
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let condition = lower_expr_skeleton(cond, ops, ctx);
            let branch_locals = ctx.snapshot_locals();
            let mut then_ops = Vec::new();
            let then_value = lower_expr_skeleton(then_branch, &mut then_ops, ctx);
            ctx.restore_locals(branch_locals.clone());
            let mut else_ops = Vec::new();
            let else_value = lower_expr_skeleton(else_branch, &mut else_ops, ctx);
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
            let condition = lower_expr_skeleton(cond, &mut condition_ops, ctx);
            ctx.restore_locals(loop_locals.clone());
            let mut body_ops = Vec::new();
            lower_expr_skeleton(body, &mut body_ops, ctx);
            ctx.restore_locals(loop_locals);
            ops.push(ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span: expr.span,
            });
            Place::unknown(expr.ty)
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee = lower_expr_skeleton(scrutinee, ops, ctx);
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
                let value = lower_expr_skeleton(&arm.body, &mut arm_ops, ctx);
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
                inputs.push(lower_expr_skeleton(payload, ops, ctx));
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
                .map(|field| lower_expr_skeleton(field, ops, ctx))
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
                .map(|item| lower_expr_skeleton(item, ops, ctx))
                .collect();
            push_construct(ops, AggregateKind::Tuple, inputs, expr, ctx)
        }
        HirExprKind::Block(block) => lower_block_skeleton(block, ops, ctx),
        HirExprKind::Let {
            name,
            mutable,
            value,
        } => {
            let initializer = lower_expr_skeleton(value, ops, ctx);
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
            let value_place = lower_expr_skeleton(value, ops, ctx);
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
            let arg_places = lower_args_skeleton(args, ops, ctx);
            let raw_operation = raw_memory_op_from_intrinsic(name);
            if raw_operation.is_some() {
                ops.push(ResourceOp::CallEffect {
                    effect: EffectOp::UnsafeMemory {
                        operation: name.clone(),
                    },
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
                push_expr(ops, ResourceExprKind::Intrinsic, expr, ctx)
            }
        }
        HirExprKind::AddrOf(inner) => {
            let source = place_from_expr_skeleton(inner, ctx);
            if matches!(&source.root, super::model::PlaceRoot::Unknown) {
                lower_expr_skeleton(inner, ops, ctx);
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
            lower_expr_skeleton(inner, ops, ctx);
            push_expr(ops, ResourceExprKind::Deref, expr, ctx)
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
) -> Vec<Place> {
    args.iter()
        .map(|arg| lower_expr_skeleton(arg, ops, ctx))
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

fn call_effect_skeleton(callee: &FuncRef) -> EffectOp {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _)
            if raw_callee_is_raw_memory_effect(name.as_str()) =>
        {
            EffectOp::UnsafeMemory {
                operation: String::from(helper_base_name(name.as_str())),
            }
        }
        FuncRef::Builtin(name) => EffectOp::UserCall {
            name: name.clone(),
            effect: Effect::Pure,
        },
        FuncRef::User(name, _, _) => EffectOp::UserCall {
            name: name.clone(),
            effect: Effect::Pure,
        },
        FuncRef::Trait {
            trait_name, method, ..
        } => EffectOp::UserCall {
            name: alloc::format!("{}::{}", trait_name, method),
            effect: Effect::Pure,
        },
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

fn place_from_expr_skeleton(expr: &HirExpr, ctx: &LoweringContext) -> Place {
    match &expr.kind {
        HirExprKind::Var(name) => ctx.local_place(name, expr.ty),
        _ => Place::unknown(expr.ty),
    }
}

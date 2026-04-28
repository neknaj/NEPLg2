extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirModule, HirParam,
};

use super::model::{
    BorrowKind, EffectOp, Place, RawBodyKind, ResourceBlock, ResourceBlockId, ResourceExprKind,
    ResourceFunction, ResourceLocal, ResourceModule, ResourceOp, ResourceTerminator,
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

fn lower_hir_function_skeleton(function: &HirFunction) -> ResourceFunction {
    let params = function
        .params
        .iter()
        .map(lower_param_skeleton)
        .collect::<Vec<_>>();
    let mut ops = Vec::new();
    let terminator = match &function.body {
        HirBody::Block(block) => {
            lower_block_skeleton(block, &mut ops);
            ResourceTerminator::Return {
                value: None,
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

fn lower_block_skeleton(block: &HirBlock, ops: &mut Vec<ResourceOp>) {
    ops.push(ResourceOp::Expr {
        kind: ResourceExprKind::Block,
        ty: block.ty,
        span: block.span,
    });
    for line in &block.lines {
        lower_expr_skeleton(&line.expr, ops);
    }
}

fn lower_expr_skeleton(expr: &HirExpr, ops: &mut Vec<ResourceOp>) {
    match &expr.kind {
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => push_expr(ops, ResourceExprKind::Literal, expr),
        HirExprKind::Var(name) => {
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::LocalRead,
                ty: expr.ty,
                span: expr.span,
            });
            ops.push(ResourceOp::Read {
                source: Place::local(name.clone(), expr.ty),
                span: expr.span,
            });
        }
        HirExprKind::FnValue(_) => push_expr(ops, ResourceExprKind::FunctionValue, expr),
        HirExprKind::Call { callee, args } => {
            for arg in args {
                lower_expr_skeleton(arg, ops);
            }
            ops.push(ResourceOp::CallEffect {
                effect: call_effect_skeleton(callee),
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Call, expr);
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            lower_expr_skeleton(callee, ops);
            for arg in args {
                lower_expr_skeleton(arg, ops);
            }
            ops.push(ResourceOp::CallEffect {
                effect: EffectOp::Unknown {
                    reason: String::from("indirect call"),
                },
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::IndirectCall, expr);
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            lower_expr_skeleton(cond, ops);
            lower_expr_skeleton(then_branch, ops);
            lower_expr_skeleton(else_branch, ops);
            push_expr(ops, ResourceExprKind::Branch, expr);
        }
        HirExprKind::While { cond, body } => {
            lower_expr_skeleton(cond, ops);
            lower_expr_skeleton(body, ops);
            push_expr(ops, ResourceExprKind::Loop, expr);
        }
        HirExprKind::Match { scrutinee, arms } => {
            lower_expr_skeleton(scrutinee, ops);
            for arm in arms {
                lower_expr_skeleton(&arm.body, ops);
            }
            push_expr(ops, ResourceExprKind::Match, expr);
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                lower_expr_skeleton(payload, ops);
            }
            push_expr(ops, ResourceExprKind::Construct, expr);
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                lower_expr_skeleton(field, ops);
            }
            push_expr(ops, ResourceExprKind::Construct, expr);
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                lower_expr_skeleton(item, ops);
            }
            push_expr(ops, ResourceExprKind::Construct, expr);
        }
        HirExprKind::Block(block) => lower_block_skeleton(block, ops),
        HirExprKind::Let {
            name,
            mutable,
            value,
        } => {
            lower_expr_skeleton(value, ops);
            ops.push(ResourceOp::DeclareLocal {
                place: Place::local(name.clone(), value.ty),
                mutable: *mutable,
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Let, expr);
        }
        HirExprKind::Set { name, value } => {
            lower_expr_skeleton(value, ops);
            ops.push(ResourceOp::Assign {
                target: Place::local(name.clone(), value.ty),
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Set, expr);
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            for arg in args {
                lower_expr_skeleton(arg, ops);
            }
            if name == "load" || name == "store" {
                ops.push(ResourceOp::CallEffect {
                    effect: EffectOp::UnsafeMemory {
                        operation: name.clone(),
                    },
                    span: expr.span,
                });
            }
            push_expr(ops, ResourceExprKind::Intrinsic, expr);
        }
        HirExprKind::AddrOf(inner) => {
            lower_expr_skeleton(inner, ops);
            ops.push(ResourceOp::Borrow {
                source: place_from_expr_skeleton(inner),
                kind: BorrowKind::Shared,
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Borrow, expr);
        }
        HirExprKind::Deref(inner) => {
            lower_expr_skeleton(inner, ops);
            push_expr(ops, ResourceExprKind::Deref, expr);
        }
        HirExprKind::Drop { name } => {
            ops.push(ResourceOp::Drop {
                place: Place::local(name.clone(), expr.ty),
                span: expr.span,
            });
            push_expr(ops, ResourceExprKind::Drop, expr);
        }
    }
}

fn push_expr(ops: &mut Vec<ResourceOp>, kind: ResourceExprKind, expr: &HirExpr) {
    ops.push(ResourceOp::Expr {
        kind,
        ty: expr.ty,
        span: expr.span,
    });
}

fn call_effect_skeleton(callee: &FuncRef) -> EffectOp {
    match callee {
        FuncRef::Builtin(name) if raw_memory_name(name.as_str()) => EffectOp::UnsafeMemory {
            operation: name.clone(),
        },
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

fn raw_memory_name(name: &str) -> bool {
    matches!(
        name,
        "load" | "store" | "alloc_raw" | "dealloc_raw" | "realloc_raw" | "mem_copy" | "mem_move"
    )
}

fn place_from_expr_skeleton(expr: &HirExpr) -> Place {
    match &expr.kind {
        HirExprKind::Var(name) => Place::local(name.clone(), expr.ty),
        _ => Place::unknown(expr.ty),
    }
}

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::TraitCapability;
use crate::hir::{
    FuncRef, HirBlock, HirExpr, HirExprKind, HirFunction, HirLine, HirMatchArm, HirMatchBindMode,
    HirMatchPattern, HirModule, HirTraitApplication, HirTraitMethodId,
};
use crate::intrinsic_kinds::CoreIntrinsicKind;
use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::resource::{
    ResourceAutoDropKind, ResourceDropElaborationDrop, ResourceDropElaborationFunction,
    ResourceDropElaborationPlan, ResourceDropRequirement,
};
use crate::scalar_primitives::I32ArithmeticPrimitive;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDropInsertionError {
    MissingFunction {
        function: String,
    },
    UnconsumedDropPoint {
        function: String,
        span: Span,
        source_names: Vec<String>,
    },
}

#[derive(Debug, Clone)]
struct PendingDropPoint {
    span: Span,
    auto_drops: Vec<ResourceDropElaborationDrop>,
    consumed: bool,
}

#[derive(Debug, Clone)]
struct EnumDropVariant {
    name: String,
    payload: Option<TypeId>,
}

struct DropPlan {
    trait_application: HirTraitApplication,
    method_id: HirTraitMethodId,
    unit_ty: TypeId,
}

struct DropInsertionContext<'a> {
    types: &'a mut TypeCtx,
    plan: &'a DropPlan,
    pending_points: Vec<PendingDropPoint>,
    reserved_names: BTreeSet<String>,
    next_temp_id: usize,
}

impl<'a> DropInsertionContext<'a> {
    fn new(
        types: &'a mut TypeCtx,
        plan: &'a DropPlan,
        function_plan: &ResourceDropElaborationFunction,
        reserved_names: BTreeSet<String>,
    ) -> Self {
        Self {
            types,
            plan,
            pending_points: function_plan
                .drop_points
                .iter()
                .map(|point| PendingDropPoint {
                    span: point.span,
                    auto_drops: point.auto_drops.clone(),
                    consumed: false,
                })
                .collect(),
            reserved_names,
            next_temp_id: 0,
        }
    }

    fn take_scope_drops(
        &mut self,
        span: Span,
        source_names: &BTreeSet<String>,
    ) -> Vec<ResourceDropElaborationDrop> {
        self.take_matching_drops(span, ResourceAutoDropKind::ScopeLocal, |drop| {
            source_names.contains(&drop.source_name)
        })
    }

    fn take_assignment_drops(
        &mut self,
        span: Span,
        name: &str,
    ) -> Vec<ResourceDropElaborationDrop> {
        self.take_matching_drops(span, ResourceAutoDropKind::AssignmentOverwrite, |drop| {
            drop.source_name == name
        })
    }

    fn take_matching_drops<F>(
        &mut self,
        span: Span,
        kind: ResourceAutoDropKind,
        mut accepts: F,
    ) -> Vec<ResourceDropElaborationDrop>
    where
        F: FnMut(&ResourceDropElaborationDrop) -> bool,
    {
        let mut drops = Vec::new();
        for point in &mut self.pending_points {
            if point.consumed || point.span != span || point.auto_drops.is_empty() {
                continue;
            }
            if !point
                .auto_drops
                .iter()
                .all(|drop| drop.kind == kind && accepts(drop))
            {
                continue;
            }
            point.consumed = true;
            drops.extend(point.auto_drops.iter().cloned());
        }
        drops
    }

    fn unconsumed_points(&self, function: &str) -> Vec<ResourceDropInsertionError> {
        self.pending_points
            .iter()
            .filter(|point| !point.consumed && point.auto_drops.iter().any(drop_needs_code))
            .map(|point| ResourceDropInsertionError::UnconsumedDropPoint {
                function: function.to_string(),
                span: point.span,
                source_names: point_sources(point),
            })
            .collect()
    }

    fn drop_lines_for_drops(&mut self, drops: &[ResourceDropElaborationDrop]) -> Vec<HirLine> {
        let mut out = Vec::new();
        for drop in drops {
            out.extend(self.drop_lines_for_drop(drop));
        }
        out
    }

    fn drop_lines_for_drop(&mut self, drop: &ResourceDropElaborationDrop) -> Vec<HirLine> {
        self.drop_lines_for_requirement(
            drop.source_name.as_str(),
            drop.place.ty,
            drop.place.ty,
            0,
            &drop.requirement,
            drop.span,
            &mut BTreeSet::new(),
        )
    }

    fn drop_lines_for_requirement(
        &mut self,
        name: &str,
        owner_ty: TypeId,
        ty: TypeId,
        base_offset: usize,
        requirement: &ResourceDropRequirement,
        span: Span,
        visiting: &mut BTreeSet<TypeId>,
    ) -> Vec<HirLine> {
        match requirement {
            ResourceDropRequirement::StateOnly => Vec::new(),
            ResourceDropRequirement::WholeValue => {
                if base_offset == 0 && self.types.same_type(owner_ty, ty) {
                    vec![HirLine {
                        expr: drop_call_expr(self.types, self.plan, name.to_string(), ty, span),
                        drop_result: true,
                    }]
                } else {
                    vec![HirLine {
                        expr: drop_field_call_expr(
                            self.types,
                            self.plan,
                            name.to_string(),
                            owner_ty,
                            ty,
                            base_offset,
                            span,
                        ),
                        drop_result: true,
                    }]
                }
            }
            ResourceDropRequirement::DynamicEnumPayload => {
                if base_offset == 0 && self.types.same_type(owner_ty, ty) {
                    self.enum_payload_drop_lines(name, ty, span, visiting)
                } else {
                    self.load_place_and_drop_lines(name, owner_ty, ty, base_offset, span)
                }
            }
            ResourceDropRequirement::Structural {
                fields,
                dynamic_enum_fields,
            } => {
                let mut out = Vec::new();
                for field in fields {
                    out.push(HirLine {
                        expr: drop_field_call_expr(
                            self.types,
                            self.plan,
                            name.to_string(),
                            owner_ty,
                            field.ty,
                            base_offset + field.offset,
                            span,
                        ),
                        drop_result: true,
                    });
                }
                for field in dynamic_enum_fields {
                    out.extend(self.load_place_and_drop_lines(
                        name,
                        owner_ty,
                        field.ty,
                        base_offset + field.offset,
                        span,
                    ));
                }
                out
            }
        }
    }

    fn enum_payload_drop_lines(
        &mut self,
        name: &str,
        ty: TypeId,
        span: Span,
        visiting: &mut BTreeSet<TypeId>,
    ) -> Vec<HirLine> {
        let resolved = self.types.resolve_named_type_id(ty);
        if !visiting.insert(resolved) {
            return Vec::new();
        }

        let Some(variants) = self.enum_drop_variants(ty) else {
            visiting.remove(&resolved);
            return Vec::new();
        };

        let unit_ty = self.plan.unit_ty;
        let mut arms = Vec::with_capacity(variants.len());
        let mut has_payload_drop = false;

        for variant in variants {
            let mut bind_local = None;
            let mut bind_ty = None;
            let body = if let Some(payload_ty) = variant.payload {
                let payload_name = self.fresh_temp("__nepl_drop_enum_payload_");
                let payload_requirement =
                    crate::resource::resource_drop_requirement_for_type(self.types, payload_ty);
                let payload_drops = self.drop_lines_for_requirement(
                    payload_name.as_str(),
                    payload_ty,
                    payload_ty,
                    0,
                    &payload_requirement,
                    span,
                    visiting,
                );
                if payload_drops.is_empty() {
                    HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::Unit,
                        span,
                    }
                } else {
                    has_payload_drop = true;
                    bind_local = Some(payload_name);
                    bind_ty = Some(payload_ty);
                    HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::Block(HirBlock {
                            lines: payload_drops,
                            ty: unit_ty,
                            span,
                        }),
                        span,
                    }
                }
            } else {
                HirExpr {
                    ty: unit_ty,
                    kind: HirExprKind::Unit,
                    span,
                }
            };

            arms.push(HirMatchArm {
                pattern: HirMatchPattern::Variant(variant.name),
                bind_local,
                bind_ty,
                bind_mode: bind_ty.map(|_| HirMatchBindMode::Owned),
                body,
            });
        }

        visiting.remove(&resolved);
        if !has_payload_drop {
            return Vec::new();
        }

        vec![HirLine {
            expr: HirExpr {
                ty: unit_ty,
                kind: HirExprKind::Match {
                    scrutinee: Box::new(HirExpr {
                        ty,
                        kind: HirExprKind::Var(name.to_string()),
                        span,
                    }),
                    arms,
                },
                span,
            },
            drop_result: true,
        }]
    }

    fn enum_drop_variants(&self, ty: TypeId) -> Option<Vec<EnumDropVariant>> {
        let resolved = self.types.resolve_named_type_id(ty);
        match self.types.get_ref(resolved).clone() {
            TypeKind::Enum { variants, .. } => Some(
                variants
                    .into_iter()
                    .map(|variant| EnumDropVariant {
                        name: variant.name,
                        payload: variant.payload,
                    })
                    .collect(),
            ),
            TypeKind::Apply { base, args } => {
                let base = self.types.resolve_named_type_id(base);
                match self.types.get_ref(base).clone() {
                    TypeKind::Enum {
                        type_params,
                        variants,
                        ..
                    } => {
                        let mapping =
                            extend_type_mapping(self.types, &BTreeMap::new(), &type_params, &args);
                        Some(
                            variants
                                .into_iter()
                                .map(|variant| EnumDropVariant {
                                    name: variant.name,
                                    payload: variant.payload.map(|payload| {
                                        mapped_type_id(self.types, payload, &mapping)
                                    }),
                                })
                                .collect(),
                        )
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn load_place_and_drop_lines(
        &mut self,
        owner_name: &str,
        owner_ty: TypeId,
        ty: TypeId,
        offset: usize,
        span: Span,
    ) -> Vec<HirLine> {
        let temp_name = self.fresh_temp("__nepl_drop_enum_payload_");
        let requirement = crate::resource::resource_drop_requirement_for_type(self.types, ty);
        let mut lines = vec![HirLine {
            expr: HirExpr {
                ty: self.plan.unit_ty,
                kind: HirExprKind::Let {
                    name: temp_name.clone(),
                    mutable: false,
                    value: Box::new(HirExpr {
                        ty,
                        kind: HirExprKind::Intrinsic {
                            name: CoreIntrinsicKind::Load.intrinsic_name().to_string(),
                            type_args: vec![ty],
                            args: vec![self.place_addr_expr(
                                owner_name.to_string(),
                                owner_ty,
                                offset,
                                span,
                            )],
                        },
                        span,
                    }),
                },
                span,
            },
            drop_result: true,
        }];
        lines.extend(self.drop_lines_for_requirement(
            temp_name.as_str(),
            ty,
            ty,
            0,
            &requirement,
            span,
            &mut BTreeSet::new(),
        ));
        lines
    }

    fn place_addr_expr(
        &self,
        owner_name: String,
        owner_ty: TypeId,
        offset: usize,
        span: Span,
    ) -> HirExpr {
        if offset == 0 {
            return HirExpr {
                ty: owner_ty,
                kind: HirExprKind::Var(owner_name),
                span,
            };
        }

        HirExpr {
            ty: self.types.i32(),
            kind: HirExprKind::Intrinsic {
                name: I32ArithmeticPrimitive::Add.base_name().to_string(),
                type_args: vec![self.types.i32()],
                args: vec![
                    HirExpr {
                        ty: owner_ty,
                        kind: HirExprKind::Var(owner_name),
                        span,
                    },
                    HirExpr {
                        ty: self.types.i32(),
                        kind: HirExprKind::LiteralI32(offset as i32),
                        span,
                    },
                ],
            },
            span,
        }
    }

    fn fresh_temp(&mut self, prefix: &str) -> String {
        loop {
            let name = format!("{}{}", prefix, self.next_temp_id);
            self.next_temp_id += 1;
            if self.reserved_names.insert(name.clone()) {
                return name;
            }
        }
    }
}

pub fn insert_resource_drops(
    module: &mut HirModule,
    types: &mut TypeCtx,
    drop_plan: &ResourceDropElaborationPlan,
) -> Result<(), Vec<ResourceDropInsertionError>> {
    let Some(plan) = find_drop_plan(module, types.unit()) else {
        return Ok(());
    };
    let plans = drop_plan
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    for function_plan in drop_plan
        .functions
        .iter()
        .filter(|function| function.auto_drops.iter().any(drop_needs_code))
    {
        if !module
            .functions
            .iter()
            .any(|function| function.name == function_plan.name)
        {
            errors.push(ResourceDropInsertionError::MissingFunction {
                function: function_plan.name.clone(),
            });
        }
    }
    for function in &mut module.functions {
        let Some(function_plan) = plans.get(function.name.as_str()) else {
            continue;
        };
        let mut reserved_names = BTreeSet::new();
        collect_function_local_names(function, &mut reserved_names);
        let crate::hir::HirBody::Block(ref mut block) = function.body else {
            continue;
        };
        let mut ctx = DropInsertionContext::new(types, &plan, function_plan, reserved_names);
        let param_names = function
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        insert_drops_in_block(block, &param_names, &mut ctx);
        errors.extend(ctx.unconsumed_points(function.name.as_str()));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn find_drop_plan(module: &HirModule, unit_ty: TypeId) -> Option<DropPlan> {
    for tr in &module.traits {
        if !tr
            .capabilities
            .iter()
            .any(|cap| *cap == TraitCapability::Drop)
        {
            continue;
        }
        let method_name = if tr.methods.contains_key("drop") {
            String::from("drop")
        } else {
            tr.methods.keys().next().cloned()?
        };
        return Some(DropPlan {
            trait_application: HirTraitApplication::new(tr.name.clone(), Vec::new()),
            method_id: HirTraitMethodId::from_name(method_name),
            unit_ty,
        });
    }
    None
}

fn drop_needs_code(drop: &ResourceDropElaborationDrop) -> bool {
    !matches!(drop.requirement, ResourceDropRequirement::StateOnly)
}

fn point_sources(point: &PendingDropPoint) -> Vec<String> {
    point
        .auto_drops
        .iter()
        .map(|drop| drop.source_name.clone())
        .collect()
}

fn drop_call_expr(
    types: &mut TypeCtx,
    plan: &DropPlan,
    name: String,
    ty: TypeId,
    span: Span,
) -> HirExpr {
    HirExpr {
        ty: plan.unit_ty,
        kind: HirExprKind::Call {
            callee: FuncRef::Trait {
                application: plan.trait_application.clone(),
                method: plan.method_id.clone(),
                self_ty: ty,
            },
            args: vec![HirExpr {
                ty: types.reference(ty, false),
                kind: HirExprKind::AddrOf(Box::new(HirExpr {
                    ty,
                    kind: HirExprKind::Var(name),
                    span,
                })),
                span,
            }],
        },
        span,
    }
}

fn drop_field_call_expr(
    types: &mut TypeCtx,
    plan: &DropPlan,
    owner_name: String,
    owner_ty: TypeId,
    field_ty: TypeId,
    offset: usize,
    span: Span,
) -> HirExpr {
    let ref_ty = types.reference(field_ty, false);
    let arg = if offset == 0 {
        HirExpr {
            ty: ref_ty,
            kind: HirExprKind::Var(owner_name),
            span,
        }
    } else {
        HirExpr {
            ty: ref_ty,
            kind: HirExprKind::Intrinsic {
                name: I32ArithmeticPrimitive::Add.base_name().to_string(),
                type_args: vec![types.i32()],
                args: vec![
                    HirExpr {
                        ty: owner_ty,
                        kind: HirExprKind::Var(owner_name),
                        span,
                    },
                    HirExpr {
                        ty: types.i32(),
                        kind: HirExprKind::LiteralI32(offset as i32),
                        span,
                    },
                ],
            },
            span,
        }
    };
    HirExpr {
        ty: plan.unit_ty,
        kind: HirExprKind::Call {
            callee: FuncRef::Trait {
                application: plan.trait_application.clone(),
                method: plan.method_id.clone(),
                self_ty: field_ty,
            },
            args: vec![arg],
        },
        span,
    }
}

fn insert_drops_in_block(
    block: &mut HirBlock,
    outer_scope_names: &BTreeSet<String>,
    ctx: &mut DropInsertionContext<'_>,
) {
    for line in &mut block.lines {
        insert_drops_in_expr(&mut line.expr, ctx);
    }
    let mut source_names = outer_scope_names.clone();
    collect_direct_block_let_names(block, &mut source_names);
    let drops = ctx.take_scope_drops(block.span, &source_names);
    block.lines.extend(ctx.drop_lines_for_drops(&drops));
}

fn insert_drops_in_expr(expr: &mut HirExpr, ctx: &mut DropInsertionContext<'_>) {
    if can_skip_drop_plan_walk_iteratively(expr) {
        return;
    }

    match &mut expr.kind {
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            insert_drops_in_expr(cond, ctx);
            insert_drops_in_expr(then_branch, ctx);
            insert_drops_in_expr(else_branch, ctx);
        }
        HirExprKind::While { cond, body } => {
            insert_drops_in_expr(cond, ctx);
            insert_drops_in_expr(body, ctx);
        }
        HirExprKind::Match { scrutinee, arms } => {
            insert_drops_in_expr(scrutinee, ctx);
            for arm in arms {
                process_match_arm(arm, ctx);
            }
        }
        HirExprKind::Call { args, .. } | HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                insert_drops_in_expr(arg, ctx);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            insert_drops_in_expr(callee, ctx);
            for arg in args {
                insert_drops_in_expr(arg, ctx);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                insert_drops_in_expr(payload, ctx);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                insert_drops_in_expr(field, ctx);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                insert_drops_in_expr(item, ctx);
            }
        }
        HirExprKind::Block(block) => insert_drops_in_block(block, &BTreeSet::new(), ctx),
        HirExprKind::Let { value, .. } => {
            insert_drops_in_expr(value, ctx);
        }
        HirExprKind::Set { name, value } => {
            let target_name = name.clone();
            insert_drops_in_expr(value, ctx);
            let drops = ctx.take_assignment_drops(expr.span, target_name.as_str());
            let drop_lines = ctx.drop_lines_for_drops(&drops);
            if drop_lines.is_empty() {
                return;
            }
            let temp_name = ctx.fresh_temp("__nepl_drop_assign_tmp_");
            let temp_ty = value.ty;
            let temp_span = value.span;
            let unit_ty = ctx.types.unit();
            let original_value = core::mem::replace(
                value,
                Box::new(HirExpr {
                    ty: unit_ty,
                    kind: HirExprKind::Unit,
                    span: expr.span,
                }),
            );
            expr.kind = HirExprKind::Block(HirBlock {
                lines: {
                    let mut lines = vec![HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Let {
                                name: temp_name.clone(),
                                mutable: false,
                                value: original_value,
                            },
                            span: expr.span,
                        },
                        drop_result: false,
                    }];
                    lines.extend(drop_lines);
                    lines.push(HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Set {
                                name: target_name,
                                value: Box::new(HirExpr {
                                    ty: temp_ty,
                                    kind: HirExprKind::Var(temp_name),
                                    span: temp_span,
                                }),
                            },
                            span: expr.span,
                        },
                        drop_result: false,
                    });
                    lines
                },
                ty: unit_ty,
                span: expr.span,
            });
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            insert_drops_in_expr(inner, ctx);
        }
        HirExprKind::FnValue(_)
        | HirExprKind::Var(_)
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Drop { .. } => {}
    }
}

fn can_skip_drop_plan_walk_iteratively(expr: &HirExpr) -> bool {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::FnValue(_)
            | HirExprKind::Var(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Drop { .. } => {}
            HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::Intrinsic { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => return false,
        }
    }
    true
}

fn process_match_arm(arm: &mut HirMatchArm, ctx: &mut DropInsertionContext<'_>) {
    insert_drops_in_expr(&mut arm.body, ctx);
    let Some(bind) = &arm.bind_local else {
        return;
    };
    let source_names = [bind.clone()].into_iter().collect::<BTreeSet<_>>();
    let drops = ctx.take_scope_drops(arm.body.span, &source_names);
    let drop_lines = ctx.drop_lines_for_drops(&drops);
    append_drop_lines_to_expr(&mut arm.body, drop_lines);
}

fn append_drop_lines_to_expr(expr: &mut HirExpr, drops: Vec<HirLine>) {
    if drops.is_empty() {
        return;
    }
    match &mut expr.kind {
        HirExprKind::Block(block) => {
            block.lines.extend(drops);
        }
        _ => {
            let original = expr.clone();
            expr.kind = HirExprKind::Block(HirBlock {
                lines: {
                    let mut lines = Vec::new();
                    lines.push(HirLine {
                        expr: original,
                        drop_result: false,
                    });
                    lines.extend(drops);
                    lines
                },
                ty: expr.ty,
                span: expr.span,
            });
        }
    }
}

fn collect_function_local_names(function: &HirFunction, out: &mut BTreeSet<String>) {
    for param in &function.params {
        out.insert(param.name.clone());
    }
    if let crate::hir::HirBody::Block(block) = &function.body {
        collect_block_local_names(block, out);
    }
}

fn collect_block_local_names(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        collect_expr_local_names(&line.expr, out);
    }
}

fn collect_direct_block_let_names(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        if let HirExprKind::Let { name, .. } = &line.expr.kind {
            out.insert(name.clone());
        }
    }
}

fn collect_expr_local_names(expr: &HirExpr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_expr_local_names(cond, out);
            collect_expr_local_names(then_branch, out);
            collect_expr_local_names(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_expr_local_names(cond, out);
            collect_expr_local_names(body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_expr_local_names(scrutinee, out);
            for arm in arms {
                if let Some(name) = &arm.bind_local {
                    out.insert(name.clone());
                }
                collect_expr_local_names(&arm.body, out);
            }
        }
        HirExprKind::Call { args, .. } | HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_expr_local_names(arg, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_expr_local_names(callee, out);
            for arg in args {
                collect_expr_local_names(arg, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_expr_local_names(payload, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                collect_expr_local_names(field, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_expr_local_names(item, out);
            }
        }
        HirExprKind::Block(block) => collect_block_local_names(block, out),
        HirExprKind::Let { name, value, .. } => {
            out.insert(name.clone());
            collect_expr_local_names(value, out);
        }
        HirExprKind::Set { value, .. } => collect_expr_local_names(value, out),
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_expr_local_names(inner, out);
        }
        HirExprKind::FnValue(_)
        | HirExprKind::Var(_)
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Drop { .. } => {}
    }
}

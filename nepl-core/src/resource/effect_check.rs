extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::TypeCtx;

use super::effect_counts::ResourceEffectCounts;
use super::effect_diagnostic::{ResourceEffectBoundaryDiagnostic, ResourceEffectCallKind};
use super::effect_identity::{
    construct_pointer_alias_fields, construct_raw_identity_fields, copy_pointer_alias,
    raw_memory_op_produces_identity, RawIdentityTable, RawPointerAliasTable,
};
use super::effect_match::copy_match_payload_bind_identity;
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::effect_return_escape::raw_identity_return_projection_is_escape;
use super::effect_summary::{RawIdentityReturnSummaryIndex, RawPointerReturnSummaryIndex};
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::model::{
    EffectOp, Place, RawAddressViewKind, RawMemoryOp, ResourceBlock, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceOp, ResourceTerminator,
};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, raw_address_view_candidate_bases,
    reference_target_place,
};

pub(super) struct ResourceEffectBoundaryEngine<'a> {
    pub(super) function: &'a str,
    pub(super) effect: Effect,
    pub(super) summaries: &'a RawIdentityReturnSummaryIndex<'a>,
    pub(super) pointer_summaries: &'a RawPointerReturnSummaryIndex<'a>,
    pub(super) types: Option<&'a TypeCtx>,
    pub(super) track_alloc_identities: bool,
    pub(super) diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
    pub(super) counts: ResourceEffectCounts,
}

impl ResourceEffectBoundaryEngine<'_> {
    pub(super) fn check_function(&mut self, function: &ResourceFunction) {
        let mut identities = RawIdentityTable::default();
        let mut pointer_aliases = RawPointerAliasTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_memory_identities = RawMemoryIdentityTable::default();
        for block in &function.blocks {
            self.check_block(
                &mut identities,
                &mut pointer_aliases,
                &mut function_aliases,
                &mut raw_memory_identities,
                block,
            );
        }
    }

    fn check_block(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            identities,
            pointer_aliases,
            function_aliases,
            raw_memory_identities,
            &block.ops,
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if matches!(self.effect, Effect::Pure) {
                    if let Some(place) = value {
                        self.report_internal_alloc_identity_return(identities, place, *span);
                    }
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn report_internal_alloc_identity_return(
        &mut self,
        identities: &RawIdentityTable,
        place: &Place,
        span: Span,
    ) {
        let mut reported = Vec::new();
        for (suffix, ty, operations) in identities.projection_operations_under(place) {
            if !raw_identity_return_projection_is_escape(self.types, place, &suffix, ty) {
                continue;
            }
            let escaped_place = place_with_suffix(place, &suffix, ty);
            if reported
                .iter()
                .any(|prefix| place_suffix_after_prefix(&escaped_place, prefix).is_some())
            {
                continue;
            }
            for operation in operations {
                self.diagnostics.push(
                    ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
                        function: String::from(self.function),
                        operation,
                        place: escaped_place.clone(),
                        span,
                    },
                );
            }
            reported.push(escaped_place);
        }
    }

    pub(super) fn check_ops(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(
                identities,
                pointer_aliases,
                function_aliases,
                raw_memory_identities,
                op,
            );
        }
    }

    fn check_op(
        &mut self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &mut RawPointerAliasTable,
        function_aliases: &mut FunctionAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::CallEffect { effect, span } => self.check_effect(effect, *span),
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => {
                self.report_raw_memory_boundary_use(*operation, *span);
                if self.track_alloc_identities && raw_memory_op_produces_identity(operation) {
                    identities.mark(output, *operation);
                }
                if raw_memory_op_produces_identity(operation) {
                    pointer_aliases.mark(output);
                }
                self.apply_raw_memory_identity_effect(
                    identities,
                    pointer_aliases,
                    raw_memory_identities,
                    operation,
                    output,
                    args,
                );
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    identities.copy_identity(initializer, place);
                    copy_pointer_alias(pointer_aliases, raw_memory_identities, initializer, place);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                identities.copy_identity(source, output);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, source, output);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Borrow { source, output, .. } => {
                let target = reference_target_place(output, source.ty);
                identities.copy_identity(source, &target);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, source, &target);
                function_aliases.copy_alias(source, &target);
            }
            ResourceOp::RawAddressAlias { source, target, .. } => {
                identities.copy_identity(source, target);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, source, target);
            }
            ResourceOp::RawAddressView {
                source,
                target,
                kind,
                span,
            } => {
                self.report_raw_address_view_boundary_use(*kind, *span);
                identities.clear(target);
                for candidate in raw_address_view_candidate_bases(source) {
                    identities.merge_identity(&candidate, target);
                }
                copy_pointer_alias(pointer_aliases, raw_memory_identities, source, target);
            }
            ResourceOp::StorageOrigin { .. } => {}
            ResourceOp::Assign { target, value, .. } => {
                identities.copy_identity(value, target);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, value, target);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                identities.clear(output);
                construct_raw_identity_fields(identities, output, kind, inputs);
                construct_pointer_alias_fields(
                    pointer_aliases,
                    raw_memory_identities,
                    output,
                    kind,
                    inputs,
                );
                construct_function_alias_fields(function_aliases, output, kind, inputs);
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                self.report_unproven_checked_mem_ptr_access(
                    identities,
                    pointer_aliases,
                    effect,
                    args,
                    *span,
                );
                self.copy_call_return_identity(identities, output, target, args);
                self.copy_call_return_pointer_alias(
                    pointer_aliases,
                    raw_memory_identities,
                    output,
                    target,
                    args,
                );
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => {
                self.copy_indirect_call_return_identity(
                    identities,
                    function_aliases,
                    output,
                    callee,
                    args,
                );
                self.copy_indirect_call_return_pointer_alias(
                    pointer_aliases,
                    raw_memory_identities,
                    output,
                    callee,
                    args,
                    function_aliases,
                );
            }
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                let mut then_identities = identities.clone();
                let mut else_identities = identities.clone();
                let mut then_pointer_aliases = pointer_aliases.clone();
                let mut else_pointer_aliases = pointer_aliases.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                let mut then_raw_memory_identities = raw_memory_identities.clone();
                let mut else_raw_memory_identities = raw_memory_identities.clone();
                self.check_ops(
                    &mut then_identities,
                    &mut then_pointer_aliases,
                    &mut then_function_aliases,
                    &mut then_raw_memory_identities,
                    then_ops,
                );
                self.check_ops(
                    &mut else_identities,
                    &mut else_pointer_aliases,
                    &mut else_function_aliases,
                    &mut else_raw_memory_identities,
                    else_ops,
                );
                then_identities.copy_identity(then_value, output);
                else_identities.copy_identity(else_value, output);
                copy_pointer_alias(
                    &mut then_pointer_aliases,
                    &mut then_raw_memory_identities,
                    then_value,
                    output,
                );
                copy_pointer_alias(
                    &mut else_pointer_aliases,
                    &mut else_raw_memory_identities,
                    else_value,
                    output,
                );
                then_function_aliases.copy_alias(then_value, output);
                else_function_aliases.copy_alias(else_value, output);
                *identities = RawIdentityTable::merge_paths(&[then_identities, else_identities]);
                *pointer_aliases = RawPointerAliasTable::merge_paths(&[
                    then_pointer_aliases,
                    else_pointer_aliases,
                ]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                *raw_memory_identities = RawMemoryIdentityTable::merge_paths(&[
                    then_raw_memory_identities,
                    else_raw_memory_identities,
                ]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_identities = identities.clone();
                let mut condition_pointer_aliases = pointer_aliases.clone();
                let mut condition_function_aliases = function_aliases.clone();
                let mut condition_raw_memory_identities = raw_memory_identities.clone();
                self.check_ops(
                    &mut condition_identities,
                    &mut condition_pointer_aliases,
                    &mut condition_function_aliases,
                    &mut condition_raw_memory_identities,
                    condition_ops,
                );
                let mut body_identities = condition_identities.clone();
                let mut body_pointer_aliases = condition_pointer_aliases.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                let mut body_raw_memory_identities = condition_raw_memory_identities.clone();
                self.check_ops(
                    &mut body_identities,
                    &mut body_pointer_aliases,
                    &mut body_function_aliases,
                    &mut body_raw_memory_identities,
                    body_ops,
                );
                *identities =
                    RawIdentityTable::merge_paths(&[condition_identities, body_identities]);
                *pointer_aliases = RawPointerAliasTable::merge_paths(&[
                    condition_pointer_aliases,
                    body_pointer_aliases,
                ]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
                *raw_memory_identities = RawMemoryIdentityTable::merge_paths(&[
                    condition_raw_memory_identities,
                    body_raw_memory_identities,
                ]);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                let mut arm_paths = Vec::new();
                let mut pointer_alias_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                let mut raw_memory_identity_paths = Vec::new();
                for arm in arms {
                    let mut arm_identities = identities.clone();
                    let mut arm_pointer_aliases = pointer_aliases.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    let mut arm_raw_memory_identities = raw_memory_identities.clone();
                    copy_match_payload_bind_identity(
                        &mut arm_identities,
                        &mut arm_pointer_aliases,
                        &mut arm_function_aliases,
                        &mut arm_raw_memory_identities,
                        scrutinee,
                        arm,
                    );
                    self.check_ops(
                        &mut arm_identities,
                        &mut arm_pointer_aliases,
                        &mut arm_function_aliases,
                        &mut arm_raw_memory_identities,
                        &arm.ops,
                    );
                    arm_identities.copy_identity(&arm.value, output);
                    copy_pointer_alias(
                        &mut arm_pointer_aliases,
                        &mut arm_raw_memory_identities,
                        &arm.value,
                        output,
                    );
                    arm_function_aliases.copy_alias(&arm.value, output);
                    arm_paths.push(arm_identities);
                    pointer_alias_paths.push(arm_pointer_aliases);
                    function_alias_paths.push(arm_function_aliases);
                    raw_memory_identity_paths.push(arm_raw_memory_identities);
                }
                if !arm_paths.is_empty() {
                    *identities = RawIdentityTable::merge_paths(&arm_paths);
                    *pointer_aliases = RawPointerAliasTable::merge_paths(&pointer_alias_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                    *raw_memory_identities =
                        RawMemoryIdentityTable::merge_paths(&raw_memory_identity_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(value),
                output,
                ty,
                ..
            } => {
                let literal = Place::i32_constant(*value, *ty);
                copy_pointer_alias(pointer_aliases, raw_memory_identities, &literal, output);
            }
            ResourceOp::Expr { .. } | ResourceOp::Drop { .. } | ResourceOp::EndScope { .. } => {}
        }
    }

    fn copy_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self.pointer_summaries.get(name) else {
            return;
        };
        for parameter_return in &summary.parameter_returns {
            let Some(arg) = args.get(parameter_return.parameter_index) else {
                continue;
            };
            let source = place_with_suffix(
                arg,
                &parameter_return.source_projections,
                parameter_return.source_ty,
            );
            let target = place_with_suffix(
                output,
                &parameter_return.return_projections,
                parameter_return.return_ty,
            );
            copy_pointer_alias(pointer_aliases, raw_memory_identities, &source, &target);
        }
    }

    fn copy_indirect_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        function_aliases: &FunctionAliasTable,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            for arg in args {
                copy_pointer_alias(pointer_aliases, raw_memory_identities, arg, output);
            }
            return;
        }
        for function in functions {
            if let Some(summary) = self.pointer_summaries.get(function) {
                for parameter_return in &summary.parameter_returns {
                    let Some(arg) = args.get(parameter_return.parameter_index) else {
                        continue;
                    };
                    let source = place_with_suffix(
                        arg,
                        &parameter_return.source_projections,
                        parameter_return.source_ty,
                    );
                    let target = place_with_suffix(
                        output,
                        &parameter_return.return_projections,
                        parameter_return.return_ty,
                    );
                    copy_pointer_alias(pointer_aliases, raw_memory_identities, &source, &target);
                }
            }
        }
    }

    fn apply_raw_memory_identity_effect(
        &self,
        identities: &mut RawIdentityTable,
        pointer_aliases: &RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        operation: &RawMemoryOp,
        output: &Place,
        args: &[Place],
    ) {
        match operation {
            RawMemoryOp::Load => {
                if let Some(ptr) = args.first() {
                    let operations = raw_memory_identities.operations(pointer_aliases, ptr);
                    identities.mark_many(output, &operations);
                }
            }
            RawMemoryOp::Store => {
                if let Some(ptr) = args.first() {
                    let operations = args
                        .get(1)
                        .map(|value| identities.operations(value))
                        .unwrap_or_default();
                    if !operations.is_empty() {
                        raw_memory_identities.mark_many(pointer_aliases, ptr, &operations);
                    } else {
                        raw_memory_identities.clear(pointer_aliases, ptr);
                    }
                }
            }
            RawMemoryOp::Realloc => {
                let carried_operations = args
                    .first()
                    .map(|ptr| raw_memory_identities.operations(pointer_aliases, ptr))
                    .unwrap_or_default();
                if let Some(ptr) = args.first() {
                    raw_memory_identities.clear(pointer_aliases, ptr);
                }
                if !carried_operations.is_empty() {
                    raw_memory_identities.mark_many(pointer_aliases, output, &carried_operations);
                }
            }
            RawMemoryOp::Dealloc => {
                if let Some(ptr) = args.first() {
                    raw_memory_identities.clear(pointer_aliases, ptr);
                }
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                if let (Some(dst), Some(src)) = (args.first(), args.get(1)) {
                    let operations = raw_memory_identities.operations(pointer_aliases, src);
                    if !operations.is_empty() {
                        raw_memory_identities.mark_many(pointer_aliases, dst, &operations);
                    } else {
                        raw_memory_identities.clear(pointer_aliases, dst);
                    }
                }
            }
            RawMemoryOp::FillBytes | RawMemoryOp::Fill => {
                if let Some(ptr) = args.first() {
                    raw_memory_identities.clear(pointer_aliases, ptr);
                }
            }
            RawMemoryOp::Alloc | RawMemoryOp::MemorySize | RawMemoryOp::MemoryGrow => {}
        }
    }

    fn check_effect(&mut self, effect: &EffectOp, span: Span) {
        match effect {
            EffectOp::InternalAlloc { operation } => {
                self.counts.internal_memory_ops.record(*operation);
                if matches!(self.effect, Effect::Pure)
                    && internal_alloc_operation_requires_pure_boundary_diagnostic(*operation)
                {
                    self.diagnostics.push(
                        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
                            function: String::from(self.function),
                            operation: *operation,
                            span,
                        },
                    );
                }
            }
            EffectOp::UnsafeMemory { operation } => {
                self.counts.unsafe_memory_ops.record(*operation);
                if matches!(self.effect, Effect::Pure) {
                    self.diagnostics.push(
                        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
                            function: String::from(self.function),
                            operation: *operation,
                            span,
                        },
                    );
                }
            }
            EffectOp::ExternalIo { operation } => {
                self.counts.external_io_ops.record(*operation);
                self.check_call_effect(
                    Effect::Impure,
                    ResourceEffectCallKind::ExternalIo {
                        operation: *operation,
                    },
                    span,
                );
            }
            EffectOp::Nondet { operation } => {
                self.counts.nondet_ops.record(*operation);
                self.check_call_effect(
                    Effect::Impure,
                    ResourceEffectCallKind::Nondet {
                        operation: *operation,
                    },
                    span,
                );
            }
            EffectOp::UserCall { name, effect } => {
                self.check_call_effect(
                    *effect,
                    ResourceEffectCallKind::Direct { name: name.clone() },
                    span,
                );
            }
            EffectOp::IndirectCall { effect } => {
                self.check_call_effect(*effect, ResourceEffectCallKind::Indirect, span);
            }
            EffectOp::Unknown { reason } => {
                self.counts.unknown_ops += 1;
                self.diagnostics
                    .push(ResourceEffectBoundaryDiagnostic::UnknownEffect {
                        function: String::from(self.function),
                        reason: *reason,
                        span,
                    });
            }
            EffectOp::Pure => {}
        }
    }

    fn report_raw_memory_boundary_use(&mut self, operation: RawMemoryOp, span: Span) {
        self.diagnostics
            .push(ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
                function: String::from(self.function),
                operation,
                span,
            });
    }

    fn report_raw_address_view_boundary_use(&mut self, kind: RawAddressViewKind, span: Span) {
        match kind {
            RawAddressViewKind::MemPtrOffset => {
                self.diagnostics.push(
                    ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
                        function: String::from(self.function),
                        kind,
                        span,
                    },
                );
            }
            RawAddressViewKind::Offset | RawAddressViewKind::NonOwningProjection => {}
        }
    }

    fn check_call_effect(&mut self, effect: Effect, call: ResourceEffectCallKind, span: Span) {
        if matches!(self.effect, Effect::Pure) && matches!(effect, Effect::Impure) {
            self.diagnostics
                .push(ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
                    function: String::from(self.function),
                    call,
                    span,
                });
        }
    }
}

fn internal_alloc_operation_requires_pure_boundary_diagnostic(operation: RawMemoryOp) -> bool {
    match operation {
        RawMemoryOp::Alloc => false,
        RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => true,
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill => false,
    }
}

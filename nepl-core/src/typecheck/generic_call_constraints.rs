use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::traits::infer_type_param_from_instantiated_pair;
use super::type_expectation::TypeExpectation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GenericCallConstraintSource {
    Argument { index: usize },
    ExpectedResult { expectation: TypeExpectation },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GenericCallConstraint {
    source: GenericCallConstraintSource,
    declared: TypeId,
    instantiated: TypeId,
    actual: TypeId,
    span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GenericCallConstraintViolation {
    Argument { index: usize, span: Span },
    ExpectedResult { span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GenericTypeArgumentConflict {
    type_param: TypeId,
    first: TypeId,
    second: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GenericTypeArgumentInference {
    NoEvidence,
    Unique(TypeId),
    Conflict(GenericTypeArgumentConflict),
}

pub(super) struct GenericTypeArgumentResolution {
    pub(super) resolved_args: Vec<TypeId>,
    pub(super) conflicts: Vec<GenericTypeArgumentConflict>,
}

impl GenericCallConstraintViolation {
    pub(super) fn diagnostic_code(self) -> TypeDiagnosticCode {
        match self {
            GenericCallConstraintViolation::Argument { .. } => TypeDiagnosticCode::ArgumentMismatch,
            GenericCallConstraintViolation::ExpectedResult { .. } => {
                TypeDiagnosticCode::AnnotationMismatch
            }
        }
    }

    pub(super) fn message(self) -> &'static str {
        match self {
            GenericCallConstraintViolation::Argument { .. } => "argument type mismatch",
            GenericCallConstraintViolation::ExpectedResult { .. } => {
                "call result does not match expected type"
            }
        }
    }

    pub(super) fn span(self) -> Span {
        match self {
            GenericCallConstraintViolation::Argument { span, .. }
            | GenericCallConstraintViolation::ExpectedResult { span } => span,
        }
    }
}

impl GenericTypeArgumentConflict {
    fn new(type_param: TypeId, first: TypeId, second: TypeId) -> Self {
        Self {
            type_param,
            first,
            second,
        }
    }

    pub(super) fn diagnostic_message(self, ctx: &TypeCtx) -> String {
        format!(
            "generic type parameter {} has inconsistent constraints ({} vs {})",
            ctx.type_to_string(self.type_param),
            ctx.type_to_string(self.first),
            ctx.type_to_string(self.second)
        )
    }
}

impl GenericTypeArgumentInference {
    fn merge(self, ctx: &TypeCtx, candidate: Option<TypeId>, type_param: TypeId) -> Self {
        match (self, candidate) {
            (GenericTypeArgumentInference::Conflict(conflict), _) => {
                GenericTypeArgumentInference::Conflict(conflict)
            }
            (current, None) => current,
            (GenericTypeArgumentInference::NoEvidence, Some(candidate)) => {
                GenericTypeArgumentInference::Unique(ctx.resolve_id(candidate))
            }
            (GenericTypeArgumentInference::Unique(current), Some(candidate)) => {
                let candidate = ctx.resolve_id(candidate);
                if ctx.same_type(current, candidate) {
                    GenericTypeArgumentInference::Unique(ctx.resolve_id(current))
                } else {
                    GenericTypeArgumentInference::Conflict(GenericTypeArgumentConflict::new(
                        type_param, current, candidate,
                    ))
                }
            }
        }
    }
}

impl GenericCallConstraint {
    pub(super) fn argument(
        index: usize,
        declared: TypeId,
        instantiated: TypeId,
        actual: TypeId,
        span: Span,
    ) -> Self {
        Self {
            source: GenericCallConstraintSource::Argument { index },
            declared,
            instantiated,
            actual,
            span,
        }
    }

    pub(super) fn expected_result(
        declared: TypeId,
        instantiated: TypeId,
        expectation: TypeExpectation,
        call_span: Span,
    ) -> Self {
        Self {
            source: GenericCallConstraintSource::ExpectedResult { expectation },
            declared,
            instantiated,
            actual: expectation.target(),
            span: expectation.diagnostic_span(call_span),
        }
    }

    pub(super) fn check(self, ctx: &mut TypeCtx) -> Result<(), GenericCallConstraintViolation> {
        ctx.unify(self.instantiated, self.actual)
            .map(|_| ())
            .map_err(|_| self.violation())
    }

    pub(super) fn infer_for_type_param(
        self,
        ctx: &TypeCtx,
        target_tp: TypeId,
        target_label: Option<&str>,
    ) -> Option<TypeId> {
        match self.source {
            GenericCallConstraintSource::Argument { .. }
            | GenericCallConstraintSource::ExpectedResult { .. } => {
                infer_type_param_from_instantiated_pair(
                    ctx,
                    self.declared,
                    self.actual,
                    target_tp,
                    target_label,
                )
            }
        }
    }

    fn violation(self) -> GenericCallConstraintViolation {
        match self.source {
            GenericCallConstraintSource::Argument { index } => {
                GenericCallConstraintViolation::Argument {
                    index,
                    span: self.span,
                }
            }
            GenericCallConstraintSource::ExpectedResult { .. } => {
                GenericCallConstraintViolation::ExpectedResult { span: self.span }
            }
        }
    }
}

pub(super) fn resolve_generic_type_args_from_constraints(
    ctx: &TypeCtx,
    type_params: &[TypeId],
    fallback_args: Vec<TypeId>,
    constraints: &[GenericCallConstraint],
) -> GenericTypeArgumentResolution {
    let mut resolved_args = fallback_args
        .into_iter()
        .map(|ty| ctx.resolve_id(ty))
        .collect::<Vec<_>>();
    let mut conflicts = Vec::new();
    if type_params.len() != resolved_args.len() {
        return GenericTypeArgumentResolution {
            resolved_args,
            conflicts,
        };
    }

    for (idx, tp) in type_params.iter().enumerate() {
        let label = match ctx.get(ctx.resolve_id(*tp)) {
            TypeKind::Var(v) => v.label.clone(),
            _ => None,
        };
        let mut inference = GenericTypeArgumentInference::NoEvidence;
        for constraint in constraints.iter().copied() {
            inference = inference.merge(
                ctx,
                constraint.infer_for_type_param(ctx, *tp, label.as_deref()),
                *tp,
            );
        }
        match inference {
            GenericTypeArgumentInference::NoEvidence => {}
            GenericTypeArgumentInference::Unique(inferred) => {
                resolved_args[idx] = ctx.resolve_id(inferred);
            }
            GenericTypeArgumentInference::Conflict(conflict) => conflicts.push(conflict),
        }
    }

    GenericTypeArgumentResolution {
        resolved_args,
        conflicts,
    }
}

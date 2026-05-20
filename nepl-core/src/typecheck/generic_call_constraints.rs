use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::type_argument_inference::{
    resolve_type_arguments_from_constraints, TypeArgumentConstraint, TypeArgumentResolution,
};
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

    fn type_argument_constraint(self) -> TypeArgumentConstraint {
        match self.source {
            GenericCallConstraintSource::Argument { .. }
            | GenericCallConstraintSource::ExpectedResult { .. } => {
                TypeArgumentConstraint::new(self.declared, self.actual)
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
) -> TypeArgumentResolution {
    let type_argument_constraints = constraints
        .iter()
        .copied()
        .map(GenericCallConstraint::type_argument_constraint)
        .collect::<Vec<_>>();
    resolve_type_arguments_from_constraints(
        ctx,
        type_params,
        fallback_args,
        &type_argument_constraints,
    )
}

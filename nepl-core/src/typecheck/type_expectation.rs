use crate::span::Span;
use crate::types::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeExpectationSource {
    ExplicitAscription,
    BlockResult,
    OuterConsumerArgument,
}

/// Tracks where an expected type came from and the stack depth where it becomes
/// applicable. Keeping this as a typed value prevents each reduction path from
/// reinterpreting an unstructured `(TypeId, usize)` pair differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TypeExpectation {
    target: TypeId,
    base_depth: usize,
    span: Span,
    source: TypeExpectationSource,
}

impl TypeExpectation {
    pub(super) fn explicit_ascription(target: TypeId, base_depth: usize, span: Span) -> Self {
        Self {
            target,
            base_depth,
            span,
            source: TypeExpectationSource::ExplicitAscription,
        }
    }

    pub(super) fn block_result(target: TypeId, base_depth: usize) -> Self {
        Self {
            target,
            base_depth,
            span: Span::dummy(),
            source: TypeExpectationSource::BlockResult,
        }
    }

    pub(super) fn outer_consumer_argument(target: TypeId, base_depth: usize) -> Self {
        Self {
            target,
            base_depth,
            span: Span::dummy(),
            source: TypeExpectationSource::OuterConsumerArgument,
        }
    }

    pub(super) fn target(self) -> TypeId {
        self.target
    }

    pub(super) fn base_depth(self) -> usize {
        self.base_depth
    }

    pub(super) fn applies_at_stack_len(self, stack_len: usize) -> bool {
        stack_len == self.base_depth + 1
    }

    pub(super) fn call_result_expectation_after_args(
        self,
        stack_len_before_call: usize,
        args_to_take: usize,
    ) -> Option<Self> {
        let new_len = stack_len_before_call.saturating_sub(args_to_take);
        if self.applies_at_stack_len(new_len) {
            Some(self)
        } else {
            None
        }
    }

    pub(super) fn diagnostic_span(self, expression_span: Span) -> Span {
        match self.source {
            TypeExpectationSource::ExplicitAscription => self.span,
            TypeExpectationSource::BlockResult | TypeExpectationSource::OuterConsumerArgument => {
                expression_span
            }
        }
    }
}

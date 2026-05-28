use alloc::format;
use alloc::string::String;

use super::env::Binding;
use super::FieldAccessorKind;

#[derive(Clone, Copy)]
pub(super) struct OverloadCandidate<'b> {
    pub(super) binding: &'b Binding,
    pub(super) type_param_count: usize,
    pub(super) instantiated_specificity: usize,
    pub(super) declared_specificity: usize,
    pub(super) field_accessor: Option<FieldAccessorKind>,
}

#[derive(Clone, Copy)]
pub(super) enum OverloadCandidateRejection {
    NotFunction,
    TypeArgumentCount,
    CaptureArity,
    UserArity,
    DeclaredExpectedResult,
    InstantiatedNotFunction,
    ArgumentType,
    ExpectedResult,
    GenericConstraintConflict,
    TraitBoundUnsatisfied,
}

#[derive(Clone, Copy)]
enum OverloadCandidateMaterializationPhase {
    BeforeInstantiation,
    AfterInstantiation,
}

impl OverloadCandidateRejection {
    fn materialization_phase(self) -> OverloadCandidateMaterializationPhase {
        match self {
            OverloadCandidateRejection::NotFunction
            | OverloadCandidateRejection::TypeArgumentCount
            | OverloadCandidateRejection::CaptureArity
            | OverloadCandidateRejection::UserArity
            | OverloadCandidateRejection::DeclaredExpectedResult => {
                OverloadCandidateMaterializationPhase::BeforeInstantiation
            }
            OverloadCandidateRejection::InstantiatedNotFunction
            | OverloadCandidateRejection::ArgumentType
            | OverloadCandidateRejection::ExpectedResult
            | OverloadCandidateRejection::GenericConstraintConflict
            | OverloadCandidateRejection::TraitBoundUnsatisfied => {
                OverloadCandidateMaterializationPhase::AfterInstantiation
            }
        }
    }
}

#[derive(Default)]
pub(super) struct OverloadCandidateStats {
    pub(super) considered: usize,
    pub(super) materialized: usize,
    pub(super) accepted: usize,
    rejected_before_materialization: usize,
    rejected_after_materialization: usize,
    not_function: usize,
    type_argument_count: usize,
    capture_arity: usize,
    user_arity: usize,
    declared_expected_result: usize,
    instantiated_not_function: usize,
    argument_type: usize,
    expected_result: usize,
    generic_constraint_conflict: usize,
    trait_bound_unsatisfied: usize,
}

impl OverloadCandidateStats {
    pub(super) fn record_considered(&mut self) {
        self.considered += 1;
    }

    pub(super) fn record_materialized(&mut self) {
        self.materialized += 1;
    }

    pub(super) fn record_accepted(&mut self) {
        self.accepted += 1;
    }

    pub(super) fn record_rejection(&mut self, reason: OverloadCandidateRejection) {
        match reason.materialization_phase() {
            OverloadCandidateMaterializationPhase::BeforeInstantiation => {
                self.rejected_before_materialization += 1
            }
            OverloadCandidateMaterializationPhase::AfterInstantiation => {
                self.rejected_after_materialization += 1
            }
        }
        match reason {
            OverloadCandidateRejection::NotFunction => self.not_function += 1,
            OverloadCandidateRejection::TypeArgumentCount => self.type_argument_count += 1,
            OverloadCandidateRejection::CaptureArity => self.capture_arity += 1,
            OverloadCandidateRejection::UserArity => self.user_arity += 1,
            OverloadCandidateRejection::DeclaredExpectedResult => {
                self.declared_expected_result += 1
            }
            OverloadCandidateRejection::InstantiatedNotFunction => {
                self.instantiated_not_function += 1
            }
            OverloadCandidateRejection::ArgumentType => self.argument_type += 1,
            OverloadCandidateRejection::ExpectedResult => self.expected_result += 1,
            OverloadCandidateRejection::GenericConstraintConflict => {
                self.generic_constraint_conflict += 1
            }
            OverloadCandidateRejection::TraitBoundUnsatisfied => self.trait_bound_unsatisfied += 1,
        }
    }

    pub(super) fn pre_materialized_rejections(&self) -> usize {
        self.rejected_before_materialization
    }

    pub(super) fn rejected_only_by_trait_bounds_after_materialization(&self) -> bool {
        self.accepted == 0
            && self.materialized > 0
            && self.trait_bound_unsatisfied > 0
            && self.rejected_before_materialization == 0
            && self.rejected_after_materialization == self.trait_bound_unsatisfied
            && self.materialized == self.trait_bound_unsatisfied
    }

    pub(super) fn assert_materialization_guard(&self) {
        debug_assert!(self.materialized + self.pre_materialized_rejections() <= self.considered);
    }
}

#[derive(Clone, Copy)]
pub(super) enum OverloadCandidateNarrowingStage {
    InitialCandidates,
    PreferPureFunction,
    SignatureDedup,
    PreferOrdinaryFunction,
    PreferConcreteSignature,
    PreferFewerTypeParameters,
    PreferInstantiatedSpecificity,
    PreferDeclaredSpecificity,
}

impl OverloadCandidateNarrowingStage {
    fn diagnostic_label(self) -> &'static str {
        match self {
            OverloadCandidateNarrowingStage::InitialCandidates => "initial candidate filtering",
            OverloadCandidateNarrowingStage::PreferPureFunction => "pure function preference",
            OverloadCandidateNarrowingStage::SignatureDedup => "signature deduplication",
            OverloadCandidateNarrowingStage::PreferOrdinaryFunction => {
                "ordinary function preference"
            }
            OverloadCandidateNarrowingStage::PreferConcreteSignature => {
                "concrete signature preference"
            }
            OverloadCandidateNarrowingStage::PreferFewerTypeParameters => {
                "type parameter count preference"
            }
            OverloadCandidateNarrowingStage::PreferInstantiatedSpecificity => {
                "instantiated specificity preference"
            }
            OverloadCandidateNarrowingStage::PreferDeclaredSpecificity => {
                "declared specificity preference"
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct OverloadAmbiguityReason {
    after_stage: OverloadCandidateNarrowingStage,
    remaining_candidates: usize,
}

impl OverloadAmbiguityReason {
    pub(super) fn after_stage(
        after_stage: OverloadCandidateNarrowingStage,
        remaining_candidates: usize,
    ) -> Self {
        Self {
            after_stage,
            remaining_candidates,
        }
    }

    pub(super) fn diagnostic_message(self) -> String {
        format!(
            "ambiguous overload after {} ({} candidates remain)",
            self.after_stage.diagnostic_label(),
            self.remaining_candidates
        )
    }
}

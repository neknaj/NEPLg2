#!/usr/bin/env node
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const typeExpectationPath = path.join(repoRoot, 'nepl-core/src/typecheck/type_expectation.rs');
const prefixCheckPath = path.join(repoRoot, 'nepl-core/src/typecheck/prefix_check.rs');
const callReductionPath = path.join(repoRoot, 'nepl-core/src/typecheck/call_reduction.rs');
const callResolutionPath = path.join(repoRoot, 'nepl-core/src/typecheck/call_resolution.rs');
const functionApplyPath = path.join(repoRoot, 'nepl-core/src/typecheck/function_apply.rs');
const genericCallConstraintsPath = path.join(repoRoot, 'nepl-core/src/typecheck/generic_call_constraints.rs');
const overloadSelectionPath = path.join(repoRoot, 'nepl-core/src/typecheck/overload_selection.rs');
const selectedCallApplyPath = path.join(repoRoot, 'nepl-core/src/typecheck/selected_call_apply.rs');
const indirectApplyPath = path.join(repoRoot, 'nepl-core/src/typecheck/indirect_apply.rs');
const traitCallApplyPath = path.join(repoRoot, 'nepl-core/src/typecheck/trait_call_apply.rs');
const traitCheckPath = path.join(repoRoot, 'nepl-core/src/typecheck/trait_check.rs');
const typeArgumentInferencePath = path.join(repoRoot, 'nepl-core/src/typecheck/type_argument_inference.rs');

const typeExpectation = fs.readFileSync(typeExpectationPath, 'utf8');
const prefixCheck = fs.readFileSync(prefixCheckPath, 'utf8');
const callReduction = fs.readFileSync(callReductionPath, 'utf8');
const callResolution = fs.readFileSync(callResolutionPath, 'utf8');
const functionApply = fs.readFileSync(functionApplyPath, 'utf8');
const genericCallConstraints = fs.readFileSync(genericCallConstraintsPath, 'utf8');
const overloadSelection = fs.readFileSync(overloadSelectionPath, 'utf8');
const selectedCallApply = fs.readFileSync(selectedCallApplyPath, 'utf8');
const indirectApply = fs.readFileSync(indirectApplyPath, 'utf8');
const traitCallApply = fs.readFileSync(traitCallApplyPath, 'utf8');
const traitCheck = fs.readFileSync(traitCheckPath, 'utf8');
const typeArgumentInference = fs.readFileSync(typeArgumentInferencePath, 'utf8');

assert.match(typeExpectation, /enum\s+TypeExpectationSource\s*{[\s\S]*ExplicitAscription[\s\S]*BlockResult[\s\S]*OuterConsumerArgument[\s\S]*}/);
assert.match(typeExpectation, /struct\s+TypeExpectation\s*{[\s\S]*target:\s*TypeId[\s\S]*base_depth:\s*usize[\s\S]*span:\s*Span[\s\S]*source:\s*TypeExpectationSource[\s\S]*}/);
assert.match(typeExpectation, /fn\s+call_result_expectation_after_args\s*\(/);
assert.match(typeExpectation, /TypeExpectationSource::ExplicitAscription\s*=>\s*self\.span/);

for (const [name, source] of [
    ['prefix_check.rs', prefixCheck],
    ['call_reduction.rs', callReduction],
]) {
    assert.doesNotMatch(
        source,
        /Option<\s*\(\s*TypeId\s*,\s*usize\s*\)\s*>/,
        `${name} must not encode type expectations as Option<(TypeId, usize)>`,
    );
}

for (const [name, source] of [
    ['function_apply.rs', functionApply],
    ['overload_selection.rs', overloadSelection],
    ['selected_call_apply.rs', selectedCallApply],
    ['indirect_apply.rs', indirectApply],
    ['trait_call_apply.rs', traitCallApply],
]) {
    assert.doesNotMatch(
        source,
        /expected_ret:\s*Option<\s*TypeId\s*>/,
        `${name} must keep call expected return evidence as TypeExpectation`,
    );
    assert.match(
        source,
        /expected_ret:\s*Option<\s*TypeExpectation\s*>/,
        `${name} must accept typed call expectations`,
    );
}

assert.doesNotMatch(
    prefixCheck,
    /pending_ascription\s*=\s*Some\s*\(\s*\(\s*ty\s*,\s*stack\.len\(\)\s*\)\s*\)/,
    'explicit type annotations must construct TypeExpectation instead of a tuple',
);
assert.match(prefixCheck, /TypeExpectation::explicit_ascription/);
assert.match(prefixCheck, /TypeExpectation::block_result/);
assert.match(callResolution, /TypeExpectation::outer_consumer_argument/);
assert.match(callReduction, /call_result_expectation_after_args/);
assert.match(
    overloadSelection,
    /let\s+func_data\s*=\s*match\s+self\.ctx\.get\(binding\.ty\)[\s\S]*let\s+checkpoint\s*=\s*self\.ctx\.checkpoint\(\);[\s\S]*self\.ctx\.instantiate\(binding\.ty\)/,
    'overload selection must classify declared function shape before instantiating candidates',
);
assert.ok(
    overloadSelection.indexOf('let func_data = match self.ctx.get(binding.ty)') <
        overloadSelection.indexOf('let checkpoint = self.ctx.checkpoint();'),
    'overload selection must not checkpoint and instantiate before cheap declared-shape pruning',
);
assert.match(
    overloadSelection,
    /type_pattern_matches\(result,\s*expectation\.target\(\)\)[\s\S]*let\s+checkpoint\s*=\s*self\.ctx\.checkpoint\(\);/,
    'overload selection must use declared result shape before candidate instantiation when expected result is available',
);
assert.match(
    overloadSelection,
    /enum\s+OverloadCandidateRejection\s*{[\s\S]*NotFunction[\s\S]*TypeArgumentCount[\s\S]*CaptureArity[\s\S]*UserArity[\s\S]*DeclaredExpectedResult[\s\S]*InstantiatedNotFunction[\s\S]*ArgumentType[\s\S]*ExpectedResult[\s\S]*}/,
    'overload candidate rejection reasons must be a typed enum',
);
assert.match(
    overloadSelection,
    /OverloadCandidateRejection::GenericConstraintConflict/,
    'overload candidate rejection reasons must include generic constraint conflict',
);
assert.match(
    overloadSelection,
    /fn\s+record_rejection\(&mut self,\s*reason:\s*OverloadCandidateRejection\)[\s\S]*match\s+reason\s*{[\s\S]*OverloadCandidateRejection::NotFunction[\s\S]*OverloadCandidateRejection::ExpectedResult[\s\S]*}/,
    'overload candidate rejection stats must dispatch through exhaustive match',
);
assert.match(
    overloadSelection,
    /fn\s+assert_materialization_guard\(&self\)[\s\S]*debug_assert!\(self\.materialized\s*\+\s*self\.pre_materialized_rejections\(\)\s*<=\s*self\.considered\)/,
    'overload selection must guard candidate materialization count',
);
assert.match(
    overloadSelection,
    /stats\.record_materialized\(\);[\s\S]*self\.ctx\.instantiate\(binding\.ty\)/,
    'overload selection must count candidates that reach instantiation/materialization',
);
assert.match(
    overloadSelection,
    /enum\s+OverloadCandidateNarrowingStage\s*{[\s\S]*InitialCandidates[\s\S]*PreferPureFunction[\s\S]*SignatureDedup[\s\S]*PreferOrdinaryFunction[\s\S]*PreferConcreteSignature[\s\S]*PreferFewerTypeParameters[\s\S]*PreferInstantiatedSpecificity[\s\S]*PreferDeclaredSpecificity[\s\S]*}/,
    'overload ambiguity narrowing stages must be a typed enum',
);
assert.match(
    overloadSelection,
    /struct\s+OverloadAmbiguityReason\s*{[\s\S]*after_stage:\s*OverloadCandidateNarrowingStage[\s\S]*remaining_candidates:\s*usize[\s\S]*}/,
    'overload ambiguity must carry a typed reason payload',
);
assert.match(
    overloadSelection,
    /OverloadAmbiguityReason::after_stage\(last_narrowing_stage,\s*candidates\.len\(\)\)[\s\S]*ambiguity\.diagnostic_message\(\)/,
    'overload ambiguity diagnostics must be produced from the typed payload',
);
assert.match(
    traitCheck,
    /enum\s+TypeParamInferenceSource\s*{[\s\S]*Argument[\s\S]*ExpectedResult[\s\S]*}/,
    'trait type-parameter inference constraints must keep their source as a typed enum',
);
assert.match(
    traitCheck,
    /struct\s+TypeParamInferenceConstraint\s*{[\s\S]*source:\s*TypeParamInferenceSource[\s\S]*original:\s*TypeId[\s\S]*actual:\s*TypeId[\s\S]*}/,
    'trait type-parameter inference must use a structured constraint object',
);
assert.match(
    typeArgumentInference,
    /enum\s+TypeArgumentInference\s*{[\s\S]*NoEvidence[\s\S]*Unique\(TypeId\)[\s\S]*Conflict\(TypeArgumentConflict\)[\s\S]*}/,
    'type-argument inference must use a shared typed result model',
);
assert.match(
    typeArgumentInference,
    /struct\s+TypeArgumentResolution\s*{[\s\S]*resolved_args:\s*Vec<TypeId>[\s\S]*conflicts:\s*Vec<TypeArgumentConflict>[\s\S]*}/,
    'type-argument resolution must return typed conflict payloads',
);
assert.match(
    typeArgumentInference,
    /fn\s+resolve_type_arguments_from_constraints\([\s\S]*match\s+inference\s*{[\s\S]*TypeArgumentInference::NoEvidence[\s\S]*TypeArgumentInference::Unique[\s\S]*TypeArgumentInference::Conflict/,
    'type-argument resolution must branch through exhaustive typed inference states',
);
assert.match(
    traitCheck,
    /expected_ret:\s*Option<\s*TypeExpectation\s*>/,
    'trait application inference must keep expected return evidence typed',
);
assert.match(
    traitCheck,
    /resolve_type_arguments_from_constraints\([\s\S]*trait_info\.type_params\.clone\(\)/,
    'trait application inference must use the shared type-argument constraint resolver',
);
assert.doesNotMatch(
    traitCheck,
    /merge_inferred_instantiation/,
    'trait application inference must not collapse conflict and no-evidence through Option merging',
);
assert.doesNotMatch(
    traitCallApply,
    /infer_trait_application_args\([\s\S]*expected_ret\.map\(\|expectation\|\s*expectation\.target\(\)\)/,
    'trait call resolution must not erase TypeExpectation before inference',
);
assert.match(
    traitCallApply,
    /TraitMethodResolution::ConstraintConflict\s*{[\s\S]*TypeDiagnosticCode::TraitConstraintConflict[\s\S]*conflict\.diagnostic_message\(self\.ctx\)/,
    'trait method resolution must report type-argument constraint conflicts from typed payloads',
);
assert.match(
    genericCallConstraints,
    /enum\s+GenericCallConstraintSource\s*{[\s\S]*Argument\s*{[\s\S]*index:\s*usize[\s\S]*ExpectedResult\s*{[\s\S]*expectation:\s*TypeExpectation[\s\S]*}/,
    'generic call constraints must keep argument/result sources in a typed enum',
);
assert.match(
    genericCallConstraints,
    /struct\s+GenericCallConstraint\s*{[\s\S]*source:\s*GenericCallConstraintSource[\s\S]*declared:\s*TypeId[\s\S]*instantiated:\s*TypeId[\s\S]*actual:\s*TypeId[\s\S]*span:\s*Span[\s\S]*}/,
    'generic call constraints must retain declared, instantiated, and actual types',
);
assert.match(
    genericCallConstraints,
    /fn\s+type_argument_constraint\([\s\S]*match\s+self\.source\s*{[\s\S]*GenericCallConstraintSource::Argument[\s\S]*GenericCallConstraintSource::ExpectedResult[\s\S]*TypeArgumentConstraint::new/,
    'generic call type-argument inference must dispatch through the typed constraint source',
);
assert.match(
    selectedCallApply,
    /GenericCallConstraint::expected_result\([\s\S]*for\s+\(idx,\s*\(arg,\s*param_ty\)\)\s+in\s+args\.iter_mut\(\)/,
    'selected generic calls must apply expected-result constraints before argument constraints',
);
assert.doesNotMatch(
    selectedCallApply,
    /ctx\.unify\(c_result,\s*expectation\.target\(\)\)/,
    'selected calls must not bypass GenericCallConstraint for expected-result unification',
);
assert.doesNotMatch(
    selectedCallApply,
    /infer_instantiated_type_arg/,
    'selected calls must derive implicit generic arguments from structured call constraints',
);
assert.match(
    selectedCallApply,
    /TypeDiagnosticCode::GenericConstraintConflict[\s\S]*conflict\.diagnostic_message\(self\.ctx\)/,
    'generic call constraint conflicts must be reported from typed payloads',
);
assert.match(
    overloadSelection,
    /GenericCallConstraint::expected_result\([\s\S]*GenericCallConstraint::argument\(/,
    'overload selection must feed expected-result and argument evidence through GenericCallConstraint',
);
assert.match(
    overloadSelection,
    /resolve_generic_type_args_from_constraints\([\s\S]*TypeDiagnosticCode::GenericConstraintConflict[\s\S]*conflict\.diagnostic_message\(self\.ctx\)/,
    'overload selection must report generic call constraint conflicts from typed payloads',
);
assert.doesNotMatch(
    genericCallConstraints,
    /merge_inferred_instantiation/,
    'generic call inference must not collapse conflict and no-evidence through Option merging',
);

console.log('type expectation model source policy passed');

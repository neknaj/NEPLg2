#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const {
    CHECK_EXPR_FACADE,
    CHECK_EXPR_SPLIT_FILES,
    readCheckExprSource,
    readRepoFile,
} = require("./selfhost_check_expr_sources");

const repoRoot = path.resolve(__dirname, "..");
const source = readCheckExprSource(repoRoot);
const implementation = source
    .split("\n")
    .filter((line) => !line.startsWith("//:"))
    .join("\n");
const moduleChecker = readRepoFile(repoRoot, "stdlib/neplg2/core/check/module.nepl")
    + "\n"
    + readRepoFile(repoRoot, "stdlib/neplg2/core/check/module/orchestrate.nepl");
const parserPrefix = readRepoFile(repoRoot, "stdlib/neplg2/core/syntax/ast/prefix_expr.nepl")
    + "\n"
    + readRepoFile(repoRoot, "stdlib/neplg2/core/syntax/parser/body_segmenter.nepl");
const bodyLine = readRepoFile(repoRoot, "stdlib/neplg2/core/check/expr/body_line.nepl");

for (const relPath of CHECK_EXPR_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/check\/expr\//, "./expr/")
        .replace(/\.nepl$/, "");
    assert.match(
        readRepoFile(repoRoot, CHECK_EXPR_FACADE),
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${CHECK_EXPR_FACADE} must re-export ${importPath}`,
    );
}

assert.match(
    source,
    /pub struct SelfhostTypeExpectation:[\s\S]*expected_type %SelfhostTypeId[\s\S]*source %SelfhostTypeExpectationSource[\s\S]*span %SelfhostSourceSpan/,
    "expected type must preserve type id, source, and span together",
);
assert.match(
    source,
    /pub enum SelfhostGenericInferenceState:[\s\S]*NoneRequired[\s\S]*Unique[\s\S]*EvidenceMissing[\s\S]*Conflict[\s\S]*Unsupported/,
    "generic inference must use an explicit enum instead of an ambiguous optional type",
);
assert.match(
    source,
    /pub enum SelfhostOverloadCandidateRejectionKind:[\s\S]*NotFunction[\s\S]*ArityMismatch[\s\S]*ExpectedResult[\s\S]*UnsupportedGeneric[\s\S]*UnsupportedTraitBound/,
    "overload candidate rejection reasons must remain typed",
);
assert.match(
    source,
    /pub struct SelfhostCallableSignature:[\s\S]*def_id %SelfhostDefId[\s\S]*callable_type %SelfhostTypeId[\s\S]*effect %SelfhostEffectKind[\s\S]*generic_state %SelfhostGenericInferenceState[\s\S]*span %SelfhostSourceSpan/,
    "callable candidate collection must preserve DefId-linked signature evidence",
);
assert.match(
    source,
    /pub struct SelfhostCallableSignatureTable:[\s\S]*entries %Vec SelfhostCallableSignature/,
    "callable candidate collection must use an explicit signature table boundary",
);
assert.match(
    source,
    /pub enum SelfhostCallableCandidateCollectErrorKind:[\s\S]*EmptyPrefix[\s\S]*PrefixBuildFailed[\s\S]*UnsupportedHead[\s\S]*HeadTokenOutOfBounds[\s\S]*PendingBinding[\s\S]*MissingSignature[\s\S]*OutOfMemory/,
    "callable candidate collection errors must fail closed with typed causes",
);
assert.match(
    source,
    /pub fn selfhost_callable_candidates_collect_for_prefix %impure fn str impure fn &Vec SelfhostToken impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostCallableSignatureTable Result Vec SelfhostCallableCandidate SelfhostCallableCandidateCollectError/,
    "callable candidate collection must expose a scope-and-signature-table boundary",
);
assert.match(
    source,
    /selfhost_token_lexeme source token[\s\S]*selfhost_callable_candidates_collect_from_scope_name name scope signatures head\.span/,
    "callable candidate collection must route the prefix head through the scope-name collector",
);
assert.match(
    source,
    /selfhost_name_scope_get scope selfhost_def_id_new idx[\s\S]*string_search::str_eq binding\.name name[\s\S]*selfhost_def_kind_eq binding\.kind SelfhostDefKind::Function[\s\S]*selfhost_callable_candidates_push_binding out binding signatures/,
    "callable candidate collection must scan all same-name function bindings instead of latest-only lookup",
);
assert.match(
    source,
    /selfhost_callable_signature_table_find signatures def_id[\s\S]*selfhost_callable_candidates_push_signature out binding\.name signature binding\.span/,
    "callable candidate collection must use DefId to recover callable signature evidence",
);
assert.match(
    source,
    /selfhost_callable_candidate_new name signature\.callable_type signature\.effect signature\.generic_state signature\.span/,
    "callable candidate collection must build reducer candidates from signature records",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*PartialApplicationRejected[\s\S]*ArgumentTypeMismatch[\s\S]*ArgumentAscriptionProjectionFailed[\s\S]*ArgumentAscriptionExpectedTypeConflict[\s\S]*UnsupportedArgumentExpression[\s\S]*OverloadAmbiguous[\s\S]*GenericInferenceEvidenceMissing[\s\S]*GenericInferenceConflict[\s\S]*ExpectedTypeMismatch/,
    "call reduction errors must distinguish partial application, argument type, argument ascription, unsupported argument expression, overload, generic, and expectation failures",
);
assert.match(
    source,
    /# check\/expr\/argument[\s\S]*pub struct SelfhostExprArgumentMatch:[\s\S]*next_index %i32[\s\S]*pub fn selfhost_expr_argument_item_literal_type_kind[\s\S]*pub fn selfhost_expr_argument_match_at/,
    "argument type evidence and consume-width scanning must live in its own check/expr split module",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::UnitValue:[\s\S]*SelfhostTypeKind::Unit[\s\S]*SelfhostExprPrefixItemKind::IntLiteral:[\s\S]*SelfhostTypeKind::I32[\s\S]*SelfhostExprPrefixItemKind::BoolLiteral:[\s\S]*SelfhostTypeKind::Bool/,
    "argument evidence must map simple literal items to typed primitive evidence",
);
assert.match(
    source,
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*TypeMismatch[\s\S]*UnsupportedAscribedArgument[\s\S]*AscriptionProjectionFailed[\s\S]*AscriptionExpectedTypeConflict[\s\S]*UnsupportedArgumentExpression/,
    "argument expression scan failures must stay typed instead of collapsing into a boolean",
);
assert.match(
    source,
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*AscriptionExpectedTypeConflict/,
    "argument-scope ascription must preserve expected-type conflicts as a typed error",
);
assert.match(
    source,
    /SelfhostExprArgumentMatchErrorKind::UnsupportedAscribedArgument:[\s\S]*SelfhostCallReduceErrorKind::UnsupportedArgumentExpression[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionProjectionFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionProjectionFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionExpectedTypeConflict:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionExpectedTypeConflict/,
    "call reducer must map argument-scope ascription projection and expected-type conflicts separately from generic unsupported argument expressions",
);
assert.match(
    source,
    /pub struct SelfhostExprArgumentOwnedMatch:[\s\S]*arena %SelfhostTypeArena[\s\S]*match_value %SelfhostExprArgumentMatch[\s\S]*pub fn selfhost_expr_argument_match_at_with_source/,
    "source-backed argument checking must return an updated arena owner with consume-width evidence",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::TypeAnnotationMarker:[\s\S]*SelfhostExprArgumentMatchErrorKind::UnsupportedAscribedArgument/,
    "ascribed argument expressions must fail closed until token/source-backed argument checking is implemented",
);
assert.match(
    source,
    /pub fn selfhost_expr_ascription_project_head_expectation %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn SelfhostSyntaxRange Result SelfhostExprAscriptionHeadProjection SelfhostExprAscriptionError/,
    "argument-scope ascription must expose a head projection that returns the first expression token",
);
assert.match(
    source,
    /selfhost_type_arena_function_arg arena candidate\.callable_type param_idx[\s\S]*selfhost_expr_argument_match_at arena prefix item_index item_count expected_arg_type[\s\S]*argument_match\.next_index/,
    "call reduction must check each consumed argument expression against the candidate parameter type",
);
assert.match(
    source,
    /selfhost_type_arena_function_arg &arena candidate\.callable_type param_idx[\s\S]*selfhost_expr_argument_match_at_with_source tokens source arena prefix item_index item_count expected_arg_type[\s\S]*argument_match\.next_index/,
    "source-backed call reduction must check consumed argument expressions with token/source-backed argument matching",
);
assert.match(
    source,
    /pub enum SelfhostExpressionLineCheckError:[\s\S]*NotExpressionLine %SelfhostSourceSpan[\s\S]*PrefixBuildFailed %SelfhostExprPrefixBuildError[\s\S]*CallReduceFailed %SelfhostCallReduceError/,
    "expression line connector must preserve segment, prefix-build, and call-reduction failures separately",
);
assert.match(
    source,
    /pub enum SelfhostExpressionLineCheckError:[\s\S]*AscriptionFailed %SelfhostExprAscriptionError[\s\S]*AscriptionExpectedTypeConflict %SelfhostAscriptionExpectedTypeConflict/,
    "expression line connector must preserve type-ascription failures and ascription-vs-outer expectation conflicts separately",
);
assert.match(
    source,
    /pub struct SelfhostAscriptionExpectedTypeConflict:[\s\S]*ascription_source %SelfhostTypeExpectationSource[\s\S]*ascription_span %SelfhostSourceSpan[\s\S]*outer_source %SelfhostTypeExpectationSource[\s\S]*outer_span %SelfhostSourceSpan/,
    "ascription conflict error must not return arena-local TypeIds after the owner arena is freed",
);
assert.match(
    source,
    /pub enum SelfhostExprAscriptionError:[\s\S]*NotTypeAscription %SelfhostSourceSpan[\s\S]*TypeReduceFailed %SelfhostTypeReduceError[\s\S]*TypeProjectFailed %SelfhostTypeProjectError[\s\S]*MissingExpressionTail %SelfhostSourceSpan/,
    "type ascription connector must keep typed range, reduce, project, and missing-tail failures",
);
assert.match(
    source,
    /pub fn selfhost_call_reduce_prefix %fn &SelfhostTypeArena fn &SelfhostExprPrefixList fn &Vec SelfhostCallableCandidate fn Option SelfhostTypeExpectation Result SelfhostCallReduceResult SelfhostCallReduceError/,
    "call reduction input must keep expected type as Option SelfhostTypeExpectation",
);
assert.match(
    source,
    /pub struct SelfhostCallReduceOwnedResult:[\s\S]*arena %SelfhostTypeArena[\s\S]*result %SelfhostCallReduceResult[\s\S]*pub fn selfhost_call_reduce_prefix_with_source %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostCallReduceOwnedResult SelfhostCallReduceError/,
    "source-backed call reduction must expose an arena-owner boundary separate from the borrowed reducer",
);
assert.match(
    source,
    /pub fn selfhost_expr_ascription_project_expectation %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn SelfhostSyntaxRange Result SelfhostExprAscriptionProjection SelfhostExprAscriptionError/,
    "type ascription must expose an arena-owner projection boundary",
);
assert.match(
    source,
    /pub fn selfhost_check_expr_reduce_body_segment_with_arena %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostBodySegment impure fn SelfhostTypeArena impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostExpressionLineCheckSuccess SelfhostExpressionLineCheckError/,
    "body segment connector must expose an arena-owner boundary for ascription projection",
);
assert.match(
    source,
    /pub fn selfhost_check_expr_reduce_body_segment %impure fn &Vec SelfhostToken impure fn SelfhostBodySegment impure fn &SelfhostTypeArena impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostCallReduceResult SelfhostExpressionLineCheckError/,
    "body segment connector must expose a typed expression-line reduction boundary",
);
assert.match(
    source,
    /ge item_index item_count[\s\S]*SelfhostCallReduceErrorKind::PartialApplicationRejected/,
    "argument shortage must reject partial application instead of producing a function value",
);
assert.match(
    source,
    /ge param_idx param_count[\s\S]*eq item_index item_count[\s\S]*SelfhostCallReduceErrorKind::OverloadNoCandidate/,
    "extra argument expressions must be detected after consume-width scanning, not by raw item count",
);
assert.doesNotMatch(
    implementation,
    /let\s+argument_count\s+%i32\s+sub\s+item_count\s+1/,
    "call reduction must not derive arity from raw prefix item count",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::BoolLiteral[\s\S]*SelfhostCallReduceErrorKind::ArgumentTypeMismatch/,
    "stage0 must include a mismatched literal argument rejection smoke check",
);
assert.match(
    source,
    /selfhost_check_expr_stage0_make_prefix_with_ascribed_first_arg[\s\S]*SelfhostExprPrefixItemKind::TypeAnnotationMarker[\s\S]*selfhost_check_expr_stage0_ascribed_argument_unsupported[\s\S]*SelfhostCallReduceErrorKind::UnsupportedArgumentExpression/,
    "stage0 must confirm that ascribed argument expressions fail closed without raw arity misclassification",
);
assert.match(
    source,
    /gt candidate_count 1[\s\S]*SelfhostCallReduceErrorKind::OverloadAmbiguous/,
    "multiple accepted candidates must remain ambiguous in the initial slice",
);
assert.match(
    source,
    /SelfhostGenericInferenceState::EvidenceMissing:[\s\S]*GenericInferenceEvidenceMissing[\s\S]*SelfhostGenericInferenceState::Conflict:[\s\S]*GenericInferenceConflict[\s\S]*SelfhostGenericInferenceState::Unsupported:[\s\S]*GenericInferenceUnsupported/,
    "generic inference failure states must fail closed with distinct errors",
);
assert.match(
    bodyLine,
    /pub fn selfhost_check_expr_reduce_body_segment[\s\S]*SelfhostBodySegmentKind::ExpressionLine:[\s\S]*selfhost_check_expr_reduce_expression_line_prefix tokens segment\.head arena candidates expected/,
    "borrowed ExpressionLine.head must remain routed to the source-less expression-line prefix reducer",
);
assert.match(
    source,
    /selfhost_name_scope_add_binding scope0 binding[\s\S]*selfhost_callable_signature_table_add table0 signature[\s\S]*selfhost_expr_prefix_list_from_syntax_range tokens segment\.head[\s\S]*selfhost_callable_candidates_collect_for_prefix source tokens &prefix &scope1 &table1/,
    "stage1 smoke path must collect direct call candidates through scope and signature tables",
);
assert.match(
    bodyLine,
    /selfhost_expr_prefix_list_from_syntax_range tokens head[\s\S]*selfhost_call_reduce_prefix arena &prefix candidates expected[\s\S]*selfhost_expr_prefix_list_free prefix/,
    "expression-line prefix reducer must build and free a prefix list around call reduction",
);
assert.match(
    bodyLine,
    /selfhost_expr_prefix_list_from_syntax_range tokens head[\s\S]*selfhost_call_reduce_prefix_with_source tokens source arena &prefix candidates expected[\s\S]*selfhost_expr_prefix_list_free prefix/,
    "owner expression-line prefix reducer must build and free a prefix list around source-backed call reduction",
);
assert.match(
    bodyLine,
    /selfhost_check_expr_head_starts_with_percent tokens segment\.head[\s\S]*selfhost_expr_ascription_project_expectation tokens source arena segment\.head[\s\S]*selfhost_check_expr_reduce_body_segment_with_projected_ascription tokens source projection candidates expected/,
    "percent-prefixed expression lines must be projected as type ascriptions before call reduction",
);
assert.match(
    bodyLine,
    /selfhost_check_expr_validate_ascription_outer_expected[\s\S]*selfhost_type_arena_types_equal arena ascription_expectation\.expected_type outer_expectation\.expected_type[\s\S]*selfhost_ascription_expected_type_conflict_new ascription_expectation outer_expectation[\s\S]*SelfhostExpressionLineCheckError::AscriptionExpectedTypeConflict conflict/,
    "ascription projection must reject conflicts with an outer expected type before reducing the inner expression",
);
assert.doesNotMatch(
    bodyLine,
    /AscriptionExpectedTypeConflict %SelfhostTypeExpectation %SelfhostTypeExpectation|SelfhostExpressionLineCheckError::AscriptionExpectedTypeConflict\s+ascription_expectation\s+outer_expectation/,
    "ascription conflict errors must not expose arena-local expectation TypeIds after freeing the arena",
);
assert.match(
    source,
    /selfhost_type_prefix_list_reduce_prefix source &type_prefix[\s\S]*SelfhostTypeExpectationSource::ExplicitAscription[\s\S]*selfhost_expr_ascription_projection_new allocated expectation tail/,
    "type ascription projection must use prefix-boundary reduction and explicit expectation evidence",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_ascription_conflict_rejected_with_candidate[\s\S]*SelfhostTypeExpectationSource::OuterConsumerArgument[\s\S]*selfhost_expression_line_check_error_is_ascription_expected_type_conflict/,
    "stage1 must smoke-test that explicit ascription and outer expected type conflicts stay typed",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_make_ascribed_argument_i32_tokens[\s\S]*"add %i32 1 2"[\s\S]*selfhost_check_expr_stage1_ascribed_argument_ok_with_scope[\s\S]*selfhost_check_expr_stage1_success_is_two_arg_direct_call/,
    "stage1 must smoke-test that argument-scope i32 ascription succeeds through the source-backed reducer",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_make_ascribed_argument_bool_tokens[\s\S]*"add %bool 1 2"[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionExpectedTypeConflict/,
    "stage1 must smoke-test that argument-scope ascription conflicts keep a typed call-reduction error",
);
assert.match(
    bodyLine,
    /SelfhostBodySegmentKind::BlockIntro:[\s\S]*SelfhostExpressionLineCheckError::NotExpressionLine/,
    "BlockIntro must be rejected instead of being flattened through expression reduction",
);
assert.match(
    bodyLine,
    /borrowed API[\s\S]*selfhost_check_expr_reduce_body_segment_with_arena/,
    "borrowed expression-line connector must document that percent ascription uses the owner-returning API",
);
assert.doesNotMatch(
    bodyLine,
    /selfhost_expr_prefix_list_from_syntax_range\s+tokens\s+segment\.body/,
    "BlockIntro.body must not be passed directly to flat prefix expression reduction",
);
assert.doesNotMatch(
    implementation,
    /Option\s+SelfhostTypeId[\s\S]{0,80}(expected|Expectation)|expected[\s\S]{0,80}Option\s+SelfhostTypeId/,
    "expected type must not be represented as a bare Option SelfhostTypeId",
);
assert.doesNotMatch(
    implementation,
    /SelfhostHirExpr|SelfhostHirExprPayload|selfhost_hir_expr_call/,
    "initial call reduction must not allocate or mutate HIR directly",
);
assert.doesNotMatch(
    moduleChecker,
    /selfhost_call_reduce_prefix|SelfhostCallReduce|SelfhostTypeExpectation|SelfhostCallableCandidate/,
    "module item checker must not own expression call reduction",
);
assert.doesNotMatch(
    parserPrefix,
    /selfhost_call_reduce_prefix|SelfhostCallReduce|SelfhostTypeExpectation|SelfhostCallableCandidate/,
    "parser and prefix input modules must not depend on checker call reduction",
);

for (const relPath of CHECK_EXPR_SPLIT_FILES) {
    const file = readRepoFile(repoRoot, relPath);
    assert.doesNotMatch(file, /#import "\.\.\/expr" as \*|#import "neplg2\/core\/check\/expr" as \*/, `${relPath} must not import the expr facade`);
}

assert.ok(
    fs.existsSync(path.join(repoRoot, "tests/stdlib/neplg2_call_reduce.n.md")),
    "focused call reduction doctest must exist",
);

console.log("selfhost expression call reduction contract passed");

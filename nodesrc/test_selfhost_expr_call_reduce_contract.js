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
    /pub struct SelfhostTrailingBlockArgument:[\s\S]*body %SelfhostSyntaxRange[\s\S]*span %SelfhostSourceSpan/,
    "trailing block arguments must be represented as body-envelope evidence instead of prefix items",
);
assert.match(
    source,
    /pub struct SelfhostCallableSignatureTable:[\s\S]*entries %Vec SelfhostCallableSignature/,
    "callable candidate collection must use an explicit signature table boundary",
);
assert.match(
    source,
    /pub struct SelfhostValueTypeEvidence:[\s\S]*def_id %SelfhostDefId[\s\S]*value_type %SelfhostTypeId[\s\S]*span %SelfhostSourceSpan[\s\S]*pub struct SelfhostValueTypeEvidenceTable:[\s\S]*entries %Vec SelfhostValueTypeEvidence/,
    "named value arguments must use an explicit DefId-linked value type evidence table",
);
assert.match(
    source,
    /pub fn selfhost_value_type_evidence_table_find %fn &SelfhostValueTypeEvidenceTable fn SelfhostDefId Option SelfhostValueTypeEvidence/,
    "value type evidence lookup must be keyed by DefId instead of raw source spelling",
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
    /pub fn selfhost_callable_candidates_collect_for_head_item %impure fn str impure fn &Vec SelfhostToken impure fn SelfhostExprPrefixItem impure fn &SelfhostNameScope impure fn &SelfhostCallableSignatureTable Result Vec SelfhostCallableCandidate SelfhostCallableCandidateCollectError/,
    "nested named call arguments must collect candidates from their own head item instead of rebuilding a top-level prefix",
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
    /fn selfhost_callable_candidates_collect_from_scope_name[\s\S]*selfhost_name_scope_find scope name[\s\S]*selfhost_def_kind_eq latest\.kind SelfhostDefKind::Function[\s\S]*selfhost_callable_candidates_collect_scope_loop name scope signatures[\s\S]*selfhost_callable_candidates_empty head_span/,
    "callable candidate collection must respect latest visible binding before collecting overload functions",
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
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*PartialApplicationRejected[\s\S]*ArgumentTypeMismatch[\s\S]*ArgumentAscriptionProjectionFailed[\s\S]*ArgumentAscriptionExpectedTypeConflict[\s\S]*ArgumentNamedValueUnresolved[\s\S]*ArgumentNamedValuePendingBinding[\s\S]*ArgumentNamedValueUnsupportedBinding[\s\S]*ArgumentNamedValueEvidenceMissing[\s\S]*UnsupportedArgumentExpression[\s\S]*OverloadAmbiguous[\s\S]*GenericInferenceEvidenceMissing[\s\S]*GenericInferenceConflict[\s\S]*ExpectedTypeMismatch/,
    "call reduction errors must distinguish partial application, argument type, argument ascription, named value evidence, unsupported argument expression, overload, generic, and expectation failures",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*ArgumentNestedCandidatePendingBinding[\s\S]*ArgumentNestedCandidateMissingSignature[\s\S]*ArgumentNestedCandidateHeadTokenOutOfBounds[\s\S]*ArgumentNestedCandidateOutOfMemory[\s\S]*ArgumentNestedCandidateInternalInvariant/,
    "nested named call candidate collection failures must remain typed at the call-reduction boundary",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*TrailingBlockBodySegmentFailed[\s\S]*TrailingBlockBodyEmpty[\s\S]*TrailingBlockBodyMultipleSegments[\s\S]*TrailingBlockBodyNestedBlockUnsupported[\s\S]*TrailingBlockBodyPrefixBuildFailed[\s\S]*TrailingBlockBodyCandidateCollectFailed[\s\S]*UnexpectedTrailingBlockArgument[\s\S]*UnsupportedArgumentExpression/,
    "trailing block argument body failures must stay typed instead of collapsing into partial application or generic unsupported argument errors",
);
assert.match(
    source,
    /# check\/expr\/block_body[\s\S]*pub struct SelfhostBlockBodyResultInput:[\s\S]*prefix %SelfhostExprPrefixList[\s\S]*candidates %Vec SelfhostCallableCandidate[\s\S]*span %SelfhostSourceSpan/,
    "trailing block body checking must own a prefix list and candidate list boundary before recursive reduction",
);
assert.match(
    source,
    /selfhost_body_segment_list_from_envelope tokens block_arg\.body[\s\S]*selfhost_block_body_result_from_segment_list tokens source scope signatures segments block_arg\.span/,
    "trailing block body checking must segment the nested body envelope before building prefix input",
);
assert.match(
    source,
    /selfhost_body_segment_list_len &segments[\s\S]*SelfhostBlockBodyResultErrorKind::EmptyBody[\s\S]*SelfhostBlockBodyResultErrorKind::MultipleSegments[\s\S]*selfhost_body_segment_list_get &segments 0/,
    "trailing block body checking must reject empty and multi-segment bodies before call reduction",
);
assert.match(
    source,
    /SelfhostBodySegmentKind::ExpressionLine:[\s\S]*selfhost_block_body_result_from_expression_segment tokens source scope signatures segment[\s\S]*SelfhostBodySegmentKind::BlockIntro:[\s\S]*SelfhostBlockBodyResultErrorKind::NestedBlockUnsupported/,
    "trailing block body checking must reject nested BlockIntro bodies in this slice",
);
assert.match(
    source,
    /selfhost_expr_prefix_list_from_syntax_range tokens segment\.head[\s\S]*selfhost_callable_candidates_collect_for_prefix source tokens &prefix scope signatures/,
    "trailing block body checking must build candidates from the nested expression head through scope and signatures",
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
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*TypeMismatch[\s\S]*UnsupportedAscribedArgument[\s\S]*AscriptionProjectionFailed[\s\S]*AscriptionExpectedTypeConflict[\s\S]*NamedValueUnresolved[\s\S]*NamedValuePendingBinding[\s\S]*NamedValueUnsupportedBinding[\s\S]*NamedValueEvidenceMissing[\s\S]*UnsupportedArgumentExpression/,
    "argument expression scan failures must stay typed instead of collapsing into a boolean",
);
assert.match(
    source,
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*AscriptionExpectedTypeConflict/,
    "argument-scope ascription must preserve expected-type conflicts as a typed error",
);
assert.match(
    source,
    /SelfhostExprArgumentMatchErrorKind::UnsupportedAscribedArgument:[\s\S]*SelfhostCallReduceErrorKind::UnsupportedArgumentExpression[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionProjectionFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionProjectionFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionExpectedTypeConflict:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionExpectedTypeConflict[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValueUnresolved:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueUnresolved[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValuePendingBinding:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValuePendingBinding[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValueUnsupportedBinding:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueUnsupportedBinding[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValueEvidenceMissing:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueEvidenceMissing/,
    "call reducer must map argument-scope ascription and named value evidence failures separately from generic unsupported argument expressions",
);
assert.match(
    source,
    /pub struct SelfhostExprArgumentOwnedMatch:[\s\S]*arena %SelfhostTypeArena[\s\S]*match_value %SelfhostExprArgumentMatch[\s\S]*pub fn selfhost_expr_argument_match_at_with_source %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable/,
    "source-backed argument checking must return an updated arena owner with consume-width evidence",
);
assert.match(
    source,
    /selfhost_name_scope_find scope name[\s\S]*selfhost_expr_argument_named_value_type_from_binding value_types binding/,
    "source-backed named value arguments must resolve through scope, DefId-linked value evidence, and arena structural equality",
);
assert.match(
    source,
    /selfhost_value_type_evidence_table_find value_types def_id[\s\S]*Result::Ok evidence\.value_type/,
    "named value type evidence must be recovered from the DefId-linked table",
);
assert.match(
    source,
    /selfhost_type_arena_types_equal &arena actual_type expected_type/,
    "named value argument types must be compared through arena structural equality",
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
    /fn selfhost_call_reduce_argument_match_direct_with_source[\s\S]*selfhost_expr_argument_match_at_with_source tokens source arena prefix scope value_types item_index item_count expected_type[\s\S]*selfhost_call_reduce_error_from_argument_match_owned argument_error head/,
    "source-backed direct argument fallback must still use token/source-backed argument matching",
);
assert.match(
    source,
    /selfhost_type_arena_function_arg &arena candidate\.callable_type param_idx[\s\S]*selfhost_call_reduce_argument_match_at_with_source_or_nested tokens source arena prefix scope value_types signatures item_index item_count expected_arg_type head[\s\S]*argument_match\.next_index/,
    "source-backed call reduction must route each consumed argument through the direct-or-nested argument boundary",
);
assert.match(
    source,
    /SelfhostCallableCandidateCollectErrorKind::PendingBinding:[\s\S]*ArgumentNestedCandidatePendingBinding[\s\S]*SelfhostCallableCandidateCollectErrorKind::MissingSignature:[\s\S]*ArgumentNestedCandidateMissingSignature[\s\S]*SelfhostCallableCandidateCollectErrorKind::HeadTokenOutOfBounds:[\s\S]*ArgumentNestedCandidateHeadTokenOutOfBounds[\s\S]*SelfhostCallableCandidateCollectErrorKind::OutOfMemory:[\s\S]*ArgumentNestedCandidateOutOfMemory/,
    "nested candidate collection failures must not collapse into UnsupportedArgumentExpression",
);
assert.match(
    source,
    /selfhost_callable_candidates_collect_for_head_item source tokens item scope signatures[\s\S]*selfhost_call_reduce_nested_named_candidates_with_source tokens source arena prefix scope value_types signatures nested_candidates item_index item_count expected_type item/,
    "nested named call arguments must collect inner candidates from the argument head and reduce them with the outer expected argument type",
);
assert.match(
    source,
    /eq candidate_count 0[\s\S]*selfhost_call_reduce_argument_match_direct_with_source tokens source arena prefix scope value_types item_index item_count expected_type head[\s\S]*gt candidate_count 1[\s\S]*SelfhostCallReduceErrorKind::OverloadAmbiguous[\s\S]*selfhost_call_reduce_nested_single_named_candidate_with_source tokens source arena prefix scope value_types signatures candidate item_index item_count expected_type head/,
    "named argument handling must fall back to value evidence only when no visible function candidates exist",
);
assert.match(
    source,
    /fn selfhost_call_reduce_argument_consume_loop_with_source[\s\S]*ge param_idx param_count[\s\S]*selfhost_expr_argument_owned_match_new arena selfhost_expr_argument_match_new item_index[\s\S]*selfhost_call_reduce_argument_match_at_with_source_or_nested tokens source arena prefix scope value_types signatures item_index item_count expected_arg_type head/,
    "nested call reduction must return the consumed next_index for the enclosing argument loop",
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
    /pub struct SelfhostCallReduceOwnedResult:[\s\S]*arena %SelfhostTypeArena[\s\S]*result %SelfhostCallReduceResult[\s\S]*pub fn selfhost_call_reduce_prefix_with_source %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostCallReduceOwnedResult SelfhostCallReduceError/,
    "source-backed call reduction must expose an arena-owner boundary with value and callable evidence separate from the borrowed reducer",
);
assert.match(
    source,
    /pub fn selfhost_call_reduce_prefix_with_source_and_trailing_block %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation impure fn Option SelfhostTrailingBlockArgument Result SelfhostCallReduceOwnedResult SelfhostCallReduceError/,
    "source-backed call reduction must expose a dedicated trailing block argument boundary",
);
assert.match(
    source,
    /pub fn selfhost_expr_ascription_project_expectation %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn SelfhostSyntaxRange Result SelfhostExprAscriptionProjection SelfhostExprAscriptionError/,
    "type ascription must expose an arena-owner projection boundary",
);
assert.match(
    source,
    /pub fn selfhost_check_expr_reduce_body_segment_with_arena %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostBodySegment impure fn SelfhostTypeArena impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostExpressionLineCheckSuccess SelfhostExpressionLineCheckError/,
    "body segment connector must expose an arena-owner boundary for ascription projection and source-backed nested calls",
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
    /selfhost_expr_prefix_list_from_syntax_range tokens head[\s\S]*selfhost_call_reduce_prefix_with_source tokens source arena &prefix scope value_types signatures candidates expected[\s\S]*selfhost_expr_prefix_list_free prefix/,
    "owner expression-line prefix reducer must build and free a prefix list around source-backed call reduction",
);
assert.match(
    bodyLine,
    /selfhost_check_expr_head_starts_with_percent tokens segment\.head[\s\S]*selfhost_expr_ascription_project_expectation tokens source arena segment\.head[\s\S]*selfhost_check_expr_reduce_body_segment_with_projected_ascription tokens source projection scope value_types signatures candidates expected/,
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
    source,
    /selfhost_check_expr_stage1_make_named_argument_tokens[\s\S]*"add x 2"[\s\S]*selfhost_check_expr_stage1_value_context_with_typed_value "x" i32_type[\s\S]*selfhost_check_expr_stage1_success_is_two_arg_direct_call/,
    "stage1 must smoke-test that named value arguments succeed only with typed value evidence",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_make_nested_call_argument_tokens[\s\S]*"add add 1 2 3"[\s\S]*selfhost_check_expr_stage1_nested_call_argument_ok_with_scope[\s\S]*selfhost_check_expr_stage1_nested_call_argument_body_line/,
    "stage1 must smoke-test that nested named call arguments consume only their own inner call",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_trailing_block_argument_segment[\s\S]*SelfhostBodySegmentKind::BlockIntro[\s\S]*selfhost_check_expr_stage1_trailing_block_argument_result_ok[\s\S]*"add 1:\\n    add 1 1"[\s\S]*selfhost_check_expr_stage1_success_is_two_arg_direct_call/,
    "stage1 must smoke-test that a trailing block argument is checked as a BlockResult expression",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_value_context_with_shadowed_function_value[\s\S]*SelfhostDefKind::Function[\s\S]*SelfhostDefKind::Local[\s\S]*selfhost_check_expr_stage1_make_shadowed_function_argument_tokens[\s\S]*"add add 2"[\s\S]*selfhost_check_expr_stage1_shadowed_function_argument_uses_value_evidence/,
    "stage1 must smoke-test that a latest local binding shadows an older same-name function candidate",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_value_context_with_binding_only "x"[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueEvidenceMissing/,
    "stage1 must smoke-test that a named binding without value type evidence fails closed",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_make_ascribed_named_argument_tokens[\s\S]*"add %i32 x 2"[\s\S]*selfhost_check_expr_stage1_ascribed_named_argument_ok_with_scope/,
    "stage1 must smoke-test argument-scope ascription with a named value tail",
);
assert.match(
    bodyLine,
    /SelfhostBodySegmentKind::BlockIntro:[\s\S]*SelfhostExpressionLineCheckError::NotExpressionLine/,
    "BlockIntro must be rejected instead of being flattened through expression reduction",
);
assert.match(
    bodyLine,
    /pub fn selfhost_check_expr_reduce_block_intro_with_arena %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostBodySegment impure fn SelfhostTypeArena impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostExpressionLineCheckSuccess SelfhostExpressionLineCheckError/,
    "BlockIntro must have a dedicated owner-returning reduction boundary",
);
assert.match(
    bodyLine,
    /selfhost_trailing_block_argument_new segment\.body block_span[\s\S]*selfhost_call_reduce_prefix_with_source_and_trailing_block tokens source arena &prefix scope value_types signatures candidates expected some trailing/,
    "BlockIntro reduction must pass the body envelope as trailing-block evidence without flattening it",
);
assert.match(
    source,
    /selfhost_call_reduce_trailing_block_body_result[\s\S]*selfhost_block_body_result_input_from_trailing_block tokens source scope signatures block_arg[\s\S]*SelfhostTypeExpectationSource::BlockResult[\s\S]*selfhost_call_reduce_prefix_with_source tokens source arena prefix scope value_types signatures candidates some block_expected/,
    "call reduction must recursively check a trailing block body with a BlockResult expectation",
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
    /selfhost_expr_prefix_list_from_syntax_range\s+tokens\s+block_arg\.body/,
    "trailing block body must be segmented before any prefix list is built",
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

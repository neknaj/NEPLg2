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
const argumentPayload = readRepoFile(repoRoot, "stdlib/neplg2/core/check/expr/argument_payload.nepl");
const literalPayload = readRepoFile(repoRoot, "stdlib/neplg2/core/check/expr/literal_payload.nepl");
const pipeCandidatesStart = source.indexOf("fn selfhost_call_reduce_pipe_candidates_with_source ");
assert.notEqual(pipeCandidatesStart, -1, "pipe candidate reducer function must exist");
const pipeCandidatesEnd = source.indexOf("//: selfhost_call_reduce_pipe_ascribed_target_with_source", pipeCandidatesStart);
assert.notEqual(pipeCandidatesEnd, -1, "pipe candidate reducer must remain before ascribed target reducer");
const pipeCandidates = source.slice(pipeCandidatesStart, pipeCandidatesEnd);

function assertLiteralPayloadDoc(name, requiredParts) {
    const marker = `//: ${name}:`;
    const markerIndex = literalPayload.indexOf(marker);
    assert.notEqual(markerIndex, -1, `${name} must have a named doc comment`);
    const fnIndex = literalPayload.indexOf(`fn ${name} `, markerIndex);
    assert.notEqual(fnIndex, -1, `${name} doc comment must be immediately before its function declaration`);
    const doc = literalPayload.slice(markerIndex, fnIndex);
    for (const part of requiredParts) {
        assert.ok(doc.includes(part), `${name} doc comment must include ${part}`);
    }
}

function assertArgumentPayloadDoc(name, declarationKind, requiredParts) {
    const marker = `//: ${name}:`;
    const markerIndex = argumentPayload.indexOf(marker);
    assert.notEqual(markerIndex, -1, `${name} must have a named doc comment`);
    const declIndex = argumentPayload.indexOf(`${declarationKind} ${name}`, markerIndex);
    assert.notEqual(declIndex, -1, `${name} doc comment must be immediately before ${declarationKind}`);
    const doc = argumentPayload.slice(markerIndex, declIndex);
    for (const part of requiredParts) {
        assert.ok(doc.includes(part), `${name} doc comment must include ${part}`);
    }
}

function assertContainsInOrder(text, parts, message) {
    let cursor = 0;
    for (const part of parts) {
        const next = text.indexOf(part, cursor);
        assert.notEqual(next, -1, `${message}: missing ${part}`);
        cursor = next + part.length;
    }
}

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
    /pub enum SelfhostTypeExpectationSource:[\s\S]*ExplicitAscription[\s\S]*BlockResult[\s\S]*BlockSequenceDiscardedExpression[\s\S]*OuterConsumerArgument/,
    "expected type source must distinguish explicit ascription, block result, discarded block-sequence expressions, and outer call arguments",
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
    /pub struct SelfhostCallableCandidate:[\s\S]*name %str[\s\S]*def_id %SelfhostDefId[\s\S]*callable_type %SelfhostTypeId[\s\S]*effect %SelfhostEffectKind[\s\S]*generic_state %SelfhostGenericInferenceState[\s\S]*span %SelfhostSourceSpan/,
    "call reducer candidates must preserve DefId evidence for later HIR function value identity lowering",
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
    /selfhost_callable_candidate_new name signature\.def_id signature\.callable_type signature\.effect signature\.generic_state signature\.span/,
    "callable candidate collection must build reducer candidates from signature records without dropping DefId evidence",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*PartialApplicationRejected[\s\S]*ArgumentTypeMismatch[\s\S]*ArgumentAscriptionProjectionFailed[\s\S]*ArgumentAscriptionExpectedTypeConflict[\s\S]*ArgumentNamedValueUnresolved[\s\S]*ArgumentNamedValuePendingBinding[\s\S]*ArgumentNamedValueUnsupportedBinding[\s\S]*ArgumentNamedValueEvidenceMissing[\s\S]*UnsupportedArgumentExpression[\s\S]*OverloadAmbiguous[\s\S]*GenericInferenceEvidenceMissing[\s\S]*GenericInferenceConflict[\s\S]*ExpectedTypeMismatch[\s\S]*CheckedTreeBuildInvalidOperation/,
    "call reduction errors must distinguish partial application, argument type, argument ascription, named value evidence, unsupported argument expression, overload, generic, expectation, and checked-tree build failures",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*ArgumentNestedCandidatePendingBinding[\s\S]*ArgumentNestedCandidateMissingSignature[\s\S]*ArgumentNestedCandidateHeadTokenOutOfBounds[\s\S]*ArgumentNestedCandidateOutOfMemory[\s\S]*ArgumentNestedCandidateInternalInvariant/,
    "nested named call candidate collection failures must remain typed at the call-reduction boundary",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*TrailingBlockBodySegmentFailed[\s\S]*TrailingBlockBodyEmpty[\s\S]*TrailingBlockBodyMultipleSegments[\s\S]*TrailingBlockBodyNestedBlockUnsupported[\s\S]*TrailingBlockBodyPrefixBuildFailed[\s\S]*TrailingBlockBodyCandidateCollectFailed[\s\S]*TrailingBlockBodyUnitTypeMissing[\s\S]*UnexpectedTrailingBlockArgument[\s\S]*UnsupportedArgumentExpression/,
    "trailing block argument body failures must stay typed instead of collapsing into partial application or generic unsupported argument errors",
);
assert.match(
    source,
    /pub enum SelfhostCallReduceErrorKind:[\s\S]*PipeMissingLeftOperand[\s\S]*PipeMissingRightTarget[\s\S]*PipeUnsupportedMultiple[\s\S]*PipeRightTargetUnsupported[\s\S]*PipeTargetPendingBinding[\s\S]*PipeTargetMissingSignature[\s\S]*PipeTargetHeadTokenOutOfBounds[\s\S]*PipeTargetOutOfMemory[\s\S]*PipeTargetInternalInvariant[\s\S]*PipeTargetUnresolved[\s\S]*PipeTargetAmbiguous[\s\S]*PipeTargetNoApplicableCandidate[\s\S]*PipeTargetAscriptionProjectionFailed[\s\S]*PipeTargetAscriptionTypeMismatch[\s\S]*PipeTargetRequiresInput[\s\S]*PipeLeftSegmentNotSingleValue/,
    "pipe reduction failures must remain typed instead of collapsing into generic unsupported-expression or overload errors",
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
    "single trailing block body input API must classify empty and multi-segment bodies before building a single reducer input",
);
assert.match(
    source,
    /SelfhostBodySegmentKind::ExpressionLine:[\s\S]*selfhost_block_body_result_from_expression_segment tokens source scope signatures segment[\s\S]*SelfhostBodySegmentKind::BlockIntro:[\s\S]*SelfhostBlockBodyResultErrorKind::NestedBlockUnsupported/,
    "legacy single block-body input API must keep rejecting nested BlockIntro instead of flattening it",
);
assert.match(
    source,
    /selfhost_expr_prefix_list_from_syntax_range tokens segment\.head[\s\S]*selfhost_callable_candidates_collect_for_prefix source tokens &prefix scope signatures/,
    "trailing block body checking must build candidates from the nested expression head through scope and signatures",
);
assert.match(
    source,
    /# check\/expr\/argument_payload[\s\S]*pub struct SelfhostCheckedValueIdentity:[\s\S]*name %str[\s\S]*def_id %SelfhostDefId[\s\S]*kind %SelfhostDefKind[\s\S]*pub enum SelfhostCheckedArgumentKind:[\s\S]*UnitValue[\s\S]*BoolLiteral %bool[\s\S]*I32Literal %i32[\s\S]*F32Literal %f32[\s\S]*CharLiteral %char[\s\S]*StrLiteral %str[\s\S]*NamedValue %SelfhostCheckedValueIdentity[\s\S]*TypedExpression[\s\S]*FunctionValue %SelfhostCallableCandidate[\s\S]*CheckedExpr %SelfhostCheckedExprId[\s\S]*NestedDirectCall %SelfhostCallableCandidate[\s\S]*BlockResult[\s\S]*pub struct SelfhostCheckedArgument:[\s\S]*kind %SelfhostCheckedArgumentKind[\s\S]*start_index %i32[\s\S]*next_index %i32[\s\S]*value_type %SelfhostTypeId[\s\S]*span %SelfhostSourceSpan/,
    "checked argument payload must preserve literal value, function value, checked tree expression ids, nested call, block-result, range, type, and span evidence",
);
assert.match(
    source,
    /# check\/expr\/checked_tree_id[\s\S]*pub struct SelfhostCheckedExprId:[\s\S]*index %i32[\s\S]*pub enum SelfhostCheckedArgumentRange:[\s\S]*Empty[\s\S]*Range %SelfhostCheckedArgumentRangeItems[\s\S]*pub enum SelfhostCheckedTreeRangeBuildError:[\s\S]*NegativeFirst[\s\S]*NegativeCount[\s\S]*NonCanonicalEmpty[\s\S]*EndOverflow[\s\S]*OutOfBounds/,
    "checked tree ids and argument ranges must use typed ids, typed ranges, and typed build errors instead of sentinel values",
);
assert.match(
    source,
    /# check\/expr\/checked_tree_id[\s\S]*pub struct SelfhostCheckedExprRangeItems:[\s\S]*first_expr %i32[\s\S]*expr_count %i32[\s\S]*pub enum SelfhostCheckedExprRange:[\s\S]*Empty[\s\S]*Range %SelfhostCheckedExprRangeItems[\s\S]*pub fn selfhost_checked_expr_range_new_bounded_result[\s\S]*pub fn selfhost_checked_expr_range_count/,
    "checked tree ids must include typed expression-id ranges for block sequence roots",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_checked_(?:expr_id|argument_range)_new_unchecked\b/,
    "checked tree unchecked id/range constructors must not be public API",
);
assert.match(
    source,
    /# check\/expr\/checked_tree[\s\S]*pub struct SelfhostCheckedDirectCall:[\s\S]*candidate %SelfhostCallableCandidate[\s\S]*arguments %SelfhostCheckedArgumentRange[\s\S]*pub enum SelfhostCheckedExprNodeKind:[\s\S]*DirectCall %SelfhostCheckedDirectCall[\s\S]*BlockResult %SelfhostCheckedExprId[\s\S]*BlockSequence %SelfhostCheckedExprRange[\s\S]*pub struct SelfhostCheckedExprTree:[\s\S]*nodes %Vec SelfhostCheckedExprNode[\s\S]*arguments %Vec SelfhostCheckedArgument[\s\S]*exprs %Vec SelfhostCheckedExprId/,
    "checked tree must own complex expression nodes, argument payload tables, and block expression-id tables separately from Copy checked argument summaries",
);
assert.match(
    source,
    /fn selfhost_checked_expr_node_refs_existing_before_append[\s\S]*SelfhostCheckedExprNodeKind::DirectCall call_payload:[\s\S]*selfhost_checked_argument_range_new_bounded_result[\s\S]*selfhost_checked_argument_range_refs_existing_before_append[\s\S]*SelfhostCheckedExprNodeKind::BlockResult body_expr:[\s\S]*selfhost_checked_expr_id_is_existing_before_append[\s\S]*SelfhostCheckedExprNodeKind::BlockSequence body_exprs:[\s\S]*selfhost_checked_expr_range_new_bounded_result[\s\S]*selfhost_checked_expr_range_refs_existing_before_append/,
    "checked tree node insertion must validate argument ranges and require CheckedExpr/BlockResult/BlockSequence references to point to existing nodes before append",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_add_argument_range[\s\S]*not selfhost_checked_argument_vec_refs_existing_before_append[\s\S]*StdErrorKind::InvalidOperation[\s\S]*selfhost_checked_argument_range_new_bounded_result/,
    "checked tree argument-range insertion must reject future/self CheckedExpr ids and build bounded typed ranges",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_add_direct_call[\s\S]*selfhost_checked_expr_tree_add_argument_range tree arguments_to_add[\s\S]*selfhost_checked_expr_node_direct_call result_type span candidate argument_range[\s\S]*selfhost_checked_expr_tree_add_node next_tree node/,
    "checked tree direct-call insertion must validate and copy checked arguments into the tree-owned argument table before storing a selected candidate node",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_add_expr_range[\s\S]*not selfhost_checked_expr_vec_refs_existing_before_append[\s\S]*StdErrorKind::InvalidOperation[\s\S]*selfhost_checked_expr_range_new_bounded_result/,
    "checked tree expression-range insertion must reject future/self expression ids and build bounded typed ranges",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_add_block_sequence[\s\S]*eq v::len &body_exprs_to_add 0[\s\S]*StdErrorKind::InvalidOperation[\s\S]*selfhost_checked_expr_tree_add_expr_range tree body_exprs_to_add[\s\S]*selfhost_checked_expr_node_block_sequence result_type span expr_range[\s\S]*selfhost_checked_expr_tree_add_node next_tree node/,
    "checked tree block-sequence insertion must store a nonempty range of checked expression root ids before appending the block node",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_get_argument[\s\S]*SelfhostCheckedArgumentRange::Empty:[\s\S]*none[\s\S]*SelfhostCheckedArgumentRange::Range range:[\s\S]*or lt idx 0 ge idx range\.argument_count[\s\S]*v::get arguments add range\.first_argument idx/,
    "checked tree argument lookup must bounds-check typed ranges and fail closed with Option::None",
);
assert.match(
    source,
    /pub fn selfhost_checked_expr_tree_get_expr_id[\s\S]*SelfhostCheckedExprRange::Empty:[\s\S]*none[\s\S]*SelfhostCheckedExprRange::Range range:[\s\S]*or lt idx 0 ge idx range\.expr_count[\s\S]*v::get exprs add range\.first_expr idx/,
    "checked tree expression-id lookup must bounds-check typed ranges and fail closed with Option::None",
);
assert.match(
    argumentPayload,
    /`TypedExpression` は、ascribed expression など、現 checkpoint で HIR child payload をまだ持たない通常の typed expression を表します。bool \/ i32 \/ f32 \/ char \/ string literal は専用 payload に分解済みなので、この variant へ戻してはいけません。/,
    "TypedExpression documentation must not describe implemented char/simple literal payloads as unsupported fallbacks",
);
assertArgumentPayloadDoc("SelfhostCheckedArgumentKind", "pub enum", [
    "[目的/もくてき]",
    "[分類/ぶんるい]",
    "[現状/げんじょう]",
    "`TypedExpression` は、型照合は済んでいるが",
    "`NestedDirectCall` と `BlockResult` は candidate や範囲の summary evidence だけを持つため",
    "detailed tree payload は `SelfhostCheckedExprTree`",
]);
assertArgumentPayloadDoc("SelfhostCheckedArgument", "pub struct", [
    "[目的/もくてき]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "literal 値や DefId-linked identity を捨てて、後段で source から復元してはいけません",
]);
assertArgumentPayloadDoc("selfhost_checked_argument_typed_expression", "pub fn", [
    "[目的/もくてき]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "f32 / char literal は専用 payload 実装済みなので",
]);
assertArgumentPayloadDoc("selfhost_checked_argument_checked_expr", "pub fn", [
    "[目的/もくてき]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "source token、scope lookup、callable candidate collection を再実行しません",
    "同じ `SelfhostCheckedExprTree` 内の node id",
]);
assertArgumentPayloadDoc("selfhost_checked_argument_nested_direct_call", "pub fn", [
    "[目的/もくてき]",
    "[現状/げんじょう]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "lowering は名前文字列から候補を再探索してはいけません",
    "full payload は `SelfhostCheckedExprTree`",
]);
assertArgumentPayloadDoc("selfhost_checked_argument_block_result", "pub fn", [
    "[目的/もくてき]",
    "[現状/げんじょう]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "`selfhost_hir_lower_checked_tree_expr`",
]);
assert.doesNotMatch(
    argumentPayload,
    /char literal、escape 付き string|CharLiteral[^。\n]*TypedExpression|char literal[^。\n]*TypedExpression/,
    "argument payload comments must not reintroduce the old char-literal-as-TypedExpression explanation",
);
assert.match(
    source,
    /selfhost_expr_argument_checked_simple_item:[\s\S]*\[目的\/もくてき\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*source-backed 経路では char literal は `CharLiteral` payload に分解済み[\s\S]*fn selfhost_expr_argument_checked_simple_item/,
    "source-less simple-item argument helper docs must preserve the source-backed CharLiteral payload boundary",
);
assert.doesNotMatch(
    source,
    /char literal は HIR payload が未実装|char literal[^。\n]*`TypedExpression` に留めます|CharLiteral[^。\n]*TypedExpression/,
    "check/expr comments must not reintroduce the old source-backed char-literal-as-TypedExpression explanation",
);
assert.match(
    source,
    /selfhost_checked_argument_unit_value[\s\S]*SelfhostCheckedArgumentKind::UnitValue[\s\S]*fn selfhost_expr_argument_checked_simple_item[\s\S]*SelfhostExprPrefixItemKind::UnitValue:[\s\S]*selfhost_checked_argument_unit_value/,
    "unit literal arguments must be preserved as a dedicated checked payload instead of a source-reread TypedExpression",
);
assertContainsInOrder(source, [
    "# check/expr/literal_payload",
    "pub enum SelfhostLiteralArgumentErrorKind:",
    "TokenOutOfBounds",
    "BoolInvalid",
    "I32Invalid",
    "I32RadixUnsupported",
    "StringMalformed",
    "StringEscapeUnsupported",
    "StringEscapeMalformed",
    "StringSliceFailed",
    "StringBuildFailed",
    "F32Invalid",
    "CharMalformed",
    "CharEscapeUnsupported",
    "CharInvalidScalar",
    "CharMultipleScalars",
    "pub fn selfhost_literal_argument_checked_with_source %fn &Vec SelfhostToken fn str fn SelfhostExprPrefixItem fn i32 fn i32 fn SelfhostTypeId Result SelfhostCheckedArgument SelfhostLiteralArgumentError",
], "literal argument payload creation must live in its own source-backed check/expr module with typed failures");
assert.match(
    source,
    /selfhost_literal_argument_bool_from_lexeme[\s\S]*string::str_eq lexeme "true"[\s\S]*string::str_eq lexeme "false"[\s\S]*selfhost_checked_argument_bool_literal/,
    "bool literal arguments must store semantic bool payloads in checked arguments",
);
assert.match(
    source,
    /SelfhostLiteralI32RadixPlan[\s\S]*selfhost_literal_argument_i32_radix_plan_from_lexeme[\s\S]*or eq prefix_second 'x' eq prefix_second 'X'[\s\S]*selfhost_literal_i32_radix_plan_new 16[\s\S]*selfhost_literal_argument_i32_unsupported_radix_marker[\s\S]*I32RadixUnsupported[\s\S]*selfhost_literal_argument_i32_body_from_plan[\s\S]*string::to_i32_radix body plan\.radix[\s\S]*selfhost_checked_argument_i32_literal/,
    "i32 literal arguments must normalize decimal and hex spelling to semantic i32 payloads without rereading source during HIR lowering",
);
assert.match(
    source,
    /selfhost_literal_argument_f32_from_lexeme[\s\S]*string::to_f32 lexeme[\s\S]*SelfhostLiteralArgumentErrorKind::F32Invalid[\s\S]*SelfhostExprPrefixItemKind::FloatLiteral:[\s\S]*selfhost_checked_argument_f32_literal/,
    "f32 literal arguments must normalize float spelling to semantic f32 payloads without rereading source during HIR lowering",
);
assertLiteralPayloadDoc("selfhost_literal_argument_negative_checked_with_source", [
    "[目的/もくてき]",
    "[契約/けいやく]",
    "[戻/もど]り[値/ち]",
    "[計算量/けいさんりょう]",
    "token 間 whitespace を意味値に混ぜません",
    "semantic lexeme authority",
]);
assert.match(
    literalPayload,
    /fn selfhost_literal_argument_negative_numeric_lexeme_from_source[\s\S]*string::str_slice_result source operand_item\.span\.start operand_item\.span\.end[\s\S]*string::concat_result "-" operand_lexeme/,
    "negative numeric literal payload creation must concatenate '-' with the operand token spelling instead of slicing the joined span",
);
assert.match(
    literalPayload,
    /pub fn selfhost_literal_argument_negative_checked_with_source[\s\S]*SelfhostExprPrefixItemKind::IntLiteral:[\s\S]*selfhost_literal_argument_negative_numeric_lexeme_from_source source operand_item signed_item\.span SelfhostLiteralArgumentErrorKind::I32Invalid[\s\S]*selfhost_checked_argument_i32_literal start_index next_index value_type signed_item\.span value[\s\S]*SelfhostExprPrefixItemKind::FloatLiteral:[\s\S]*selfhost_literal_argument_negative_numeric_lexeme_from_source source operand_item signed_item\.span SelfhostLiteralArgumentErrorKind::F32Invalid[\s\S]*selfhost_checked_argument_f32_literal start_index next_index value_type signed_item\.span value/,
    "negative numeric literal payload creation must preserve typed i32/f32 payload paths",
);
assert.doesNotMatch(
    source,
    /selfhost_literal_argument_i32_from_lexeme[\s\S]*(?:string::str_starts_with_at lexeme [0-9]+ "i32"|byte_at lexeme add [^\n]+(?:'i'|'u'|'f'))/,
    "i32 literal payload parsing must not scan past the numeric token to recover suffix syntax",
);
assert.match(
    source,
    /selfhost_literal_argument_string_value_from_lexeme[\s\S]*selfhost_literal_argument_lexeme_contains_byte_loop lexeme '\\\\' 1 sub n 1[\s\S]*selfhost_literal_argument_string_escaped_value_from_lexeme lexeme n span[\s\S]*string::str_slice_result lexeme 1 sub n 1[\s\S]*SelfhostLiteralArgumentErrorKind::StringSliceFailed[\s\S]*selfhost_checked_argument_str_literal/,
    "string literal arguments must store decoded semantic string values and keep escape decode before HIR lowering",
);
assert.match(
    source,
    /selfhost_literal_argument_string_simple_escape[\s\S]*'n':[\s\S]*'r':[\s\S]*'t':[\s\S]*'\\\\':[\s\S]*'"':[\s\S]*'0':[\s\S]*StringEscapeUnsupported[\s\S]*selfhost_literal_argument_string_hex_escape[\s\S]*StringEscapeMalformed[\s\S]*char_from_i32_result[\s\S]*selfhost_literal_argument_string_decode_loop/,
    "string escape decode must keep the Rust string escape set separate from char-only escapes",
);
assert.match(
    source,
    /selfhost_literal_argument_char_value_from_lexeme[\s\S]*selfhost_literal_argument_char_escape_from_lexeme[\s\S]*string::str_next_char_result lexeme 1[\s\S]*CharMultipleScalars[\s\S]*SelfhostExprPrefixItemKind::CharLiteral:[\s\S]*selfhost_checked_argument_char_literal/,
    "char literal arguments must store semantic char payloads and reject malformed or multi-scalar literals as typed errors",
);
for (const name of [
    "selfhost_literal_argument_error_new",
    "selfhost_literal_char_decode_new",
    "selfhost_literal_string_decode_new",
    "selfhost_literal_argument_bool_from_lexeme",
    "selfhost_literal_i32_radix_plan_new",
    "selfhost_literal_argument_i32_unsupported_radix_marker",
    "selfhost_literal_argument_i32_radix_plan_from_lexeme",
    "selfhost_literal_argument_i32_body_from_plan",
    "selfhost_literal_argument_i32_from_lexeme",
    "selfhost_literal_argument_f32_from_lexeme",
    "selfhost_literal_argument_lexeme_contains_byte_loop",
    "selfhost_literal_argument_string_quotes_valid",
    "selfhost_literal_argument_string_builder_error",
    "selfhost_literal_argument_string_append_decode",
    "selfhost_literal_argument_string_simple_escape",
    "selfhost_literal_argument_string_hex_escape",
    "selfhost_literal_argument_string_escape_from_lexeme",
    "selfhost_literal_argument_string_decode_loop",
    "selfhost_literal_argument_string_escaped_value_from_lexeme",
    "selfhost_literal_argument_string_value_from_lexeme",
    "selfhost_literal_argument_hex_digit_value",
    "selfhost_literal_argument_char_quotes_valid",
    "selfhost_literal_argument_char_decode_from_code",
    "selfhost_literal_argument_char_simple_escape",
    "selfhost_literal_argument_char_hex_escape",
    "selfhost_literal_argument_char_unicode_digits_loop",
    "selfhost_literal_argument_char_unicode_escape",
    "selfhost_literal_argument_char_escape_from_lexeme",
    "selfhost_literal_argument_char_value_from_lexeme",
    "selfhost_literal_argument_checked_from_lexeme",
    "selfhost_literal_argument_checked_with_source",
]) {
    assertLiteralPayloadDoc(name, ["[目的/もくてき]", "[計算量/けいさんりょう]"]);
}
for (const name of [
    "selfhost_literal_argument_bool_from_lexeme",
    "selfhost_literal_argument_i32_radix_plan_from_lexeme",
    "selfhost_literal_argument_i32_body_from_plan",
    "selfhost_literal_argument_i32_from_lexeme",
    "selfhost_literal_argument_f32_from_lexeme",
    "selfhost_literal_argument_string_quotes_valid",
    "selfhost_literal_argument_string_append_decode",
    "selfhost_literal_argument_string_simple_escape",
    "selfhost_literal_argument_string_hex_escape",
    "selfhost_literal_argument_string_escape_from_lexeme",
    "selfhost_literal_argument_string_decode_loop",
    "selfhost_literal_argument_string_escaped_value_from_lexeme",
    "selfhost_literal_argument_string_value_from_lexeme",
    "selfhost_literal_argument_hex_digit_value",
    "selfhost_literal_argument_char_quotes_valid",
    "selfhost_literal_argument_char_decode_from_code",
    "selfhost_literal_argument_char_simple_escape",
    "selfhost_literal_argument_char_hex_escape",
    "selfhost_literal_argument_char_unicode_digits_loop",
    "selfhost_literal_argument_char_unicode_escape",
    "selfhost_literal_argument_char_escape_from_lexeme",
    "selfhost_literal_argument_char_value_from_lexeme",
    "selfhost_literal_argument_checked_from_lexeme",
    "selfhost_literal_argument_checked_with_source",
]) {
    assertLiteralPayloadDoc(name, ["[戻/もど]り[値/ち]"]);
}
for (const name of [
    "selfhost_literal_argument_error_new",
    "selfhost_literal_i32_radix_plan_new",
    "selfhost_literal_argument_i32_unsupported_radix_marker",
    "selfhost_literal_argument_i32_radix_plan_from_lexeme",
    "selfhost_literal_argument_i32_body_from_plan",
    "selfhost_literal_argument_f32_from_lexeme",
    "selfhost_literal_argument_lexeme_contains_byte_loop",
    "selfhost_literal_char_decode_new",
    "selfhost_literal_string_decode_new",
    "selfhost_literal_argument_string_builder_error",
    "selfhost_literal_argument_string_append_decode",
    "selfhost_literal_argument_string_hex_escape",
    "selfhost_literal_argument_string_escape_from_lexeme",
    "selfhost_literal_argument_string_decode_loop",
    "selfhost_literal_argument_char_hex_escape",
    "selfhost_literal_argument_char_unicode_digits_loop",
    "selfhost_literal_argument_char_unicode_escape",
    "selfhost_literal_argument_char_escape_from_lexeme",
    "selfhost_literal_argument_checked_from_lexeme",
]) {
    assertLiteralPayloadDoc(name, ["[契約/けいやく]"]);
}
assert.match(
    source,
    /selfhost_expr_argument_match_literal_with_source[\s\S]*selfhost_literal_argument_checked_with_source tokens source item item_index next_index expected_type[\s\S]*selfhost_type_arena_free arena[\s\S]*selfhost_expr_argument_match_error_from_literal literal_error/,
    "source-backed argument checking must convert literal payload failures into typed argument errors after freeing the arena owner",
);
assert.match(
    source,
    /fn selfhost_expr_argument_negative_numeric_kind[\s\S]*SelfhostExprPrefixItemKind::IntLiteral:[\s\S]*SelfhostExprPrefixItemKind::FloatLiteral:[\s\S]*Option::None/,
    "negative literal argument matching must accept only int and float literal followers",
);
assert.match(
    source,
    /fn selfhost_expr_argument_match_negative_literal_with_source[\s\S]*NegativeLiteralMissingOperand[\s\S]*selfhost_expr_argument_negative_numeric_kind operand_item[\s\S]*source_span_join_result minus_item\.span operand_item\.span[\s\S]*selfhost_expr_prefix_item_new literal_kind minus_item\.token_index joined_span[\s\S]*selfhost_literal_argument_negative_checked_with_source tokens source minus_item operand_item signed_item item_index next_index expected_type/,
    "negative literal argument matching must use joined span for diagnostics while payload lexeme construction stays in literal_payload",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::MinusMarker:[\s\S]*selfhost_expr_argument_match_negative_literal_with_source arena tokens source prefix item_index item_count expected_type item/,
    "source-backed argument checking must route MinusMarker through the negative numeric literal matcher",
);
assert.match(
    source,
    /fn selfhost_expr_argument_match_ascribed_with_projection[\s\S]*SelfhostExprPrefixItemKind::MinusMarker:[\s\S]*selfhost_expr_argument_match_negative_literal_with_source arena tokens source prefix expression_item_index item_count expectation\.expected_type expression_item/,
    "ascribed source-backed argument checking must route MinusMarker through the same negative numeric literal matcher",
);
assert.match(
    source,
    /selfhost_checked_argument_function_value[\s\S]*SelfhostCheckedArgumentKind::FunctionValue candidate[\s\S]*selfhost_checked_argument_is_function_value[\s\S]*SelfhostCheckedArgumentKind::FunctionValue _candidate:[\s\S]*true/,
    "function value arguments must be represented as typed checked-argument payloads",
);
assert.match(
    source,
    /selfhost_checked_argument_named_value[\s\S]*SelfhostCheckedArgumentKind::NamedValue identity[\s\S]*fn selfhost_expr_argument_named_value_evidence_from_binding[\s\S]*selfhost_checked_value_identity_new binding\.name def_id binding\.kind[\s\S]*selfhost_checked_argument_named_value item_index next_index named\.value_type item\.span named\.identity/,
    "named value arguments must be represented as DefId-linked checked-argument payloads instead of TypedExpression",
);
assert.match(
    source,
    /# check\/expr\/argument[\s\S]*pub struct SelfhostExprArgumentMatch:[\s\S]*next_index %i32[\s\S]*pub fn selfhost_expr_argument_item_literal_type_kind[\s\S]*pub fn selfhost_expr_argument_match_at/,
    "argument type evidence and consume-width scanning must live in its own check/expr split module",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::UnitValue:[\s\S]*SelfhostTypeKind::Unit[\s\S]*SelfhostExprPrefixItemKind::IntLiteral:[\s\S]*SelfhostTypeKind::I32[\s\S]*SelfhostExprPrefixItemKind::FloatLiteral:[\s\S]*SelfhostTypeKind::F32[\s\S]*SelfhostExprPrefixItemKind::BoolLiteral:[\s\S]*SelfhostTypeKind::Bool/,
    "argument evidence must map simple literal items to typed primitive evidence",
);
assert.match(
    source,
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*TypeMismatch[\s\S]*UnsupportedAscribedArgument[\s\S]*AscriptionProjectionFailed[\s\S]*AscriptionExpectedTypeConflict[\s\S]*NamedValueUnresolved[\s\S]*NamedValuePendingBinding[\s\S]*NamedValueUnsupportedBinding[\s\S]*NamedValueEvidenceMissing[\s\S]*FunctionValueExpectedFunctionType[\s\S]*FunctionValueMissingName[\s\S]*FunctionValueUnresolved[\s\S]*FunctionValueAmbiguous[\s\S]*FunctionValueGenericUnsupported[\s\S]*FunctionValueTypeMismatch[\s\S]*LiteralTokenOutOfBounds[\s\S]*LiteralBoolInvalid[\s\S]*LiteralI32Invalid[\s\S]*LiteralI32RadixUnsupported[\s\S]*LiteralF32Invalid[\s\S]*LiteralStringMalformed[\s\S]*LiteralStringEscapeUnsupported[\s\S]*LiteralStringEscapeMalformed[\s\S]*LiteralStringSliceFailed[\s\S]*LiteralStringBuildFailed[\s\S]*LiteralCharMalformed[\s\S]*LiteralCharEscapeUnsupported[\s\S]*LiteralCharInvalidScalar[\s\S]*LiteralCharMultipleScalars[\s\S]*NegativeLiteralMissingOperand[\s\S]*NegativeLiteralOperandNotNumeric[\s\S]*NegativeLiteralSpanJoinFailed[\s\S]*NegativeLiteralTypeMismatch[\s\S]*UnsupportedArgumentExpression/,
    "argument expression scan failures must stay typed instead of collapsing into a boolean",
);
assert.match(
    source,
    /pub enum SelfhostExprArgumentMatchErrorKind:[\s\S]*AscriptionExpectedTypeConflict/,
    "argument-scope ascription must preserve expected-type conflicts as a typed error",
);
assert.match(
    source,
    /SelfhostExprArgumentMatchErrorKind::UnsupportedAscribedArgument:[\s\S]*SelfhostCallReduceErrorKind::UnsupportedArgumentExpression[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionProjectionFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionProjectionFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::AscriptionExpectedTypeConflict:[\s\S]*SelfhostCallReduceErrorKind::ArgumentAscriptionExpectedTypeConflict[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValueUnresolved:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueUnresolved[\s\S]*SelfhostExprArgumentMatchErrorKind::NamedValueEvidenceMissing:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNamedValueEvidenceMissing[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueExpectedFunctionType:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueExpectedFunctionType[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueMissingName:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueMissingName[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueUnresolved:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueUnresolved[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueAmbiguous:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueAmbiguous[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValuePendingBinding:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValuePendingBinding[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueMissingSignature:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueMissingSignature[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueHeadTokenOutOfBounds:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueHeadTokenOutOfBounds[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueOutOfMemory:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueOutOfMemory[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueGenericUnsupported:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueGenericUnsupported[\s\S]*SelfhostExprArgumentMatchErrorKind::FunctionValueTypeMismatch:[\s\S]*SelfhostCallReduceErrorKind::ArgumentFunctionValueTypeMismatch/,
    "call reducer must map argument-scope ascription, named value evidence, and explicit function value failures separately from generic unsupported argument expressions",
);
assert.match(
    source,
    /SelfhostExprArgumentMatchErrorKind::LiteralTokenOutOfBounds:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralTokenOutOfBounds[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralBoolInvalid:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralBoolInvalid[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralI32Invalid:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralI32Invalid[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralI32RadixUnsupported:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralI32RadixUnsupported[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralF32Invalid:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralF32Invalid[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralStringMalformed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralStringMalformed[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralStringEscapeUnsupported:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralStringEscapeUnsupported[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralStringEscapeMalformed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralStringEscapeMalformed[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralStringSliceFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralStringSliceFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralStringBuildFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralStringBuildFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralCharMalformed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralCharMalformed[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralCharEscapeUnsupported:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralCharEscapeUnsupported[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralCharInvalidScalar:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralCharInvalidScalar[\s\S]*SelfhostExprArgumentMatchErrorKind::LiteralCharMultipleScalars:[\s\S]*SelfhostCallReduceErrorKind::ArgumentLiteralCharMultipleScalars/,
    "call reducer must preserve literal payload decode failures as typed call-reduction errors",
);
assert.match(
    source,
    /SelfhostExprArgumentMatchErrorKind::NegativeLiteralMissingOperand:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNegativeLiteralMissingOperand[\s\S]*SelfhostExprArgumentMatchErrorKind::NegativeLiteralOperandNotNumeric:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNegativeLiteralOperandNotNumeric[\s\S]*SelfhostExprArgumentMatchErrorKind::NegativeLiteralSpanJoinFailed:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNegativeLiteralSpanJoinFailed[\s\S]*SelfhostExprArgumentMatchErrorKind::NegativeLiteralTypeMismatch:[\s\S]*SelfhostCallReduceErrorKind::ArgumentNegativeLiteralTypeMismatch/,
    "call reducer must preserve negative numeric literal boundary failures as typed call-reduction errors",
);
assert.match(
    source,
    /pub struct SelfhostExprArgumentOwnedMatch:[\s\S]*arena %SelfhostTypeArena[\s\S]*match_value %SelfhostExprArgumentMatch[\s\S]*checked_argument %SelfhostCheckedArgument[\s\S]*pub fn selfhost_expr_argument_match_at_with_source %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable/,
    "source-backed argument checking must return an updated arena owner with consume-width and checked-argument evidence",
);
assert.match(
    source,
    /SelfhostExprPrefixItemKind::AtMarker:[\s\S]*selfhost_expr_argument_match_function_value_with_source tokens source arena prefix scope signatures item_index item_count expected_type item/,
    "source-backed argument checking must treat explicit @ident through the function value boundary",
);
assert.match(
    source,
    /fn selfhost_expr_argument_match_function_value_with_source[\s\S]*not selfhost_expr_argument_expected_type_is_function &arena expected_type[\s\S]*selfhost_callable_candidates_collect_for_head_item source tokens name_item scope signatures/,
    "function value arguments must require an expected function type before collecting callable candidate evidence",
);
assert.match(
    source,
    /fn selfhost_expr_argument_match_function_value_candidate[\s\S]*selfhost_type_arena_types_equal &arena candidate\.callable_type expected_type[\s\S]*selfhost_checked_argument_function_value start_index next_index candidate\.callable_type span candidate/,
    "function value arguments must compare the selected callable signature and preserve the selected candidate payload",
);
assert.match(
    source,
    /selfhost_expr_argument_function_value_candidate_is_monomorphic[\s\S]*SelfhostGenericInferenceState::NoneRequired:[\s\S]*true[\s\S]*FunctionValueGenericUnsupported/,
    "function value arguments must fail closed for unresolved generic callable candidates",
);
assert.match(
    source,
    /selfhost_name_scope_find scope name[\s\S]*selfhost_expr_argument_named_value_evidence_from_binding value_types binding/,
    "source-backed named value arguments must resolve through scope, DefId-linked value evidence, and arena structural equality",
);
assert.match(
    source,
    /selfhost_value_type_evidence_table_find value_types def_id[\s\S]*selfhost_checked_value_identity_new binding\.name def_id binding\.kind[\s\S]*Result::Ok selfhost_expr_argument_named_value_evidence_new identity evidence\.value_type/,
    "named value type evidence must be recovered from the DefId-linked table",
);
assert.match(
    source,
    /selfhost_type_arena_types_equal &arena named\.value_type expected_type/,
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
    /fn selfhost_call_reduce_argument_match_direct_with_source[\s\S]*selfhost_expr_argument_match_at_with_source tokens source arena prefix scope value_types signatures item_index item_count expected_type[\s\S]*selfhost_call_reduce_error_from_argument_match_owned argument_error head/,
    "source-backed direct argument fallback must still use token/source-backed argument matching",
);
assert.match(
    source,
    /selfhost_type_arena_function_arg &arena candidate\.callable_type param_idx[\s\S]*selfhost_call_reduce_argument_match_at_with_source_or_nested tokens source arena checked_tree prefix scope value_types signatures SelfhostNestedArgumentBoundary::OuterContinuation candidate param_count param_idx item_index item_count expected_arg_type head[\s\S]*let checked_argument %SelfhostCheckedArgument field::get match_state "checked_argument"[\s\S]*let next_tree %SelfhostCheckedExprTree field::get match_state "checked_tree"[\s\S]*selfhost_call_reduce_push_checked_argument next_arena checked_arguments next_tree checked_argument head\.span[\s\S]*argument_match\.next_index/,
    "source-backed call reduction must keep each consumed argument's checked payload while passing the outer candidate cursor to nested overload narrowing",
);
assert.match(
    source,
    /selfhost_call_reduce_error_from_candidate_collect[\s\S]*SelfhostCallableCandidateCollectErrorKind::EmptyPrefix:[\s\S]*ArgumentNestedCandidateInternalInvariant[\s\S]*SelfhostCallableCandidateCollectErrorKind::PrefixBuildFailed:[\s\S]*ArgumentNestedCandidateInternalInvariant[\s\S]*SelfhostCallableCandidateCollectErrorKind::UnsupportedHead:[\s\S]*ArgumentNestedCandidateInternalInvariant[\s\S]*SelfhostCallableCandidateCollectErrorKind::PendingBinding:[\s\S]*ArgumentNestedCandidatePendingBinding[\s\S]*SelfhostCallableCandidateCollectErrorKind::MissingSignature:[\s\S]*ArgumentNestedCandidateMissingSignature[\s\S]*SelfhostCallableCandidateCollectErrorKind::HeadTokenOutOfBounds:[\s\S]*ArgumentNestedCandidateHeadTokenOutOfBounds[\s\S]*SelfhostCallableCandidateCollectErrorKind::OutOfMemory:[\s\S]*ArgumentNestedCandidateOutOfMemory/,
    "nested candidate collection failures must be mapped exhaustively instead of collapsing into UnsupportedArgumentExpression",
);
assert.match(
    source,
    /selfhost_call_reduce_pipe_error_from_candidate_collect[\s\S]*SelfhostCallableCandidateCollectErrorKind::EmptyPrefix:[\s\S]*PipeTargetInternalInvariant[\s\S]*SelfhostCallableCandidateCollectErrorKind::PrefixBuildFailed:[\s\S]*PipeTargetInternalInvariant[\s\S]*SelfhostCallableCandidateCollectErrorKind::UnsupportedHead:[\s\S]*PipeRightTargetUnsupported[\s\S]*SelfhostCallableCandidateCollectErrorKind::PendingBinding:[\s\S]*PipeTargetPendingBinding[\s\S]*SelfhostCallableCandidateCollectErrorKind::MissingSignature:[\s\S]*PipeTargetMissingSignature[\s\S]*SelfhostCallableCandidateCollectErrorKind::HeadTokenOutOfBounds:[\s\S]*PipeTargetHeadTokenOutOfBounds[\s\S]*SelfhostCallableCandidateCollectErrorKind::OutOfMemory:[\s\S]*PipeTargetOutOfMemory/,
    "pipe candidate collection failures must be mapped exhaustively to pipe-specific typed errors",
);
assert.match(
    source,
    /selfhost_callable_candidates_collect_for_head_item source tokens item scope signatures[\s\S]*selfhost_call_reduce_nested_named_candidates_with_source tokens source arena checked_tree prefix scope value_types signatures nested_candidates boundary outer_candidate outer_param_count outer_param_idx item_index item_count expected_type item/,
    "nested named call arguments must collect inner candidates from the argument head and reduce them with the outer expected argument type",
);
assert.match(
    source,
    /eq candidate_count 0[\s\S]*selfhost_call_reduce_argument_match_direct_with_source tokens source arena checked_tree prefix scope value_types signatures item_index item_count expected_type head[\s\S]*gt candidate_count 1[\s\S]*match boundary:[\s\S]*SelfhostNestedArgumentBoundary::FinalRange:[\s\S]*selfhost_call_reduce_nested_candidate_select_by_complete_borrowed_match &arena prefix &candidates item_index item_count expected_type head candidate_count[\s\S]*eq argument_match\.next_index item_count[\s\S]*SelfhostNestedArgumentBoundary::OuterContinuation:[\s\S]*selfhost_call_reduce_nested_candidate_select_by_continuation_borrowed_match &arena prefix &candidates outer_candidate outer_param_count outer_param_idx item_index item_count expected_type head candidate_count[\s\S]*eq argument_match\.next_index probed_next_index[\s\S]*selfhost_call_reduce_nested_single_named_candidate_with_source tokens source arena checked_tree prefix scope value_types signatures candidate item_index item_count expected_type head/,
    "named argument handling must fall back to value evidence only when no visible function candidates exist and must separate final-range and continuation-aware borrowed narrowing for multiple candidates",
);
assert.match(
    source,
    /enum SelfhostNestedArgumentBoundary:[\s\S]*FinalRange[\s\S]*OuterContinuation[\s\S]*impl Clone for SelfhostNestedArgumentBoundary[\s\S]*impl Copy for SelfhostNestedArgumentBoundary/,
    "nested argument overload narrowing must expose final-range versus outer-continuation as an enum contract rather than a bool or implicit caller convention",
);
assertContainsInOrder(
    source,
    [
        "fn selfhost_call_reduce_nested_candidate_error_is_no_match",
        "SelfhostCallReduceErrorKind::ArgumentTypeMismatch:",
        "SelfhostCallReduceErrorKind::PartialApplicationRejected:",
        "SelfhostCallReduceErrorKind::OverloadNoCandidate:",
        "SelfhostCallReduceErrorKind::ExpectedTypeMismatch:",
        "fn selfhost_call_reduce_nested_candidate_complete_borrowed_matches",
        "selfhost_call_reduce_argument_type_check_loop arena prefix candidate param_count 0 add item_index 1 item_count head",
        "selfhost_call_reduce_expected_result arena candidate some expected",
        "fn selfhost_call_reduce_candidate_borrowed_next_index_loop",
        "Result::Ok item_index",
        "selfhost_expr_argument_match_at arena prefix item_index item_count expected_arg_type",
        "argument_match.next_index",
        "fn selfhost_call_reduce_nested_candidate_borrowed_next_index",
        "selfhost_call_reduce_candidate_borrowed_next_index_loop arena prefix candidate param_count 0 add item_index 1 item_count head",
        "fn selfhost_call_reduce_nested_candidate_continuation_borrowed_matches",
        "selfhost_call_reduce_argument_type_check_loop arena prefix outer_candidate outer_param_count add outer_param_idx 1 next_index item_count head",
        "fn selfhost_call_reduce_nested_candidate_continuation_match_count_loop",
        "fn selfhost_call_reduce_nested_candidate_continuation_first_match_loop",
        "Result SelfhostNestedCandidateBorrowedSelection SelfhostCallReduceError",
        "Result::Ok selfhost_nested_candidate_borrowed_selection_new candidate next_index",
        "fn selfhost_call_reduce_nested_candidate_select_by_continuation_borrowed_match",
        "Result SelfhostNestedCandidateBorrowedSelection SelfhostCallReduceError",
        "eq match_count 1",
        "SelfhostCallReduceErrorKind::OverloadAmbiguous",
        "fn selfhost_call_reduce_nested_candidate_match_count_loop",
        "fn selfhost_call_reduce_nested_candidate_first_match_loop",
        "fn selfhost_call_reduce_nested_candidate_select_by_complete_borrowed_match",
        "eq match_count 1",
        "SelfhostCallReduceErrorKind::OverloadAmbiguous",
    ],
    "source-backed nested overload narrowing must use borrowed consume-width, outer continuation, and expected result while retaining final-range probe helpers",
);
assert.match(
    source,
    /fn selfhost_call_reduce_nested_candidate_select_by_continuation_borrowed_match %fn &SelfhostTypeArena fn &SelfhostExprPrefixList fn &Vec SelfhostCallableCandidate fn SelfhostCallableCandidate fn i32 fn i32 fn i32 fn i32 fn SelfhostTypeId fn SelfhostExprPrefixItem fn i32 Result SelfhostNestedCandidateBorrowedSelection SelfhostCallReduceError \\arena\\prefix\\candidates\\outer_candidate\\outer_param_count\\outer_param_idx\\item_index\\item_count\\expected_type\\head\\candidate_count:/,
    "nested overload selector must stay a borrowed continuation probe, return probed next_index, and must not take tokens, source text, scope, value evidence, signatures, checked tree, or owner reducer inputs",
);
assert.match(
    source,
    /SelfhostNestedCandidateBorrowedSelection:[\s\S]*candidate %SelfhostCallableCandidate[\s\S]*next_index %i32[\s\S]*selfhost_call_reduce_nested_candidate_select_by_continuation_borrowed_match[\s\S]*let selected_candidate %SelfhostCallableCandidate field::get selection "candidate"[\s\S]*let probed_next_index %i32 field::get selection "next_index"[\s\S]*selfhost_call_reduce_nested_single_named_candidate_with_source[\s\S]*let argument_match %SelfhostExprArgumentMatch field::get match_state "match_value"[\s\S]*eq argument_match\.next_index probed_next_index[\s\S]*SelfhostCallReduceErrorKind::InternalInvariant/,
    "source-backed nested overload selection must verify that the finisher consumes the same next_index as the borrowed probe",
);
assert.match(
    source,
    /fn selfhost_call_reduce_argument_match_at_with_source_or_nested[\s\S]*boundary\\outer_candidate\\outer_param_count\\outer_param_idx\\item_index\\item_count\\expected_type\\head:[\s\S]*selfhost_callable_candidates_collect_for_head_item source tokens item scope signatures[\s\S]*selfhost_call_reduce_nested_named_candidates_with_source tokens source arena checked_tree prefix scope value_types signatures nested_candidates boundary outer_candidate outer_param_count outer_param_idx item_index item_count expected_type item/,
    "same-name nested narrowing is owned by the common source-backed nested argument path and receives an explicit boundary plus the outer continuation cursor",
);
assert.match(
    source,
    /fn selfhost_call_reduce_nested_single_named_candidate_with_source[\s\S]*selfhost_call_reduce_argument_consume_loop_with_source tokens source arena checked_tree nested_arguments[\s\S]*selfhost_checked_expr_tree_add_direct_call consumed_tree candidate result_type head\.span nested_arguments_result[\s\S]*selfhost_checked_argument_checked_expr item_index argument_match\.next_index result_type head\.span expr_id/,
    "nested call reduction must store the nested call in the checked tree and return a CheckedExpr payload for the enclosing argument loop",
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
    /fn selfhost_call_reduce_error_from_checked_tree_std_error[\s\S]*StdErrorKind::OutOfMemory:[\s\S]*SelfhostCallReduceErrorKind::OutOfMemory[\s\S]*_:[\s\S]*SelfhostCallReduceErrorKind::CheckedTreeBuildInvalidOperation/,
    "checked tree builder failures must keep a dedicated call-reduction error instead of collapsing into InternalInvariant",
);
assert.match(
    source,
    /pub struct SelfhostCallReduceOwnedResult:[\s\S]*arena %SelfhostTypeArena[\s\S]*result %SelfhostCallReduceResult[\s\S]*checked_arguments %Vec SelfhostCheckedArgument[\s\S]*checked_tree %SelfhostCheckedExprTree[\s\S]*root_expr %SelfhostCheckedExprId[\s\S]*pub fn selfhost_call_reduce_prefix_with_source %impure fn &Vec SelfhostToken impure fn str impure fn SelfhostTypeArena impure fn &SelfhostExprPrefixList impure fn &SelfhostNameScope impure fn &SelfhostValueTypeEvidenceTable impure fn &SelfhostCallableSignatureTable impure fn &Vec SelfhostCallableCandidate impure fn Option SelfhostTypeExpectation Result SelfhostCallReduceOwnedResult SelfhostCallReduceError/,
    "source-backed call reduction must expose an arena-owner boundary with checked arguments plus checked tree/root payloads separate from the borrowed reducer",
);
assert.match(
    source,
    /pub struct SelfhostExpressionLineCheckSuccess:[\s\S]*arena %SelfhostTypeArena[\s\S]*result %SelfhostCallReduceResult[\s\S]*checked_arguments %Vec SelfhostCheckedArgument[\s\S]*checked_tree %SelfhostCheckedExprTree[\s\S]*root_expr %SelfhostCheckedExprId[\s\S]*selfhost_expression_line_check_success_checked_tree[\s\S]*selfhost_expression_line_check_success_root_expr/,
    "expression-line success must preserve checked argument evidence, checked tree, and root expression id at the body-line boundary",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_success_has_function_value_argument[\s\S]*selfhost_expression_line_check_success_checked_arguments success[\s\S]*selfhost_checked_argument_is_function_value &argument[\s\S]*selfhost_check_expr_stage1_function_value_argument_ok_with_scope[\s\S]*selfhost_check_expr_stage1_success_has_function_value_argument &success/,
    "stage1 must verify that explicit @ident keeps a FunctionValue checked-argument payload through body_line success",
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
    /selfhost_check_expr_stage1_make_general_nested_overload_argument_tokens[\s\S]*"outer add 1 2 3"[\s\S]*selfhost_check_expr_stage1_general_nested_overload_argument_ok_with_scope[\s\S]*selfhost_check_expr_stage1_success_has_general_nested_overload_argument_order[\s\S]*selfhost_check_expr_stage1_general_nested_root_links_child_direct_call &success "add"[\s\S]*selfhost_check_expr_stage1_general_nested_overload_duplicate_rejected_with_scope[\s\S]*SelfhostCallReduceErrorKind::OverloadAmbiguous[\s\S]*selfhost_check_expr_stage1_run_general_nested_overload_argument_with_tokens[\s\S]*selfhost_check_expr_stage1_run_general_nested_overload_duplicate_with_tokens[\s\S]*selfhost_check_expr_stage1_nested_call_argument_body_line/,
    "stage1 must smoke-test general nested overload narrowing where the inner call leaves an outer continuation argument and duplicate matches stay ambiguous",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_trailing_block_argument_segment[\s\S]*SelfhostBodySegmentKind::BlockIntro[\s\S]*selfhost_check_expr_stage1_trailing_block_argument_result_ok[\s\S]*"add 1:\\n    add 1 1"[\s\S]*selfhost_check_expr_stage1_success_is_two_arg_direct_call/,
    "stage1 must smoke-test that a trailing block argument is checked as a BlockResult expression",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_trailing_block_sequence_body_line[\s\S]*selfhost_check_expr_stage1_make_trailing_block_sequence_tokens[\s\S]*selfhost_check_expr_stage1_run_trailing_block_sequence_with_tokens[\s\S]*selfhost_check_expr_stage1_make_trailing_block_sequence_non_unit_tokens[\s\S]*selfhost_check_expr_stage1_run_trailing_block_sequence_non_unit_with_tokens/,
    "stage1 must smoke-test that a multi-expression trailing block succeeds only when non-final expressions satisfy the unit expectation",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_trailing_block_nested_segment[\s\S]*SelfhostBodySegmentKind::BlockIntro[\s\S]*selfhost_check_expr_stage1_make_trailing_block_nested_tokens[\s\S]*TokenKind::Colon[\s\S]*selfhost_check_expr_stage1_trailing_block_nested_result_ok[\s\S]*"add 1 add 1: add 1 1"[\s\S]*selfhost_check_expr_stage1_trailing_block_nested_body_line/,
    "stage1 must smoke-test that a nested BlockIntro in a trailing block is recursively checked",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_segment[\s\S]*SelfhostBodySegmentKind::ExpressionLine[\s\S]*selfhost_check_expr_stage1_make_pipe_tokens[\s\S]*TokenKind::Pipe[\s\S]*selfhost_check_expr_stage1_argument_is_i32_literal_range[\s\S]*selfhost_check_expr_stage1_success_has_pipe_argument_order[\s\S]*"1 \|> add 2"[\s\S]*selfhost_check_expr_stage1_pipe_body_line/,
    "stage1 must smoke-test that a pipe expression is normalized to a direct call with left and suffix argument order preserved",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_trailing_block_segment[\s\S]*SelfhostBodySegmentKind::BlockIntro[\s\S]*selfhost_check_expr_stage1_make_pipe_trailing_block_tokens[\s\S]*TokenKind::Pipe[\s\S]*selfhost_check_expr_stage1_argument_is_block_result_checked_expr_range[\s\S]*SelfhostCheckedArgumentKind::CheckedExpr[\s\S]*selfhost_check_expr_stage1_success_has_pipe_trailing_block_argument_order[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_body_call_ok[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_node_ok[\s\S]*SelfhostCheckedExprNodeKind::BlockResult[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_root_links_block_result[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_ok_with_scope[\s\S]*"1 \|> add:\\n    add 1 1"[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_narrows_overload_with_scope[\s\S]*selfhost_check_expr_stage1_pipe_trailing_block_unexpected_rejected_with_scope[\s\S]*SelfhostCallReduceErrorKind::UnexpectedTrailingBlockArgument[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_overload_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_unexpected_with_tokens/,
    "stage1 must smoke-test that a pipe trailing block is carried as a CheckedExpr argument whose checked-tree node is BlockResult, and participates in overload narrowing",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_chain_segment[\s\S]*selfhost_check_expr_stage1_make_pipe_chain_tokens[\s\S]*TokenKind::Pipe[\s\S]*selfhost_check_expr_stage1_argument_is_checked_expr_range[\s\S]*SelfhostCheckedArgumentKind::CheckedExpr[\s\S]*selfhost_check_expr_stage1_success_has_pipe_chain_argument_order[\s\S]*selfhost_check_expr_stage1_pipe_chain_root_links_previous_direct_call[\s\S]*selfhost_checked_expr_tree_get_node tree root_expr[\s\S]*Option::Some node[\s\S]*Option::Some first[\s\S]*Option::Some second[\s\S]*selfhost_check_expr_stage1_pipe_chain_previous_call_arguments_ok tree previous_expr[\s\S]*Option::None[\s\S]*"1 \|> add 2 \|> add 3"[\s\S]*selfhost_check_expr_stage1_pipe_chain_direct_call_ok_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_chain_with_tokens/,
    "stage1 must smoke-test that a pipe chain is normalized through a checked intermediate expression",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_left_nested_overload_segment[\s\S]*"add 1 2 \|> use 3"[\s\S]*selfhost_check_expr_stage1_make_pipe_left_nested_overload_tokens[\s\S]*TokenKind::Pipe source_span_new_unchecked 0 8 10[\s\S]*TokenKind::Ident source_span_new_unchecked 0 11 14[\s\S]*selfhost_check_expr_stage1_success_has_pipe_left_nested_overload_argument_order[\s\S]*selfhost_check_expr_stage1_pipe_left_nested_root_links_child_direct_call[\s\S]*selfhost_check_expr_stage1_pipe_left_nested_overload_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions_and_named_function "add" one_arg_type two_arg_type add_span "use" two_arg_type use_span[\s\S]*selfhost_check_expr_stage1_run_pipe_left_nested_overload_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_body_line/,
    "stage1 must smoke-test that a pipe left nested overload consumes only the left range and does not use the pipe target suffix as continuation",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_make_pipe_chain_trailing_block_tokens[\s\S]*TokenKind::Pipe source_span_new_unchecked 0 11 13[\s\S]*TokenKind::Ident source_span_new_unchecked 0 23 26[\s\S]*selfhost_check_expr_stage1_make_pipe_chain_missing_right_tokens[\s\S]*TokenKind::Pipe source_span_new_unchecked 0 11 13[\s\S]*selfhost_check_expr_stage1_make_pipe_chain_literal_target_tokens[\s\S]*TokenKind::IntLiteral source_span_new_unchecked 0 14 15[\s\S]*selfhost_check_expr_stage1_make_pipe_missing_left_tokens[\s\S]*TokenKind::Pipe[\s\S]*selfhost_check_expr_stage1_make_pipe_missing_right_tokens[\s\S]*TokenKind::Pipe[\s\S]*selfhost_check_expr_stage1_make_pipe_literal_target_tokens[\s\S]*TokenKind::IntLiteral[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_target_tokens[\s\S]*TokenKind::Percent[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_function_target_tokens[\s\S]*TokenKind::KwFn[\s\S]*selfhost_check_expr_stage1_make_pipe_multi_value_left_tokens[\s\S]*"1 2 \|> add 3"[\s\S]*selfhost_check_expr_stage1_make_pipe_zero_arg_target_tokens[\s\S]*"answer"/,
    "stage1 must keep executable token fixtures for representative pipe fail-closed cases",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_chain_error_after_first_stage_with_i32[\s\S]*SelfhostCallReduceErrorKind[\s\S]*selfhost_check_expr_stage1_run_pipe_chain_error_after_first_stage_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_chain_trailing_block_rejected_with_i32[\s\S]*SelfhostCallReduceErrorKind::UnexpectedTrailingBlockArgument[\s\S]*selfhost_check_expr_stage1_run_pipe_chain_trailing_block_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_chain_failclosed_body_line[\s\S]*SelfhostCallReduceErrorKind::PipeMissingRightTarget[\s\S]*SelfhostCallReduceErrorKind::PipeRightTargetUnsupported[\s\S]*selfhost_check_expr_stage1_run_pipe_chain_trailing_block_with_tokens/,
    "stage1 must check pipe-chain downstream fail-closed fixtures against pipe-specific typed errors",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_run_pipe_missing_left_with_tokens[\s\S]*SelfhostCallReduceErrorKind::PipeMissingLeftOperand[\s\S]*selfhost_check_expr_stage1_run_pipe_missing_right_with_tokens[\s\S]*SelfhostCallReduceErrorKind::PipeMissingRightTarget[\s\S]*selfhost_check_expr_stage1_run_pipe_literal_target_with_tokens[\s\S]*SelfhostCallReduceErrorKind::PipeRightTargetUnsupported[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_mismatch_with_tokens[\s\S]*SelfhostCallReduceErrorKind::PipeTargetAscriptionTypeMismatch[\s\S]*selfhost_check_expr_stage1_pipe_multi_value_left_rejected_with_i32[\s\S]*SelfhostCallReduceErrorKind::PipeLeftSegmentNotSingleValue[\s\S]*selfhost_check_expr_stage1_pipe_zero_arg_target_rejected_with_i32[\s\S]*SelfhostCallReduceErrorKind::PipeTargetRequiresInput/,
    "stage1 must check pipe fail-closed fixtures against pipe-specific typed errors",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_failclosed_body_line[\s\S]*selfhost_check_expr_stage1_make_pipe_missing_left_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_missing_left_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_missing_right_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_missing_right_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_literal_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_literal_target_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_mismatch_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_function_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_overload_no_match_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_function_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_duplicate_match_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_multi_value_left_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_multi_value_left_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_zero_arg_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_zero_arg_target_with_tokens/,
    "stage1 public pipe smoke must run the representative pipe fail-closed cases",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_value_context_with_two_functions[\s\S]*SelfhostDefKind::Function[\s\S]*SelfhostDefKind::Function[\s\S]*selfhost_check_expr_stage1_pipe_ascribed_target_narrows_overload_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions "add" one_arg_type two_arg_type add_span[\s\S]*"1 \|> %fn i32 fn i32 i32 add 2"[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_overload_narrowing_with_tokens/,
    "stage1 must smoke-test that a pipe target function type ascription narrows same-name overload candidates",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_unascribed_target_narrows_overload_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions "add" one_arg_type two_arg_type add_span[\s\S]*"1 \|> add 2"[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_no_applicable_rejected_with_scope[\s\S]*SelfhostCallReduceErrorKind::PipeTargetNoApplicableCandidate[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_duplicate_match_rejected_with_scope[\s\S]*SelfhostCallReduceErrorKind::PipeTargetAmbiguous[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_single_probe_unsupported_succeeds_with_scope[\s\S]*"1 \|> add %i32 2"[\s\S]*selfhost_check_expr_stage1_success_has_pipe_suffix_ascribed_argument_order[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_named_suffix_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions_and_typed_value "add" one_arg_type two_arg_type add_span "x" i32_type x_span[\s\S]*"1 \|> add x"[\s\S]*selfhost_check_expr_stage1_success_has_pipe_suffix_named_argument_order[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_nested_suffix_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions_and_named_function "add" one_arg_type two_arg_type add_span "sum" two_arg_type sum_span[\s\S]*"1 \|> add sum 2 3"[\s\S]*selfhost_check_expr_stage1_pipe_suffix_nested_call_root_links_child_direct_call &success "sum"[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_same_name_nested_suffix_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_value_context_with_two_functions "add" one_arg_type two_arg_type add_span[\s\S]*"1 \|> add add 2 3"[\s\S]*selfhost_check_expr_stage1_pipe_suffix_nested_call_root_links_child_direct_call &success "add"[\s\S]*selfhost_check_expr_stage1_pipe_left_nested_overload_succeeds_with_scope[\s\S]*"add 1 2 \|> use 3"/,
    "stage1 must smoke-test non-ascribed pipe target argument narrowing, its fail-closed cases, single source-backed suffix success cases, and same-name nested suffix checked tree topology",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_run_pipe_unascribed_target_overload_narrowing_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_no_applicable_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_duplicate_match_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_single_probe_unsupported_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_named_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_nested_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_same_name_nested_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_left_nested_overload_with_tokens/,
    "stage1 must expose token-owner runners for non-ascribed pipe target and pipe-left nested overload fixtures",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_run_pipe_ascribed_target_overload_no_match_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_zero_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_ascribed_target_overload_no_match_rejected_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_duplicate_match_with_i32[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_ascribed_target_duplicate_match_rejected_with_scope/,
    "stage1 must run runtime negative smoke for ascribed pipe target overload no-match and duplicate-match cases",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_run_pipe_unascribed_target_overload_narrowing_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_narrows_overload_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_no_applicable_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_zero_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_no_applicable_rejected_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_duplicate_match_with_i32[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_duplicate_match_rejected_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_single_probe_unsupported_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_single_probe_unsupported_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_named_suffix_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_named_suffix_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_nested_suffix_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_nested_suffix_succeeds_with_scope[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_same_name_nested_suffix_with_i32[\s\S]*selfhost_check_expr_stage1_add_one_i32_function[\s\S]*selfhost_check_expr_stage1_add_two_i32_function[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_same_name_nested_suffix_succeeds_with_scope/,
    "stage1 must run runtime smoke for non-ascribed pipe target unique-match, no-applicable, duplicate-match, and source-backed suffix success cases",
);
assert.match(
    source,
    /selfhost_check_expr_stage1_pipe_unascribed_target_argument_narrowing_body_line[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_overload_narrowing_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_no_applicable_with_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_duplicate_match_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_suffix_ascribed_argument_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_single_probe_unsupported_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_suffix_named_argument_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_named_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_suffix_nested_call_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_nested_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_suffix_same_name_nested_call_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_unascribed_target_same_name_nested_suffix_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_body_line[\s\S]*selfhost_check_expr_stage1_make_pipe_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_function_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_ascribed_function_target_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_ascribed_target_overload_narrowing_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_unascribed_target_argument_narrowing_body_line[\s\S]*selfhost_check_expr_stage1_make_pipe_chain_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_chain_with_tokens[\s\S]*selfhost_check_expr_stage1_pipe_chain_failclosed_body_line[\s\S]*selfhost_check_expr_stage1_pipe_failclosed_body_line[\s\S]*pub fn selfhost_check_expr_stage1_body_line[\s\S]*selfhost_check_expr_stage1_pipe_body_line[\s\S]*selfhost_check_expr_stage1_trailing_block_argument_body_line[\s\S]*selfhost_check_expr_stage1_trailing_block_sequence_body_line[\s\S]*selfhost_check_expr_stage1_trailing_block_nested_body_line/,
    "public stage1 body-line smoke must include pipe, pipe chain, ascribed pipe overload narrowing, non-ascribed pipe argument narrowing, block result, block sequence, and nested BlockIntro fixtures",
);
assert.match(
    source,
    /pub fn selfhost_check_expr_stage1_pipe_body_line[\s\S]*selfhost_check_expr_stage1_make_pipe_trailing_block_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_trailing_block_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_overload_with_tokens[\s\S]*selfhost_check_expr_stage1_make_pipe_trailing_block_tokens[\s\S]*selfhost_check_expr_stage1_run_pipe_trailing_block_unexpected_with_tokens/,
    "public pipe smoke must execute success, overload narrowing, and surplus-block rejection for pipe trailing blocks",
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
assertContainsInOrder(
    source,
    [
        "fn selfhost_check_expr_stage1_value_context_with_two_functions_and_typed_value",
        "SelfhostDefKind::Function",
        "SelfhostDefKind::Function",
        "SelfhostDefKind::Local",
        "selfhost_value_type_evidence_new value_def_id value_type value_span",
        "selfhost_callable_signature_new first_def_id first_type",
        "selfhost_callable_signature_new second_def_id second_type",
    ],
    "stage1 must build one context that contains same-name pipe target overloads and a DefId-linked typed local value",
);
assertContainsInOrder(
    source,
    [
        "selfhost_check_expr_stage1_make_pipe_suffix_named_argument_tokens",
        "`1 |> add x`",
        "TokenKind::Ident source_span_new_unchecked 0 9 10",
        "selfhost_check_expr_stage1_argument_is_named_value_range",
        "SelfhostCheckedArgumentKind::NamedValue _identity:",
        "selfhost_check_expr_stage1_success_has_pipe_suffix_named_argument_order",
        "selfhost_check_expr_stage1_pipe_unascribed_target_named_suffix_succeeds_with_scope",
        "\"1 |> add x\"",
    ],
    "stage1 must smoke-test that a source-backed pipe suffix named value becomes a NamedValue checked argument",
);
assertContainsInOrder(
    source,
    [
        "selfhost_check_expr_stage1_make_pipe_suffix_nested_call_tokens",
        "`1 |> add sum 2 3`",
        "selfhost_check_expr_stage1_make_pipe_suffix_same_name_nested_call_tokens",
        "`1 |> add add 2 3`",
    ],
    "stage1 must keep token fixtures for source-backed nested suffix calls with a single inner name and with the same overloaded name",
);
assertContainsInOrder(
    source,
    [
        "SelfhostCheckedArgumentKind::CheckedExpr",
        "selfhost_check_expr_stage1_pipe_suffix_nested_call_child_arguments_ok",
        "string_search::str_eq call.candidate.name expected_name",
        "selfhost_check_expr_stage1_success_has_pipe_suffix_nested_argument_order",
        "selfhost_check_expr_stage1_pipe_suffix_nested_call_root_links_child_direct_call",
        "selfhost_check_expr_stage1_pipe_unascribed_target_nested_suffix_succeeds_with_scope",
        "\"1 |> add sum 2 3\"",
        "selfhost_check_expr_stage1_pipe_suffix_nested_call_root_links_child_direct_call &success \"sum\"",
        "selfhost_check_expr_stage1_pipe_unascribed_target_same_name_nested_suffix_succeeds_with_scope",
        "\"1 |> add add 2 3\"",
        "selfhost_check_expr_stage1_pipe_suffix_nested_call_root_links_child_direct_call &success \"add\"",
    ],
    "stage1 must smoke-test that source-backed pipe suffix nested calls, including same-name overloaded nested calls, become CheckedExpr arguments with child direct-call topology",
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
    /fn selfhost_call_reduce_trailing_block_segment_result[\s\S]*SelfhostBodySegmentKind::ExpressionLine:[\s\S]*selfhost_block_body_result_input_from_segment tokens source scope signatures segment[\s\S]*selfhost_call_reduce_prefix_with_source_in_tree tokens source arena checked_tree prefix scope value_types signatures candidates expected none[\s\S]*SelfhostBodySegmentKind::BlockIntro:[\s\S]*selfhost_expr_prefix_list_from_syntax_range tokens segment\.head[\s\S]*selfhost_callable_candidates_collect_for_prefix source tokens &prefix scope signatures[\s\S]*selfhost_trailing_block_argument_new segment\.body block_span[\s\S]*selfhost_call_reduce_prefix_with_source_in_tree tokens source arena checked_tree &prefix scope value_types signatures &candidates expected some trailing/,
    "call reduction must use one segment dispatcher for expression lines and nested BlockIntro bodies without flattening the body range",
);
assert.match(
    source,
    /fn selfhost_call_reduce_trailing_block_body_single_result[\s\S]*SelfhostTypeExpectationSource::BlockResult[\s\S]*selfhost_call_reduce_trailing_block_segment_result tokens source arena checked_tree scope value_types signatures some block_expected segment[\s\S]*selfhost_checked_expr_tree_add_block_result body_tree expected_type block_span body_expr[\s\S]*selfhost_checked_argument_checked_expr item_index item_index expected_type block_span expr_id[\s\S]*selfhost_call_reduce_trailing_block_body_result[\s\S]*selfhost_block_body_result_segments_from_trailing_block tokens block_arg[\s\S]*selfhost_call_reduce_trailing_block_body_segments_result tokens source arena checked_tree scope value_types signatures expected_type segments block_arg\.span item_index/,
    "call reduction must segment a trailing block body once, then wrap the checked segment root as a CheckedExpr payload pointing at a BlockResult node",
);
assert.match(
    source,
    /fn selfhost_call_reduce_trailing_block_sequence_loop[\s\S]*selfhost_checked_expr_tree_add_block_sequence checked_tree expected_type block_span body_exprs[\s\S]*selfhost_checked_argument_checked_expr item_index item_index expected_type block_span expr_id/,
    "multi-expression trailing block reduction must return a CheckedExpr BlockSequence payload at loop completion",
);
assert.match(
    source,
    /fn selfhost_call_reduce_trailing_block_sequence_line_expected[\s\S]*SelfhostTypeId fn SelfhostTypeId[\s\S]*\\expected_type\\unit_type\\idx\\count\\line_span[\s\S]*eq idx sub count 1[\s\S]*SelfhostTypeExpectationSource::BlockResult[\s\S]*SelfhostTypeExpectationSource::BlockSequenceDiscardedExpression/,
    "multi-expression trailing block reduction must use the outer expected type only for the last expression and a pre-resolved unit type for discarded earlier expressions",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_operator_index_loop[\s\S]*SelfhostExprPrefixItemKind::PipeOperator[\s\S]*Result::Ok some existing[\s\S]*fn selfhost_call_reduce_pipe_next_operator_index[\s\S]*fn selfhost_call_reduce_pipe_prefix_with_source[\s\S]*PipeMissingLeftOperand[\s\S]*PipeMissingRightTarget[\s\S]*selfhost_callable_candidates_collect_for_head_item source tokens rhs_head scope signatures/,
    "pipe reduction must detect the first pipe for the source-backed reducer and collect each right-hand target from scope and signatures",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_finish_candidate_with_checked_left[\s\S]*selfhost_call_reduce_push_checked_argument arena checked_arguments checked_tree left_argument rhs_head\.span[\s\S]*selfhost_call_reduce_argument_type_check_loop_with_source tokens source first_arena first_arguments first_tree prefix scope value_types signatures candidate param_count 1 rhs_suffix_index item_count trailing_block rhs_head[\s\S]*selfhost_checked_expr_tree_add_direct_call_borrowed checked_tree2 candidate result_type call_span &checked_arguments2[\s\S]*fn selfhost_call_reduce_pipe_single_candidate_with_source[\s\S]*selfhost_call_reduce_argument_match_at_with_source_or_nested tokens source arena checked_tree prefix scope value_types signatures SelfhostNestedArgumentBoundary::FinalRange candidate param_count 0 0 pipe_index first_arg_type rhs_head[\s\S]*selfhost_call_reduce_pipe_finish_candidate_with_checked_left tokens source checked_arena left_tree prefix scope value_types signatures pipe_candidates candidate expected trailing_block rhs_head item_count rhs_suffix_index param_count checked_argument/,
    "pipe reduction must type-check the left side as the first argument and then reuse the shared checked-left finisher for the right-side suffix",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_checked_left_candidate_with_source[\s\S]*selfhost_type_arena_function_arg &arena candidate\.callable_type 0[\s\S]*selfhost_type_arena_types_equal &arena left_argument\.value_type first_arg_type[\s\S]*selfhost_call_reduce_pipe_finish_candidate_with_checked_left tokens source arena checked_tree prefix scope value_types signatures pipe_candidates candidate expected trailing_block rhs_head item_count rhs_suffix_index param_count left_argument[\s\S]*fn selfhost_call_reduce_pipe_chain_continue_after_step[\s\S]*selfhost_checked_argument_checked_expr 0 next_pipe_index call\.result_type call\.span root_expr[\s\S]*selfhost_call_reduce_pipe_chain_continue_with_left/,
    "pipe chain reduction must pass the previous checked expression as the next stage's first argument",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_candidate_matches_target_ascription[\s\S]*selfhost_type_arena_types_equal arena expectation\.expected_type candidate\.callable_type[\s\S]*fn selfhost_call_reduce_pipe_target_ascription_match_count_loop[\s\S]*selfhost_call_reduce_pipe_candidate_matches_target_ascription arena candidate expectation[\s\S]*fn selfhost_call_reduce_pipe_target_ascription_first_match_loop[\s\S]*selfhost_call_reduce_pipe_candidate_matches_target_ascription arena candidate expectation/,
    "ascribed pipe target narrowing must share one callable-type equality rule across counting and selection",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_target_ascription_select_candidate[\s\S]*selfhost_call_reduce_pipe_target_ascription_match_count_loop[\s\S]*eq match_count 0[\s\S]*PipeTargetAscriptionTypeMismatch[\s\S]*gt match_count 1[\s\S]*PipeTargetAmbiguous[\s\S]*selfhost_call_reduce_pipe_target_ascription_first_match_loop/,
    "ascribed pipe target overload narrowing must distinguish zero, one, and multiple callable-type matches",
);
assertContainsInOrder(
    source,
    [
        "pub enum SelfhostPipeCandidateApplicability:",
        "Match",
        "NoMatch",
        "SourceBackedRequired",
        "SelectionBlockedUnsupported",
        "pub struct SelfhostPipeCandidateProbeSummary:",
        "match_count %i32",
        "source_backed_required_count %i32",
        "blocked_unsupported_count %i32",
    ],
    "pipe target argument narrowing must keep match, no-match, source-backed retry, and blocked unsupported states as typed data",
);
assertContainsInOrder(
    source,
    [
        "fn selfhost_call_reduce_pipe_candidate_applicability_from_error",
        "SelfhostCallReduceErrorKind::ArgumentTypeMismatch:",
        "SelfhostPipeCandidateApplicability::NoMatch",
        "SelfhostCallReduceErrorKind::ArgumentAscriptionProjectionFailed:",
        "SelfhostPipeCandidateApplicability::SourceBackedRequired",
        "SelfhostCallReduceErrorKind::ArgumentNamedValueEvidenceMissing:",
        "SelfhostPipeCandidateApplicability::SourceBackedRequired",
        "SelfhostCallReduceErrorKind::ArgumentFunctionValueGenericUnsupported:",
        "SelfhostPipeCandidateApplicability::SelectionBlockedUnsupported",
        "SelfhostCallReduceErrorKind::PipeTargetNoApplicableCandidate:",
        "SelfhostPipeCandidateApplicability::NoMatch",
        "SelfhostCallReduceErrorKind::UnsupportedArgumentExpression:",
        "SelfhostPipeCandidateApplicability::SourceBackedRequired",
        "SelfhostCallReduceErrorKind::GenericInferenceUnsupported:",
        "SelfhostPipeCandidateApplicability::SelectionBlockedUnsupported",
        "SelfhostCallReduceErrorKind::ExpectedTypeMismatch:",
        "SelfhostPipeCandidateApplicability::NoMatch",
        "SelfhostCallReduceErrorKind::InternalInvariant:",
        "SelfhostPipeCandidateApplicability::SelectionBlockedUnsupported",
    ],
    "pipe target argument narrowing must classify ordinary candidate mismatch separately from source-backed retryable evidence and blocked unsupported states",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_candidate_suffix_applicability[\s\S]*Option SelfhostTrailingBlockArgument[\s\S]*\\arena\\prefix\\candidate\\param_count\\expected\\trailing_block\\rhs_head\\item_count\\rhs_suffix_index:[\s\S]*Result::Ok _unit:[\s\S]*Option::Some _block_arg:[\s\S]*SelfhostPipeCandidateApplicability::NoMatch[\s\S]*Option::None:[\s\S]*selfhost_call_reduce_expected_result arena candidate expected[\s\S]*Result::Err e:[\s\S]*Option::Some _block_arg:[\s\S]*SelfhostCallReduceErrorKind::PartialApplicationRejected:[\s\S]*SelfhostPipeCandidateApplicability::SourceBackedRequired/,
    "pipe target suffix probing must reject surplus trailing blocks while keeping block-satisfied partial applications for source-backed reduction",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_candidate_applicability[\s\S]*Option SelfhostTrailingBlockArgument[\s\S]*\\arena\\prefix\\candidate\\expected\\trailing_block\\left_argument[\s\S]*selfhost_call_reduce_generic_state_error candidate[\s\S]*Option::Some checked_left:[\s\S]*selfhost_type_arena_types_equal arena checked_left\.value_type first_arg_type[\s\S]*Option::None:[\s\S]*selfhost_expr_argument_match_at arena prefix 0 pipe_index first_arg_type[\s\S]*selfhost_call_reduce_pipe_candidate_suffix_applicability arena prefix candidate param_count expected trailing_block rhs_head item_count rhs_suffix_index/,
    "pipe target argument narrowing must probe the checked-left path and the source-less left segment without mutating checked tree state",
);
assertContainsInOrder(
    source,
    [
        "fn selfhost_call_reduce_pipe_candidate_probe_summary_loop",
        "\\arena\\prefix\\pipe_candidates\\expected\\trailing_block\\left_argument",
        "selfhost_call_reduce_pipe_candidate_applicability arena prefix candidate expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index",
        "fn selfhost_call_reduce_pipe_candidate_probe_first_match_loop",
        "\\arena\\prefix\\pipe_candidates\\expected\\trailing_block\\left_argument",
        "selfhost_call_reduce_pipe_candidate_applicability arena prefix candidate expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index",
        "fn selfhost_call_reduce_pipe_candidate_probe_first_source_backed_required_loop",
        "\\arena\\prefix\\pipe_candidates\\expected\\trailing_block\\left_argument",
        "selfhost_call_reduce_pipe_candidate_applicability arena prefix candidate expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index",
        "fn selfhost_call_reduce_pipe_target_argument_select_candidate",
        "\\arena\\prefix\\pipe_candidates\\expected\\trailing_block\\left_argument",
        "selfhost_call_reduce_pipe_candidate_probe_summary_loop arena prefix pipe_candidates expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index 0 candidate_count initial",
        "selfhost_call_reduce_pipe_candidate_probe_first_source_backed_required_loop arena prefix pipe_candidates expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index 0 candidate_count",
        "selfhost_call_reduce_pipe_candidate_probe_first_match_loop arena prefix pipe_candidates expected trailing_block left_argument rhs_head item_count pipe_index rhs_suffix_index 0 candidate_count",
    ],
    "non-ascribed pipe target narrowing must pass trailing-block evidence through every probe pass before selecting the single source-backed finisher candidate",
);
assertContainsInOrder(
    source,
    [
        "fn selfhost_call_reduce_pipe_candidate_probe_first_match_loop",
        "SelfhostPipeCandidateApplicability::Match:",
        "Result::Ok candidate",
        "SelfhostPipeCandidateApplicability::SourceBackedRequired:",
        "selfhost_call_reduce_pipe_candidate_probe_first_match_loop",
        "SelfhostPipeCandidateApplicability::SelectionBlockedUnsupported:",
        "selfhost_call_reduce_pipe_candidate_probe_first_match_loop",
        "fn selfhost_call_reduce_pipe_candidate_probe_first_source_backed_required_loop",
        "SelfhostPipeCandidateApplicability::SourceBackedRequired:",
        "Result::Ok candidate",
        "SelfhostPipeCandidateApplicability::SelectionBlockedUnsupported:",
        "PipeTargetInternalInvariant",
        "fn selfhost_call_reduce_pipe_target_argument_select_candidate",
        "selfhost_call_reduce_pipe_candidate_probe_summary_loop",
        "gt summary.blocked_unsupported_count 0",
        "PipeTargetAmbiguous",
        "gt summary.source_backed_required_count 0",
        "eq summary.match_count 0",
        "eq summary.source_backed_required_count 1",
        "selfhost_call_reduce_pipe_candidate_probe_first_source_backed_required_loop",
        "PipeTargetAmbiguous",
        "eq summary.match_count 0",
        "PipeTargetNoApplicableCandidate",
        "gt summary.match_count 1",
        "PipeTargetAmbiguous",
        "selfhost_call_reduce_pipe_candidate_probe_first_match_loop",
    ],
    "non-ascribed pipe target narrowing must distinguish blocked unsupported probes, zero applicable candidates, unique matches, and a single source-backed required candidate",
);
assertContainsInOrder(
    pipeCandidates,
    [
        "eq candidate_count 0",
        "PipeTargetUnresolved",
        "match target_ascription:",
        "Option::Some expectation:",
        "selfhost_call_reduce_pipe_target_ascription_select_candidate",
        "Option::None:",
        "gt candidate_count 1",
        "selfhost_call_reduce_pipe_target_argument_select_candidate",
    ],
    "pipe target ascription must narrow candidates before non-ascribed argument-based candidate selection",
);
assert.match(
    source,
    /fn selfhost_call_reduce_pipe_ascribed_target_with_source[\s\S]*selfhost_expr_ascription_project_head_expectation tokens source arena syntax_range[\s\S]*selfhost_expr_ascription_head_projection_expression_first_token[\s\S]*selfhost_call_reduce_pipe_find_prefix_item_by_token prefix add rhs_index 1 item_count expression_token[\s\S]*selfhost_call_reduce_pipe_candidates_with_source tokens source projected_arena checked_tree prefix scope value_types signatures pipe_candidates expected some target_expectation trailing_block left_argument target_item item_count pipe_index add target_index 1/,
    "ascribed pipe targets must project the function type, recover the named target token, and start the suffix scan after that target",
);
assert.match(
    source,
    /pub fn selfhost_call_reduce_prefix %fn[\s\S]*SelfhostExprPrefixItemKind::NamedValue:[\s\S]*selfhost_call_reduce_named_prefix arena prefix candidates expected head item_count[\s\S]*SelfhostCallReduceErrorKind::UnsupportedPrefixItem/,
    "source-less call reduction must remain named-head only and must not accept pipe without source-backed evidence",
);
assert.match(
    source,
    /fn selfhost_call_reduce_trailing_block_sequence_loop[\s\S]*\\expected_type\\unit_type\\segments\\idx\\count\\item_index\\block_span[\s\S]*selfhost_call_reduce_trailing_block_sequence_line_expected expected_type unit_type idx count line_span[\s\S]*selfhost_call_reduce_trailing_block_segment_result tokens source arena checked_tree scope value_types signatures line_expected segment[\s\S]*v::push body_exprs body_expr[\s\S]*selfhost_call_reduce_trailing_block_sequence_loop tokens source checked_arena body_tree next_body_exprs scope value_types signatures expected_type unit_type/,
    "multi-expression trailing block reduction must check each segment through the shared dispatcher and collect checked root expression ids",
);
assert.match(
    source,
    /fn selfhost_call_reduce_trailing_block_body_segments_result[\s\S]*let count %i32 selfhost_body_segment_list_len &segments[\s\S]*eq count 0[\s\S]*TrailingBlockBodyEmpty[\s\S]*eq count 1[\s\S]*selfhost_call_reduce_trailing_block_body_single_result tokens source arena checked_tree scope value_types signatures expected_type segments block_span item_index[\s\S]*selfhost_type_arena_find_kind &arena SelfhostTypeKind::Unit[\s\S]*selfhost_call_reduce_trailing_block_sequence_loop tokens source arena checked_tree body_exprs scope value_types signatures expected_type unit_type[\s\S]*TrailingBlockBodyUnitTypeMissing/,
    "multi-segment trailing block bodies must segment once, resolve the unit type once, and branch to the BlockSequence reducer instead of using a MultipleSegments error fallback",
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

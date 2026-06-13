#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_evidence_producer.nepl";
const classifierRelPath = "stdlib/neplg2/core/check/module/memo_trait_operation_classifier.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const operationEvidenceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_evidence.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const classifier = read(classifierRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const operationEvidence = read(operationEvidenceRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_evidence_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "operation evidence producer must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("ty 層の `memo_trait_operation_evidence` は session-local table transport だけを担当") &&
        source.includes("public impl header、method body purity、Drop なし proof の checker-layer evidence を畳みます"),
    "docs must separate ty transport from checker-layer producer responsibility",
);
assert.ok(
    source.includes("flattened public declaration payload hash ではなく `SelfhostMemoTraitPublicImplHeaderInput`") &&
        source.includes("resolved_type_shape_hash") &&
        source.includes("classified_trait_application_shape_hash") &&
        source.includes("operation 別 evidence matrix"),
    "docs must require typed public impl header input, target shape evidence, shape-bound trait-operation classifier evidence, and operation-specific evidence applicability",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record は accepted operation proof authority に入りません"),
    "docs must exclude source/display/diagnostic/module path/HIR/Resource/backend/proof-store authority",
);
assert.ok(
    source.includes("trait impl table の探索、trait application shape から operation classifier を作る実体、method body purity checker の実体、Drop なし proof の導出、generic impl binder / bound 詳細 evidence は後続 stage の責務"),
    "docs must mark scanner, purity checker, drop proof derivation, and generic impl evidence as later stages",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_evidence_producer/,
    "operation evidence producer must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_evidence_producer/,
    "checker-layer producer must not be registered in the ty source list",
);
assert.match(
    code,
    /#import "\.\/memo_trait_operation_classifier" as \*/,
    "operation evidence producer must import classifier-owned shape-bound evidence",
);
assert.ok(
    operationEvidence.includes("この module は Drop なし proof を推測で作りません"),
    "ty operation evidence transport must keep Drop proof derivation outside the table layer",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyEvidence:",
        "Pure",
        "Missing",
        "Impure",
        "Unknown",
        "NotRequired",
    ],
    "method body evidence must be a typed enum with proven, missing, impure, unknown, and not-required states",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropEvidence:",
        "NoDropRequired",
        "PureDrop",
        "Missing",
        "ImpureDrop",
        "Unknown",
        "NotRequired",
    ],
    "drop evidence must distinguish no-drop, pure-drop, missing, impure, unknown, and not-required states",
);
assertOrdered(
    classifier,
    [
        "pub struct SelfhostMemoTraitOperationClassifierEvidence:",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "classified_trait_application_shape_hash %Option i32",
    ],
    "trait-operation classifier evidence must be owned by the classifier module and bind the classified operation to the trait application shape it inspected",
);
assert.doesNotMatch(
    code,
    /pub struct SelfhostMemoTraitOperationClassifierEvidence|pub fn selfhost_memo_trait_operation_classifier_evidence_new/,
    "operation evidence producer must not own classifier evidence types or constructors",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_operation_classifier_evidence_new/,
    "operation evidence producer must not call the classifier evidence constructor directly",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationEvidenceProducerInput:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "trait_operation %SelfhostMemoTraitOperationClassifierEvidence",
        "impl_header %SelfhostMemoTraitPublicImplHeaderInput",
        "resolved_type_shape_hash %Option i32",
        "method_body %SelfhostMemoTraitOperationMethodBodyEvidence",
        "drop_evidence %SelfhostMemoTraitOperationDropEvidence",
    ],
    "producer input must carry type id, operation kind, trait-operation classifier, public impl header input, target shape evidence, method body evidence, and drop evidence",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationEvidenceProducerErrorKind:",
        "HeaderRejected %SelfhostMemoTraitPublicImplHeaderErrorKind",
        "TargetTypeShapeMissing",
        "TargetTypeShapePlaceholder",
        "TargetTypeShapeMismatch",
        "TraitOperationClassifierShapeMissing",
        "TraitOperationClassifierShapePlaceholder",
        "TraitOperationClassifierShapeMismatch",
        "TraitOperationMismatch",
        "MethodBodyEvidenceRequired",
        "UnexpectedMethodBodyEvidence",
        "DropEvidenceRequired",
        "UnexpectedDropEvidence",
    ],
    "producer errors must distinguish public header rejection, target shape failures, classifier shape failures, trait-operation mismatch, and operation-specific evidence misuse",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_impl_header_result"),
    [
        "selfhost_memo_trait_public_impl_header_evidence_result impl_header",
        "Result::Ok unit",
        "HeaderRejected e",
    ],
    "impl header validator must re-use the public impl header producer instead of trusting flattened declaration evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_target_shape_match_result"),
    [
        "selfhost_memo_trait_operation_evidence_producer_resolved_type_shape_result input.resolved_type_shape_hash",
        "selfhost_memo_trait_operation_evidence_producer_impl_target_shape_result input.impl_header",
        "eq resolved_shape impl_shape",
        "Result::Ok unit",
        "TargetTypeShapeMismatch",
    ],
    "producer must check that TypeId shape evidence matches the impl header target shape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_trait_shape_match_result"),
    [
        "selfhost_memo_trait_operation_evidence_producer_impl_trait_shape_result input.impl_header",
        "selfhost_memo_trait_operation_evidence_producer_classifier_shape_result input.trait_operation",
        "eq impl_shape classifier_shape",
        "Result::Ok unit",
        "TraitOperationClassifierShapeMismatch",
    ],
    "producer must check that classifier evidence was produced for the impl header trait application shape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_trait_operation_match_result"),
    [
        "selfhost_memo_trait_operation_evidence_kind_eq input.operation input.trait_operation.operation",
        "Result::Ok unit",
        "TraitOperationMismatch",
    ],
    "producer must require trait-operation classifier evidence to match the requested operation kind",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_method_body_status"),
    [
        "SelfhostMemoTraitOperationMethodBodyEvidence::Pure:",
        "SelfhostMemoTraitAggregateProofStatus::Proven",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Missing:",
        "SelfhostMemoTraitAggregateProofStatus::Missing",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Impure:",
        "SelfhostMemoTraitAggregateProofStatus::Impure",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Unknown:",
        "SelfhostMemoTraitAggregateProofStatus::Unknown",
        "SelfhostMemoTraitOperationMethodBodyEvidence::NotRequired:",
        "SelfhostMemoTraitAggregateProofStatus::Proven",
    ],
    "method body evidence must fold to aggregate status without using producer errors for missing/impure/unknown",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_drop_status"),
    [
        "SelfhostMemoTraitOperationDropEvidence::NoDropRequired:",
        "SelfhostMemoTraitAggregateProofStatus::Proven",
        "SelfhostMemoTraitOperationDropEvidence::PureDrop:",
        "SelfhostMemoTraitAggregateProofStatus::Proven",
        "SelfhostMemoTraitOperationDropEvidence::Missing:",
        "SelfhostMemoTraitAggregateProofStatus::Missing",
        "SelfhostMemoTraitOperationDropEvidence::ImpureDrop:",
        "SelfhostMemoTraitAggregateProofStatus::Impure",
        "SelfhostMemoTraitOperationDropEvidence::Unknown:",
        "SelfhostMemoTraitAggregateProofStatus::Unknown",
        "SelfhostMemoTraitOperationDropEvidence::NotRequired:",
        "SelfhostMemoTraitAggregateProofStatus::Proven",
    ],
    "drop evidence must fold no-drop and pure-drop to Proven and preserve missing/impure/unknown statuses",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_evidence_applicability_result"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_evidence_producer_method_not_required_result input.method_body",
        "selfhost_memo_trait_operation_evidence_producer_drop_not_required_result input.drop_evidence",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "selfhost_memo_trait_operation_evidence_producer_method_not_required_result input.method_body",
        "selfhost_memo_trait_operation_evidence_producer_drop_required_result input.drop_evidence",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "selfhost_memo_trait_operation_evidence_producer_method_required_result input.method_body",
        "selfhost_memo_trait_operation_evidence_producer_drop_not_required_result input.drop_evidence",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "selfhost_memo_trait_operation_evidence_producer_method_required_result input.method_body",
        "selfhost_memo_trait_operation_evidence_producer_drop_not_required_result input.drop_evidence",
    ],
    "producer must validate operation-specific evidence applicability before folding NotRequired to Proven",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_status_merge"),
    [
        "selfhost_memo_trait_operation_evidence_producer_status_rank body_status",
        "selfhost_memo_trait_operation_evidence_producer_status_rank drop_status",
        "then:",
        "body_status",
        "else:",
        "drop_status",
    ],
    "status merge must conservatively keep the stronger non-proven status",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_status_result"),
    [
        "selfhost_memo_trait_operation_evidence_producer_impl_header_result input.impl_header",
        "selfhost_memo_trait_operation_evidence_producer_target_shape_match_result input",
        "selfhost_memo_trait_operation_evidence_producer_trait_shape_match_result input",
        "selfhost_memo_trait_operation_evidence_producer_trait_operation_match_result input",
        "selfhost_memo_trait_operation_evidence_producer_evidence_applicability_result input",
        "selfhost_memo_trait_operation_evidence_producer_method_body_status input.method_body",
        "selfhost_memo_trait_operation_evidence_producer_drop_status input.drop_evidence",
        "selfhost_memo_trait_operation_evidence_producer_status_merge body_status drop_status",
    ],
    "status result must validate impl header, target shape, trait operation, and fold method/drop evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_record_result"),
    [
        "selfhost_memo_trait_operation_evidence_producer_status_result input",
        "selfhost_memo_trait_operation_evidence_record_new input.type_id input.operation status",
    ],
    "record producer must construct the ty operation evidence record through the existing transport constructor",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_evidence_producer_record_result_header_rejected_eq"),
    [
        "Result::Err e:",
        "SelfhostMemoTraitOperationEvidenceProducerErrorKind::HeaderRejected header_error:",
        "selfhost_memo_trait_public_impl_header_error_kind_eq header_error expected",
        "SelfhostMemoTraitOperationEvidenceProducerErrorKind::TraitOperationMismatch:",
        "false",
    ],
    "record result helper must compare nested public impl header errors without constructing payload errors in doctests",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_operation_evidence_producer_stage0_classifier_result",
        "selfhost_memo_trait_operation_trait_application_shape_hash_result source 0",
        "selfhost_memo_trait_operation_classifier_evidence_result input",
        "selfhost_memo_trait_operation_evidence_producer_stage0_classifier_shape_result copy_classifier",
        "selfhost_memo_trait_operation_evidence_producer_stage0_impl_header_with_trait_shape copy_shape",
        "selfhost_memo_trait_operation_evidence_producer_stage0_impl_header_with_trait_shape hash_shape",
        "accepted_input",
        "SelfhostMemoTraitOperationEvidenceKind::Copy",
        "copy_classifier",
        "some 7101",
        "SelfhostMemoTraitOperationMethodBodyEvidence::NotRequired",
        "drop_input",
        "SelfhostMemoTraitOperationDropEvidence::NoDropRequired",
        "missing_input",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Missing",
        "impure_input",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Impure",
        "unknown_drop_input",
        "SelfhostMemoTraitOperationDropEvidence::Unknown",
        "target_mismatch_input",
        "some 7102",
        "classifier_shape_mismatch_input",
        "SelfhostMemoTraitOperationEvidenceKind::Hash",
        "hash_classifier",
        "copy_header",
        "trait_operation_mismatch_input",
        "hash_classifier",
        "hash_header",
        "eq_method_not_required_input",
        "drop_evidence_not_required_input",
        "generic_header_input",
        "eq_method_not_required_rejected",
        "drop_evidence_not_required_rejected",
        "selfhost_memo_trait_operation_evidence_producer_stage0_projection_from_results",
    ],
    "stage0 must exercise accepted, missing, impure, unknown-drop, target mismatch, classifier shape mismatch, trait-operation mismatch, operation-specific NotRequired misuse, generic header rejection, and table projection paths",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|span|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash/,
    "producer code must not use source text, spans, lexemes, display names, diagnostics, or module path as proof authority",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key)/,
    "producer must not import HIR, Resource IR, backend, proof store, artifact reader, serializer, preseed, decoded proof, payload reader, or canonical-key layers",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_source_identity_new|SelfhostMemoTraitTrustedSourceRegistry|signature_available\s+true|selfhost_memo_trait_aggregate_proof_to_record|SelfhostMemoTraitEvidenceRecord/,
    "producer must not construct trusted source identities, registries, source records, aggregate consumer records, or accepted MemoKey/MemoValue evidence records directly",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation evidence producer contract passed");

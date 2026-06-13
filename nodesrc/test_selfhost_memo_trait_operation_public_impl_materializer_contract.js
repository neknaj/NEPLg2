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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_public_impl_materializer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl materializer must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("operation kind は caller supplied field ではなく、trusted operation classifier が返す shape-bound evidence から採用します") &&
        source.includes("method body fact、Drop proof、operation evidence record、aggregate proof status を作りません"),
    "docs must place the materializer between typed public impl records and the candidate builder without producing proof artifacts",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string") &&
        source.includes("operation や HIR root を推測しません"),
    "docs must reject source-derived authority for operation or method root materialization",
);
assert.ok(
    source.includes("Drop record が現れた場合、classifier で shape を確認したうえで builder input に写し") &&
        source.includes("`DropOperationUnsupportedUntilResourceProof`"),
    "docs must route Drop through the existing fail-closed builder boundary until Resource proof is connected",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_public_impl_materializer/,
    "public impl materializer must remain facade-private until full operation orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_public_impl_materializer/,
    "checker-layer public impl materializer must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_impl_candidate_builder\" as *",
        "#import \"./memo_trait_operation_impl_table\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
    ],
    "materializer imports must go through classifier, candidate builder, impl table lookup helpers, and public impl header boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver)/,
    "materializer must not import Resource IR, backend, proof store, canonical-key, producer, purity, body-check, method-body, or Drop resolver layers directly",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationPublicImplMaterializerRecord:",
        "type_id %SelfhostTypeId",
        "module_fingerprint %i32",
        "declaration_ordinal %Option i32",
        "visibility %SelfhostModuleDeclarationVisibility",
        "impl_kind %SelfhostMemoTraitPublicImplHeaderKind",
        "target_type_shape_hash %Option i32",
        "trait_source %SelfhostMemoTraitOperationSourceIdentity",
        "trait_type_argument_count %i32",
        "trait_application_shape_hash %Option i32",
        "type_parameter_count %i32",
        "type_parameter_bound_count %i32",
        "method_body_root %Option SelfhostHirExprId",
        "fuel %i32",
    ],
    "materializer record must carry typed public impl header fields, classifier fields, method root, and fuel",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationPublicImplMaterializerErrorKind:",
        "RecordTableAllocFailed %StdErrorKind",
        "RecordPushFailed %StdErrorKind",
        "BuilderInputTableAllocFailed %StdErrorKind",
        "SourceReadFailed %i32",
        "ClassifierRejected %SelfhostMemoTraitOperationClassifierErrorKind",
        "BuilderInputPushRejected %SelfhostMemoTraitOperationImplCandidateBuilderErrorKind",
        "CandidateBuilderRejected %SelfhostMemoTraitOperationImplCandidateBuilderErrorKind",
    ],
    "materializer errors must distinguish setup, read, classifier, builder-input push, and candidate-builder failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_header_input record",
        "selfhost_memo_trait_operation_public_impl_materializer_trait_application_input record",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "Result::Ok classifier:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_new record.type_id classifier.operation impl_header trait_application record.target_type_shape_hash record.method_body_root record.fuel",
        "Result::Err classifier_error:",
        "ClassifierRejected classifier_error",
    ],
    "record materialization must derive operation from classifier evidence before constructing builder input",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result"),
    /record\.trait_source\.operation|SelfhostMemoTraitOperationEvidenceKind::Copy|SelfhostMemoTraitOperationEvidenceKind::Drop|SelfhostMemoTraitOperationEvidenceKind::Eq|SelfhostMemoTraitOperationEvidenceKind::Hash/,
    "record_to_builder_input_result must not directly trust operation kind from source identity or hard-code operation variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_builder_input_loop"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result record",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_push builder builder_input",
        "BuilderInputPushRejected push_error",
        "Result::Err e:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder",
        "Result::Err e",
        "Option::None:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder",
        "SourceReadFailed index",
    ],
    "builder_input_loop must clean up the temporary builder input table on classifier and read failures while relying on push boundary cleanup for push failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_builder_inputs_from_records_result source",
        "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result module &builder_inputs",
        "Result::Ok candidates:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "Result::Ok candidates",
        "Result::Err builder_error:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "CandidateBuilderRejected builder_error",
    ],
    "candidate table entry must close the temporary builder input owner after both builder success and builder rejection",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_operation_impl_table_push|SelfhostMemoTraitOperationEvidenceProducerInput|selfhost_memo_trait_operation_impl_candidate_producer_input_result|selfhost_memo_trait_operation_impl_record_for_type_operation_result|selfhost_memo_trait_operation_evidence_producer_input_new|selfhost_memo_trait_operation_evidence_producer_status_result|selfhost_memo_trait_operation_evidence_producer_record_result|SelfhostMemoTraitAggregateProofStatus|SelfhostMemoTraitOperationEvidenceRecord/,
    "materializer must not push candidates directly or produce producer input, evidence record, or aggregate proof status",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_operation_public_impl_materializer_stage0",
        "selfhost_memo_trait_operation_public_impl_materializer_accepted_len_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_accepted_operation_present_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_classifier_rejected_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_candidate_builder_rejected_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_drop_unsupported_result_eq",
    ],
    "materializer must expose a stage0 smoke API and typed assertion helpers for accepted, classifier rejection, builder rejection, and Drop unsupported paths",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_drop_unsupported_result_eq"),
    [
        "CandidateBuilderRejected builder_error:",
        "let builder_result %Result i32 SelfhostMemoTraitOperationImplCandidateBuilderErrorKind Result::Err builder_error",
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_unsupported_error_result_eq builder_result expected_index",
        "ClassifierRejected _classifier:",
        "false",
    ],
    "Drop unsupported helper must prove materializer-originated Drop records reach the existing builder Drop rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_stage0_with_shapes"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_accepted",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_untrusted",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_duplicate",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_drop_unsupported",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_summary_new accepted untrusted_rejected duplicate_rejected drop_unsupported",
    ],
    "stage0 must cover accepted records, classifier rejection, duplicate rejection, and Drop unsupported routing",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "materializer contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait operation public impl materializer contract ok");

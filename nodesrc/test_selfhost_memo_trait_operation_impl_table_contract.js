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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_impl_table.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_impl_table",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "operation impl table must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("public surface normalizer が作る trait impl 候補列") &&
        source.includes("operation classifier / operation evidence producer の間に置く探索境界"),
    "docs must place the table between public surface impl candidates and operation classifier / producer",
);
assert.ok(
    source.includes("record order による first-wins にせず `CandidateDuplicate`") &&
        source.includes("method body purity、Drop なし proof、generic impl binder、trait coherence、full public surface orchestration を推測しません"),
    "docs must reject first-wins duplicate handling and keep method/drop/generic/full-orchestration outside this slice",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path") &&
        source.includes("HIR、Resource IR、backend artifact、proof store"),
    "docs must exclude source/display/module path/HIR/Resource/backend/proof-store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_impl_table/,
    "operation impl table must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_impl_table/,
    "checker-layer impl table must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_evidence_producer\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
    ],
    "impl table must import classifier, producer, and public impl header boundaries in checker-layer direction",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key)/,
    "impl table must not import HIR, Resource IR, backend, proof store, artifact reader, serializer, preseed, decoded proof, payload reader, or canonical-key layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationImplCandidate:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "impl_header %SelfhostMemoTraitPublicImplHeaderInput",
        "trait_application %SelfhostMemoTraitOperationTraitApplicationInput",
        "resolved_type_shape_hash %Option i32",
        "method_body %SelfhostMemoTraitOperationMethodBodyEvidence",
        "drop_evidence %SelfhostMemoTraitOperationDropEvidence",
    ],
    "candidate payload must carry type id, operation kind, typed impl header, typed trait application, target shape evidence, method evidence, and drop evidence",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationImplTableErrorKind:",
        "CandidateMissing",
        "CandidateDuplicate",
        "CandidateReadFailed %i32",
        "CandidatePushFailed %StdErrorKind",
        "ClassifierRejected %SelfhostMemoTraitOperationClassifierErrorKind",
        "ProducerRejected %SelfhostMemoTraitOperationEvidenceProducerErrorKind",
    ],
    "impl table errors must distinguish missing, duplicate, push, classifier, and producer failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_find_loop"),
    [
        "selfhost_memo_trait_operation_impl_candidate_key_matches candidate type_id operation",
        "Option::Some _existing:",
        "CandidateDuplicate",
        "Option::None:",
        "selfhost_memo_trait_operation_impl_find_loop table type_id operation add idx 1 some candidate",
        "Option::None:",
        "CandidateReadFailed idx",
    ],
    "impl table lookup must reject duplicate matching candidates instead of first-wins and treat impossible Vec read misses as typed read failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_table_push"),
    [
        "v::push records candidate",
        "Result::Ok next_records:",
        "Result::Ok SelfhostMemoTraitOperationImplTable next_records",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
        "CandidatePushFailed error",
    ],
    "table push must recover and free the owner Vec returned by a failed push",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_producer_input_result"),
    [
        "selfhost_memo_trait_operation_classifier_evidence_result candidate.trait_application",
        "selfhost_memo_trait_operation_evidence_producer_input_new",
        "selfhost_memo_trait_operation_evidence_producer_status_result input",
        "Result::Ok input",
        "ProducerRejected producer_error",
        "ClassifierRejected classifier_error",
    ],
    "candidate conversion must create classifier evidence and pass producer validation before returning producer input",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_record_for_type_operation_result"),
    [
        "selfhost_memo_trait_operation_impl_producer_input_for_type_operation_result table type_id operation",
        "selfhost_memo_trait_operation_evidence_producer_record_result input",
        "Result::Ok record",
        "ProducerRejected producer_error",
    ],
    "record entry must reuse the producer record boundary and not construct operation records directly",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_table_error_kind_eq"),
    [
        "CandidateMissing:",
        "CandidateDuplicate:",
        "CandidateReadFailed a_idx:",
        "eq a_idx b_idx",
        "CandidatePushFailed a_push:",
        "ClassifierRejected a_classifier:",
        "selfhost_memo_trait_operation_classifier_error_kind_eq a_classifier b_classifier",
        "ProducerRejected a_producer:",
        "selfhost_memo_trait_operation_evidence_producer_error_kind_eq a_producer b_producer",
    ],
    "error equality must be exhaustive and compare nested classifier / producer payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_operation_impl_table_stage0",
        "selfhost_memo_trait_operation_trusted_source_registry_current_result",
        "selfhost_memo_trait_operation_trait_application_shape_hash_result registry.copy_source 0",
        "selfhost_memo_trait_operation_impl_table_stage0_push_accepted",
    ],
    "stage0 must obtain current registry, derive Copy trait application shape, and enter the accepted stage0 table path",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_table_stage0_after_table"),
    [
        "selfhost_memo_trait_operation_impl_record_for_type_operation_result &table type_id SelfhostMemoTraitOperationEvidenceKind::Copy",
        "missing_rejected",
        "selfhost_memo_trait_operation_impl_table_stage0_duplicate",
        "untrusted_source",
        "classifier_rejected",
        "selfhost_memo_trait_operation_impl_table_stage0_generic_impl_header",
        "producer_rejected",
        "target_mismatch_rejected",
    ],
    "stage0 helper must exercise accepted lookup, missing lookup, duplicate rejection, untrusted classifier rejection, and generic producer rejection",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|\bspan\b|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash/,
    "impl table code must not use source text, spans, lexemes, display names, diagnostics, module paths, or flattened payload hash as authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "impl table policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation impl table contract passed");

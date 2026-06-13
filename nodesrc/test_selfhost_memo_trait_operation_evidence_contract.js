#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    TY_ROOT_REEXPORT_FILES,
    TY_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_evidence.nepl";
const operationProofPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl";
const operationSolverPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_solver.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_operation_evidence.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_operation_evidence.nepl",
);

function assertEvidenceOrder(list, label) {
    const operationProofIndex = list.indexOf(operationProofPath);
    const operationEvidenceIndex = list.indexOf(relPath);
    const operationSolverIndex = list.indexOf(operationSolverPath);
    assert.ok(
        operationProofIndex >= 0 && operationEvidenceIndex >= 0 && operationSolverIndex >= 0,
        `${label} must include operation proof, operation evidence, and operation solver files`,
    );
    assert.ok(
        operationProofIndex < operationEvidenceIndex && operationEvidenceIndex < operationSolverIndex,
        `${label} must keep operation evidence after proof transport and before operation solver`,
    );
}

assertEvidenceOrder(TY_ROOT_REEXPORT_FILES, "ty root re-export order");
assertEvidenceOrder(TY_SPLIT_FILES, "ty split order");
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_operation_evidence" as \*$/m,
    "ty facade must re-export the memo trait operation evidence split module",
);
assert.match(
    facade,
    /pub #import "\.\/ty\/memo_trait_operation_proof" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_evidence" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_solver" as \*/,
    "ty facade must keep operation proof before operation evidence before operation solver",
);

assert.match(
    source,
    /# ty\/memo_trait_operation_evidence[\s\S]*\[目的\/もくてき\]:[\s\S]*operation evidence[\s\S]*\[契約\/けいやく\]:[\s\S]*RecordMissing[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "operation evidence documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source text、span、path、display name、diagnostic text、lexeme は proof authority にしません/,
    "operation evidence docs must explicitly reject source text, spans, paths, display names, diagnostics, and lexemes as proof authority",
);
assert.match(
    source,
    /HIR、Resource IR、backend、proof store、artifact reader、serializer、canonical key codec へ依存しません/,
    "operation evidence docs must keep HIR, Resource IR, backend, proof stores, artifacts, and codecs out of this checkpoint",
);
assert.match(
    source,
    /`Drop` operation の `Proven` は「上流が Drop なし、または pure Drop proof を確認済み」という意味です[\s\S]*Drop なし proof を推測で作りません/,
    "operation evidence docs must distinguish transported Drop evidence from inferred Drop absence",
);

assert.match(
    source,
    /pub enum SelfhostMemoTraitOperationEvidenceKind:[\s\S]*Copy[\s\S]*Drop[\s\S]*Eq[\s\S]*Hash/,
    "operation evidence kind must enumerate Copy, Drop, Eq, and Hash",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationEvidenceRecord:[\s\S]*type_id %SelfhostTypeId[\s\S]*operation %SelfhostMemoTraitOperationEvidenceKind[\s\S]*status %SelfhostMemoTraitAggregateProofStatus/,
    "operation evidence record must store TypeId, operation kind, and aggregate proof status",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationEvidenceTable:[\s\S]*records %Vec SelfhostMemoTraitOperationEvidenceRecord/,
    "operation evidence table must own typed evidence records",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitOperationEvidenceErrorKind:[\s\S]*RecordMissing[\s\S]*DuplicateRecord[\s\S]*RecordPushFailed %StdErrorKind/,
    "operation evidence errors must preserve missing, duplicate, and push failure cases",
);

assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_find_loop[\s\S]*selfhost_type_id_eq record\.type_id type_id[\s\S]*selfhost_memo_trait_operation_evidence_kind_eq record\.operation operation[\s\S]*Option::Some _existing:[\s\S]*DuplicateRecord[\s\S]*Option::None:[\s\S]*some record/,
    "operation evidence lookup must key by TypeId plus operation kind and reject duplicates instead of using first-wins",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_status_for_type_operation_or_missing_result[\s\S]*RecordMissing:[\s\S]*Result::Ok SelfhostMemoTraitAggregateProofStatus::Missing[\s\S]*DuplicateRecord:[\s\S]*Result::Err SelfhostMemoTraitOperationEvidenceErrorKind::DuplicateRecord/,
    "operation evidence missing conversion must only turn missing evidence into Missing status and must preserve duplicate errors",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_record_for_type_or_missing_result[\s\S]*EvidenceKind::Copy[\s\S]*EvidenceKind::Drop[\s\S]*EvidenceKind::Eq[\s\S]*EvidenceKind::Hash[\s\S]*selfhost_memo_trait_operation_proof_record_new type_id copy_status drop_status eq_status hash_status/,
    "operation evidence table must project Copy, Drop, Eq, and Hash statuses into an operation proof record",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_status_code[\s\S]*Proven:[\s\S]*0[\s\S]*Missing:[\s\S]*1[\s\S]*Unknown:[\s\S]*2[\s\S]*Impure:[\s\S]*3/,
    "operation evidence status equality must preserve the solver status ordering",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*selfhost_memo_trait_operation_evidence_error_kind_eq[\s\S]*RecordMissing:[\s\S]*DuplicateRecord:[\s\S]*RecordPushFailed a_kind:[\s\S]*selfhost_memo_trait_operation_evidence_std_error_kind_eq a_kind b_kind/,
    "operation evidence error equality must avoid wildcard arms and compare nested StdErrorKind payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_stage0[\s\S]*selfhost_type_id_new 50[\s\S]*selfhost_type_id_new 51[\s\S]*selfhost_memo_trait_operation_evidence_stage0_push_copy/,
    "operation evidence stage0 must use stable typed ids and public table helpers",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_evidence_stage0_duplicate[\s\S]*同じ TypeId と operation kind を2件入れ[\s\S]*status_for_type_operation_result[\s\S]*EvidenceKind::Copy/,
    "operation evidence stage0 must cover duplicate evidence rejection",
);

assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_(?:proof_store|proof_reader|proof_payload_reader|proof_serializer|proof_preseed|proof_stable_map|proof_artifact|proof_index|proof_decoded|artifact_word_codec|canonical_key|canonical_key_payload|canonical_key_payload_codec)"/,
    "operation evidence must not depend on proof store, artifact, canonical key, reader, serializer, index, or preseed modules",
);
assert.doesNotMatch(
    codeOnly,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "operation evidence must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostMemoTraitEvidenceRecord|selfhost_memo_trait_aggregate_proof_(?:new|to_record)|selfhost_memo_trait_recursive_producer_record_result/,
    "operation evidence must not construct accepted evidence records or producer aggregate proofs",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "operation evidence code must not use source text, spans, paths, display names, diagnostics, or lexemes as proof authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "operation evidence policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation evidence contract passed");

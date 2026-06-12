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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_operation_proof.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_operation_proof.nepl",
);
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_operation_proof" as \*$/m,
    "ty facade must re-export the memo trait operation proof split module",
);
assert.match(
    source,
    /# ty\/memo_trait_operation_proof[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "operation proof module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /session-local `SelfhostTypeId` を key にする一時 table[\s\S]*永続 artifact、`\.neplmeta`、`\.neplproof`、cross-arena cache の authority にしてはいけません/,
    "operation proof docs must state that the table is session-local and not a persistent artifact authority",
);
assert.match(
    source,
    /table に record が無い型は、Copy \/ Drop \/ Eq \/ Hash がすべて `Missing`[\s\S]*欠落を暗黙に `Proven` へ補完しません/,
    "operation proof docs must state that missing records become Missing statuses and never implicit Proven evidence",
);
assert.match(
    source,
    /trait impl table、method body purity、Drop なし proof、recursive field traversal は後続 slice/,
    "operation proof docs must keep the real trait solver and recursive aggregate traversal out of this checkpoint",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_producer" as \*$/m,
    "operation proof must reuse the existing producer gate instead of rebuilding producer rejection taxonomy",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "operation proof must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_(?:proof|artifact|canonical_key|canonical_key_payload|canonical_key_payload_codec|artifact_word_codec|proof_store|proof_reader|proof_serializer|proof_preseed)/m,
    "operation proof must not depend on proof store, artifact, canonical key, reader, serializer, preseed, or codec modules",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationProofRecord:[\s\S]*type_id %SelfhostTypeId[\s\S]*copy_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*drop_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*eq_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*hash_proof %SelfhostMemoTraitAggregateProofStatus/,
    "operation proof record must carry TypeId and the four typed aggregate proof statuses",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationProofTable:[\s\S]*records %Vec SelfhostMemoTraitOperationProofRecord/,
    "operation proof table must own a typed record vector",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitOperationProofErrorKind:[\s\S]*RecordMissing[\s\S]*DuplicateRecord[\s\S]*MissingTypeRecord[\s\S]*RecordPushFailed %StdErrorKind/,
    "operation proof errors must preserve missing record, duplicate record, missing type, and push storage failures as typed variants",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitOperationProofStage0Summary:[\s\S]*produced_record[\s\S]*missing_record_rejected[\s\S]*copy_missing_rejected[\s\S]*eq_impure_rejected[\s\S]*hash_unknown_rejected[\s\S]*duplicate_rejected[\s\S]*fake_type_rejected[\s\S]*direct_missing/,
    "stage0 summary must expose accepted, missing, copy-missing, eq-impure, hash-unknown, duplicate, fake-type, and direct missing lookup results",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_record_all_missing[\s\S]*SelfhostMemoTraitAggregateProofStatus::Missing[\s\S]*SelfhostMemoTraitAggregateProofStatus::Missing[\s\S]*SelfhostMemoTraitAggregateProofStatus::Missing[\s\S]*SelfhostMemoTraitAggregateProofStatus::Missing/,
    "all-missing record constructor must set every operation status to Missing",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_table_push[\s\S]*let error %StdErrorKind field::get e "error"[\s\S]*v::free v::vec_push_error_vec e[\s\S]*RecordPushFailed error/,
    "table push must preserve the StdErrorKind and close the Vec owner returned by failed push",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_find_loop[\s\S]*selfhost_type_id_eq record\.type_id type_id[\s\S]*Option::Some _existing:[\s\S]*DuplicateRecord[\s\S]*Option::None:[\s\S]*selfhost_memo_trait_operation_proof_find_loop table type_id add idx 1 some record/,
    "operation proof lookup must compare typed TypeId values and reject duplicate records instead of using first-wins",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_record_for_type_or_missing_result[\s\S]*RecordMissing:[\s\S]*Result::Ok selfhost_memo_trait_operation_proof_record_all_missing type_id[\s\S]*DuplicateRecord:[\s\S]*Result::Err SelfhostMemoTraitOperationProofErrorKind::DuplicateRecord/,
    "operation proof aggregate path must convert only missing table records into Missing status records and preserve duplicate errors",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*selfhost_memo_trait_operation_proof_error_kind_eq[\s\S]*DuplicateRecord:[\s\S]*true[\s\S]*RecordPushFailed a_kind:[\s\S]*RecordPushFailed b_kind:[\s\S]*selfhost_memo_trait_operation_proof_std_error_kind_eq a_kind b_kind[\s\S]*selfhost_memo_trait_operation_proof_std_error_kind_code[\s\S]*StdErrorKind::Failure:[\s\S]*StdErrorKind::Other:/,
    "operation proof error equality must avoid wildcard arms, include DuplicateRecord, and compare RecordPushFailed payload variants",
);
assert.match(
    source,
    /selfhost_memo_trait_aggregate_proof_from_operation_table_result[\s\S]*selfhost_type_arena_get_record types type_id[\s\S]*Option::Some _record:[\s\S]*selfhost_memo_trait_operation_proof_record_for_type_or_missing_result table type_id[\s\S]*Result::Ok operation_record:[\s\S]*selfhost_memo_trait_aggregate_proof_new type_id fields operation_record\.copy_proof operation_record\.drop_proof operation_record\.eq_proof operation_record\.hash_proof hazard key_result value_result[\s\S]*Result::Err e:[\s\S]*Result::Err e[\s\S]*Option::None:[\s\S]*MissingTypeRecord/,
    "aggregate proof construction must verify TypeId existence, use operation statuses, and reject missing type records",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_stage0_record_to_producer_result[\s\S]*selfhost_memo_trait_aggregate_proof_from_operation_table_result[\s\S]*selfhost_memo_trait_aggregate_proof_to_record arena proof/,
    "stage0 must route operation table results through the public aggregate proof constructor and producer gate",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_stage0_push_duplicate_first[\s\S]*record_all_proven duplicate_id[\s\S]*selfhost_memo_trait_operation_proof_stage0_push_duplicate_second arena table_next proven_id/,
    "stage0 duplicate first helper must add an all-proven duplicate record and continue to the second helper",
);
assert.match(
    source,
    /selfhost_memo_trait_operation_proof_stage0_push_duplicate_second[\s\S]*record_all_missing duplicate_id[\s\S]*selfhost_memo_trait_operation_proof_stage0_after_table arena table_next/,
    "stage0 duplicate second helper must add a second record for the same TypeId and continue to summary collection",
);
assert.match(
    source,
    /let duplicate_rejected %Result SelfhostMemoTraitAggregateProof SelfhostMemoTraitOperationProofErrorKind selfhost_memo_trait_aggregate_proof_from_operation_table_result &arena &table duplicate_id/,
    "stage0 must route duplicate TypeId records through the aggregate proof constructor",
);
assert.match(
    source,
    /let fake_type_id %SelfhostTypeId selfhost_type_id_new 9999[\s\S]*selfhost_memo_trait_aggregate_proof_from_operation_table_result &arena &table fake_type_id/,
    "stage0 must prove that an all-proven operation record cannot create proof for a TypeId outside the arena",
);
assert.match(
    source,
    /selfhost_memo_trait_evidence_produce_result_is_accept summary\.produced_record[\s\S]*CopyProofMissing[\s\S]*EqProofImpure[\s\S]*HashProofUnknown[\s\S]*DuplicateRecord[\s\S]*MissingTypeRecord/,
    "doctest must check the accepted path, operation proof rejections, duplicate rejection, and fake TypeId rejection",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "operation proof code must not use source text, spans, paths, display names, diagnostics, or lexemes as proof authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "operation proof policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation proof contract passed");

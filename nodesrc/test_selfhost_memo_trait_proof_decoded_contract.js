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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const tySources = readRepoFile(repoRoot, "nodesrc/selfhost_ty_sources.js");
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
function codeSliceBetween(startNeedle, endNeedle) {
    const start = source.indexOf(startNeedle);
    const end = source.indexOf(endNeedle);
    assert.notEqual(start, -1, `missing source slice start: ${startNeedle}`);
    assert.notEqual(end, -1, `missing source slice end: ${endNeedle}`);
    assert.ok(start < end, `source slice must be ordered: ${startNeedle}`);
    return source
        .slice(start, end)
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

const constructorImplementation = codeSliceBetween(
    "pub fn selfhost_memo_trait_neplproof_decoded_artifact_from_records",
    "//: selfhost_memo_trait_neplproof_decoded_artifact_lookup_result",
);
const persistedConstructorImplementation = codeSliceBetween(
    "pub fn selfhost_memo_trait_neplproof_decoded_artifact_from_record_and_index_tables",
    "//: selfhost_memo_trait_neplproof_decoded_artifact_lookup_result",
);
const lookupImplementation = codeSliceBetween(
    "pub fn selfhost_memo_trait_neplproof_decoded_artifact_lookup_result",
    "//: selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result",
);
const candidateImplementation = codeSliceBetween(
    "pub fn selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result",
    "//: selfhost_memo_trait_neplproof_decoded_artifact_record_at_result",
);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_decoded" as \*$/m,
    "ty facade must re-export the decoded .neplproof artifact owner module",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost TY root re-export source list must include the decoded artifact owner module",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost TY split source list must include the decoded artifact owner module",
);
assert.match(
    tySources,
    /memo_trait_proof_index\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_decoded\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_payload_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_serializer\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_stable_map\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_preseed\.nepl"/,
    "decoded artifact owner must sit after the sidecar index producer and before record reader / payload reader / serializer / stable map / proof preseed in the TY source order",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "decoded artifact owner must reuse artifact schema validation",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_index" as \*$/m,
    "decoded artifact owner must reuse sorted sidecar index validation and lookup",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_preseed" as \*$/m,
    "decoded artifact owner must stay before proof-store preseed acceptance",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "decoded artifact owner must not depend on proof-store implementation details",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_source" as \*$/m,
    "decoded artifact owner must not depend on trusted-source construction details",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "decoded artifact owner must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_decoded[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "decoded artifact owner documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /reader \/ serializer \/ preseed の前段[\s\S]*records と indexes は同じ header に属/,
    "decoded artifact documentation must define the owner boundary before serializer and preseed acceptance",
);
assert.match(
    source,
    /lower-bound binary search[\s\S]*O\(n \* m \+ m \* m \+ log m \+ c\)[\s\S]*candidate_record_at_result[\s\S]*range \/ offset \/ target fingerprint \/ index entry と record の対応を O\(1\) で検査/,
    "decoded artifact documentation must describe the sorted-index lookup complexity and candidate record accessor cost",
);
assert.match(
    source,
    /失敗時には、入力 `records` owner と、構築済み `indexes` owner をこの module が閉じます/,
    "decoded artifact documentation must state failure-path ownership cleanup",
);
assert.match(
    source,
    /source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`[\s\S]*authority ではありません/,
    "decoded artifact documentation must exclude source text, spans, paths, diagnostics, lexemes, and session-local ids from authority",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedArtifact:[\s\S]*header %SelfhostMemoTraitNeplProofHeader[\s\S]*records %Vec SelfhostMemoTraitNeplProofRecord[\s\S]*indexes %Vec SelfhostMemoTraitNeplProofIndexEntry/,
    "decoded artifact owner must own header, decoded records, and sorted indexes together",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedCandidateRecord:[\s\S]*index_entry %SelfhostMemoTraitNeplProofIndexEntry[\s\S]*record %SelfhostMemoTraitNeplProofRecord/,
    "decoded candidate record result must pair the candidate index entry with the copied decoded record",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofDecodedCandidateErrorKind:[\s\S]*CandidateRangeInvalid[\s\S]*CandidateOffsetOutOfRange[\s\S]*CandidateIndexEntryMissing[\s\S]*CandidateRecordEntryMissing[\s\S]*CandidateTargetFingerprintMismatch[\s\S]*CandidateRecordFingerprintMismatch[\s\S]*CandidateRecordHashMismatch[\s\S]*CandidateRecordValidationUnexpected %SelfhostMemoTraitNeplProofArtifactErrorKind/,
    "decoded candidate access errors must classify invalid range, invalid offset, projection-local missing entries, target mismatch, record fingerprint mismatch, record hash mismatch, and unexpected validator errors as typed variants",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofDecodedArtifactErrorKind:[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*IndexBuildInvalid %SelfhostMemoTraitNeplProofIndexProducerErrorKind[\s\S]*TableValidationInvalid %SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*SortedIndexInvalid %SelfhostMemoTraitNeplProofSortedIndexErrorKind[\s\S]*LookupInvalid %SelfhostMemoTraitNeplProofSortedIndexErrorKind[\s\S]*CandidateMissing[\s\S]*CandidateAccessInvalid %SelfhostMemoTraitNeplProofDecodedCandidateErrorKind[\s\S]*RecordEntryMissing[\s\S]*IndexEntryMissing/,
    "decoded artifact owner errors must preserve typed nested error payloads and split valid candidate misses, candidate access failure, and lookup corruption",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_artifact_free[\s\S]*%impure fn SelfhostMemoTraitNeplProofDecodedArtifact unit[\s\S]*v::free field::get artifact "records"[\s\S]*v::free field::get artifact "indexes"/,
    "decoded artifact free boundary must consume and close both Vec owners",
);
assert.doesNotMatch(
    source,
    /impl Copy for SelfhostMemoTraitNeplProofDecodedArtifact:\n|impl Clone for SelfhostMemoTraitNeplProofDecodedArtifact:\n/,
    "decoded artifact owner must not be Copy or Clone because it owns Vec storage",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_artifact_validate_result[\s\S]*field::get_ref artifact "header"[\s\S]*field::get_ref artifact "records"[\s\S]*field::get_ref artifact "indexes"[\s\S]*selfhost_memo_trait_neplproof_header_result[\s\S]*selfhost_memo_trait_neplproof_index_table_result[\s\S]*selfhost_memo_trait_neplproof_sorted_index_order_result/,
    "decoded artifact validate boundary must borrow owner fields and re-run header, table, and sorted-order validation",
);
assert.match(
    constructorImplementation,
    /selfhost_memo_trait_neplproof_sorted_index_build_result &records[\s\S]*selfhost_memo_trait_neplproof_header_result[\s\S]*selfhost_memo_trait_neplproof_index_table_result header &records &indexes[\s\S]*selfhost_memo_trait_neplproof_sorted_index_order_result &indexes/,
    "decoded artifact constructor must build the sorted sidecar index and revalidate header, decoded table, and sorted order",
);
assert.match(
    constructorImplementation,
    /Result::Err kind:[\s\S]*v::free records[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::IndexBuildInvalid kind/,
    "decoded artifact constructor must free input records when index construction fails",
);
assert.match(
    constructorImplementation,
    /Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::SortedIndexInvalid kind[\s\S]*Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::TableValidationInvalid kind[\s\S]*Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::HeaderInvalid kind/,
    "decoded artifact constructor must free both records and indexes on post-build validation failures",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_artifact_from_record_and_index_tables[\s\S]*from_records[\s\S]*index table を lookup source として読み[\s\S]*永続 artifact の探索範囲を再構築 cost から切り離します[\s\S]*index hit は proof acceptance authority ではありません[\s\S]*pub fn selfhost_memo_trait_neplproof_decoded_artifact_from_record_and_index_tables/,
    "decoded artifact persisted constructor docs must state why serialized indexes are lookup sources but not proof acceptance authority",
);
assert.match(
    persistedConstructorImplementation,
    /selfhost_memo_trait_neplproof_header_result header\.artifact_schema_version header\.canonical_payload_schema_version header\.policy_schema_version header\.record_count header\.index_count[\s\S]*selfhost_memo_trait_neplproof_index_table_result checked_header &records &indexes[\s\S]*selfhost_memo_trait_neplproof_sorted_index_order_result &indexes[\s\S]*Result::Ok selfhost_memo_trait_neplproof_decoded_artifact_new checked_header records indexes/,
    "decoded artifact persisted constructor must validate header, record/index table relation, and sorted order before returning the owner",
);
assert.match(
    persistedConstructorImplementation,
    /Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::SortedIndexInvalid kind[\s\S]*Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::TableValidationInvalid kind[\s\S]*Result::Err kind:[\s\S]*v::free records[\s\S]*v::free indexes[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::HeaderInvalid kind/,
    "decoded artifact persisted constructor must close record and index owners on every validation failure",
);
assert.match(
    lookupImplementation,
    /selfhost_memo_trait_neplproof_sorted_index_lookup_result header records indexes target[\s\S]*SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::LookupInvalid[\s\S]*SelfhostMemoTraitNeplProofSortedIndexErrorKind::CandidateMissing:[\s\S]*Result::Err SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::CandidateMissing/,
    "decoded artifact lookup must delegate to the sorted index public lookup boundary while splitting valid candidate miss from typed lookup corruption",
);
assert.match(
    candidateImplementation,
    /selfhost_memo_trait_neplproof_decoded_artifact_validate_result artifact[\s\S]*lt range\.start_index 0[\s\S]*le range\.candidate_count 0[\s\S]*CandidateRangeInvalid[\s\S]*lt candidate_offset 0[\s\S]*ge candidate_offset range\.candidate_count[\s\S]*CandidateOffsetOutOfRange/,
    "candidate record accessor must validate artifact invariants and reject invalid range or offset before reading vectors",
);
assert.match(
    candidateImplementation,
    /let index_slot %i32 add range\.start_index candidate_offset[\s\S]*lt index_slot range\.start_index[\s\S]*v::get indexes index_slot[\s\S]*CandidateIndexEntryMissing/,
    "candidate record accessor must derive the index slot defensively and classify projection-local missing index entries",
);
assert.match(
    candidateImplementation,
    /not selfhost_memo_trait_canonical_type_fingerprint_eq entry\.canonical_fingerprint target[\s\S]*CandidateTargetFingerprintMismatch[\s\S]*v::get records entry\.record_ordinal[\s\S]*CandidateRecordEntryMissing/,
    "candidate record accessor must verify the target fingerprint before reading the pointed record and classify projection-local missing records",
);
assert.match(
    candidateImplementation,
    /selfhost_memo_trait_neplproof_index_entry_matches_record_result entry record[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_record_new entry record[\s\S]*CandidateRecordFingerprintMismatch[\s\S]*CandidateRecordHashMismatch[\s\S]*CandidateRecordValidationUnexpected kind/,
    "candidate record accessor must re-check index-entry to record consistency and split fingerprint/hash mismatch from unexpected validator payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_artifact_record_at_result[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_validate_result artifact[\s\S]*v::get records record_ordinal[\s\S]*RecordEntryMissing/,
    "record accessor must validate the artifact and classify defensive record slot absence",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_artifact_index_at_result[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_validate_result artifact[\s\S]*v::get indexes index[\s\S]*IndexEntryMissing/,
    "index accessor must validate the artifact and classify defensive index slot absence",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_candidate_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq[\s\S]*pub fn selfhost_memo_trait_neplproof_decoded_artifact_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_index_producer_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_index_validation_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_sorted_index_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_error_kind_eq/,
    "decoded artifact error equality must compare all nested typed payloads, including candidate access errors, instead of stringifying errors",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_artifact_stage0[\s\S]*selfhost_memo_trait_neplproof_artifact_stage0[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_stage0_records_one[\s\S]*accepted_lookup[\s\S]*accepted_candidate_record[\s\S]*candidate_range_error[\s\S]*candidate_offset_error[\s\S]*candidate_target_error[\s\S]*accepted_collision_candidate_record[\s\S]*accepted_record[\s\S]*accepted_index[\s\S]*missing_lookup[\s\S]*invalid_record_result/,
    "stage0 smoke must reuse the artifact schema accepted record and cover lookup, candidate record access, invalid range, offset rejection, target mismatch, collision offset, record access, index access, missing candidate, and invalid-record rejection",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostCanonicalTypeKeyId|SelfhostTypeId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "decoded artifact owner code must not store session-local ids, proof-store records, stable identities, or store sidecar index entries",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "decoded artifact owner code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    constructorImplementation,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)|selfhost_memo_trait_neplproof_decoded_record_batch_append/,
    "decoded artifact constructor must not call proof-store, preseed, or decoded-batch append APIs directly",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|return Ok immediately after fingerprint|index hit is authority|proof acceptance by index|stable index only|record payload hash only/,
    "decoded artifact owner must not document or implement fingerprint-only or index-only proof acceptance",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "decoded artifact owner must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof decoded contract passed");

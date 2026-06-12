#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_artifact" as \*$/m,
    "ty facade must re-export the memo trait .neplproof artifact schema module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key" as \*$/m,
    "proof artifact schema must use typed canonical fingerprint payloads",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_policy" as \*$/m,
    "proof artifact schema must use typed solver policy payloads",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "proof artifact schema may reuse stored proof payload types from the proof store boundary",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "proof artifact schema must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_artifact[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "proof artifact schema module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /store-local `SelfhostCanonicalTypeKeyId`[\s\S]*永続 artifact へ出さず[\s\S]*serialized proof record が保持してよい小さい stable payload/,
    "proof artifact documentation must explicitly keep store-local canonical ids out of serialized .neplproof payloads",
);
assert.match(
    source,
    /source text、source span、path suffix、display name、diagnostic text、lexeme[\s\S]*accepted authority に入りません/,
    "proof artifact documentation must exclude source text, spans, paths, display names, diagnostics, and lexemes from accepted authority",
);
assert.match(
    source,
    /artifact index table は record ごとに 1 つの sidecar entry[\s\S]*proof store へ投入せず fail-closed/,
    "proof artifact documentation must require decoded index table coverage before proof store preseed",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofHeader:[\s\S]*artifact_schema_version %i32[\s\S]*canonical_payload_schema_version %i32[\s\S]*policy_schema_version %i32[\s\S]*record_count %i32[\s\S]*index_count %i32/,
    "proof artifact header must carry typed schema versions and bounded record/index counts",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofRecordKey:[\s\S]*canonical_payload_schema_version %i32[\s\S]*canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*canonical_payload_hash %i32[\s\S]*policy %SelfhostMemoTraitProofStorePolicy/,
    "serialized proof record key must use canonical fingerprint, canonical payload hash, and typed policy",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofRecord:[\s\S]*key %SelfhostMemoTraitNeplProofRecordKey[\s\S]*proof_kind %SelfhostMemoTraitStoredProofKind[\s\S]*proof %SelfhostMemoTraitStoredAggregateProof[\s\S]*record_payload_hash %i32/,
    "serialized proof record must keep proof kind, stored proof payload, and record payload hash together",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofIndexEntry:[\s\S]*canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*record_ordinal %i32[\s\S]*record_payload_hash %i32/,
    "serialized proof index entry must be a narrowing hint with fingerprint, ordinal, and payload hash only",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofArtifactErrorKind:[\s\S]*ArtifactSchemaMismatch[\s\S]*CanonicalKeySchemaMismatch[\s\S]*PolicySchemaMismatch[\s\S]*RecordCountNegative[\s\S]*IndexCountNegative[\s\S]*CanonicalPayloadHashPlaceholder[\s\S]*RecordPayloadHashPlaceholder[\s\S]*RecordIndexNegative[\s\S]*RecordIndexOutOfRange[\s\S]*IndexFingerprintMismatch[\s\S]*IndexRecordHashMismatch/,
    "proof artifact schema errors must be typed enum variants for schema, placeholder, and index invariant failures",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofIndexValidationErrorKind:[\s\S]*RecordCountMismatch[\s\S]*IndexCountMismatch[\s\S]*RecordEntryMissing[\s\S]*IndexEntryMissing[\s\S]*RecordInvalid[\s\S]*IndexEntryInvalid[\s\S]*IndexRecordMismatch[\s\S]*IndexRecordOrdinalDuplicate[\s\S]*IndexRecordOrdinalMissing/,
    "decoded proof index table validation must use typed enum variants for count, entry, mismatch, duplicate, and missing coverage failures",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofIndexValidationStage0Summary:[\s\S]*accepted %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*record_count_mismatch %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*index_count_mismatch %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*record_invalid %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*index_entry_invalid %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*index_record_mismatch %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*index_record_ordinal_duplicate %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*index_record_ordinal_missing %Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind/,
    "decoded proof index table stage0 must carry representative aggregate failure paths as typed Result values",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_header_result[\s\S]*Result SelfhostMemoTraitNeplProofHeader SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*artifact_schema_version selfhost_memo_trait_neplproof_artifact_schema_version[\s\S]*canonical_payload_schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version[\s\S]*policy_schema_version selfhost_memo_trait_neplproof_policy_schema_version[\s\S]*lt record_count 0[\s\S]*lt index_count 0/,
    "header validation must reject schema mismatches and negative counts with typed Result errors",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_key_result[\s\S]*Result SelfhostMemoTraitNeplProofRecordKey SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*canonical_payload_schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version[\s\S]*canonical_fingerprint\.schema_version canonical_payload_schema_version[\s\S]*eq canonical_payload_hash 0[\s\S]*policy\.rules\.schema_version selfhost_memo_trait_neplproof_policy_schema_version/,
    "record key validation must reject canonical schema mismatch, placeholder canonical payload hash, and stale policy schema",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_key_from_parts_result[\s\S]*SelfhostMemoTraitCanonicalTypeFingerprint canonical_fingerprint_schema_version canonical_fingerprint_root_hash[\s\S]*selfhost_memo_trait_neplproof_record_key_result canonical_fingerprint canonical_payload_schema_version canonical_payload_hash policy/,
    "artifact schema must expose a small serialized-parts record-key boundary for binary readers without making the reader import the full canonical-key producer",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_result[\s\S]*eq record_payload_hash 0[\s\S]*RecordPayloadHashPlaceholder[\s\S]*SelfhostMemoTraitNeplProofRecord/,
    "record validation must reject placeholder record payload hashes before accepting proof payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_index_entry_result[\s\S]*canonical_fingerprint\.schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version[\s\S]*lt record_ordinal 0[\s\S]*ge record_ordinal record_count[\s\S]*eq record_payload_hash 0/,
    "index validation must check canonical schema, ordinal lower/upper bounds, and payload hash",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_index_entry_matches_record_result[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq entry\.canonical_fingerprint record\.key\.canonical_fingerprint[\s\S]*eq entry\.record_payload_hash record\.record_payload_hash/,
    "index-to-record validation must require fingerprint and record payload hash equality",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_index_table_result[\s\S]*SelfhostMemoTraitNeplProofHeader[\s\S]*&Vec SelfhostMemoTraitNeplProofRecord[\s\S]*&Vec SelfhostMemoTraitNeplProofIndexEntry[\s\S]*Result unit SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*not eq record_len header\.record_count[\s\S]*RecordCountMismatch[\s\S]*not eq index_len header\.index_count[\s\S]*IndexCountMismatch[\s\S]*selfhost_memo_trait_neplproof_index_validation_record_loop[\s\S]*selfhost_memo_trait_neplproof_index_validation_entry_loop[\s\S]*selfhost_memo_trait_neplproof_index_validation_coverage_loop/,
    "decoded index table validation must compare header counts, revalidate records and entries, and require coverage",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_index_validation_duplicate_before_loop[\s\S]*Result bool SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*eq previous\.record_ordinal record_ordinal[\s\S]*IndexEntryMissing[\s\S]*IndexRecordOrdinalDuplicate/,
    "decoded index table validation must reject duplicate record ordinals while preserving typed defensive index-entry failures",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_index_validation_record_covered_loop[\s\S]*Result bool SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*IndexEntryMissing[\s\S]*selfhost_memo_trait_neplproof_index_validation_coverage_loop[\s\S]*selfhost_memo_trait_neplproof_index_validation_record_covered_loop[\s\S]*IndexRecordOrdinalMissing/,
    "decoded index table validation must reject missing record ordinal coverage and classify broken index vector reads separately",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_index_validation_error_kind_eq[\s\S]*RecordCountMismatch[\s\S]*IndexRecordOrdinalMissing/,
    "decoded index validation error equality must compare typed variants without string output",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_artifact_error_kind_eq[\s\S]*ArtifactSchemaMismatch[\s\S]*IndexRecordHashMismatch/,
    "artifact error equality must compare typed variants without string output",
);
assert.match(
    source,
    /stage0[\s\S]*key_schema_mismatch[\s\S]*key_payload_placeholder[\s\S]*policy_schema_mismatch[\s\S]*index_out_of_range[\s\S]*index_fingerprint_mismatch[\s\S]*index_record_hash_mismatch[\s\S]*record_count_mismatch[\s\S]*index_count_mismatch[\s\S]*record_invalid[\s\S]*index_entry_invalid[\s\S]*index_record_mismatch[\s\S]*index_record_ordinal_duplicate[\s\S]*index_record_ordinal_missing/,
    "stage0 must exercise accepted schema, key schema mismatch, placeholder payload, stale policy, fingerprint mismatch, payload-hash mismatch, count mismatch, invalid record/index, mismatch, duplicate ordinal, and missing coverage paths",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostCanonicalTypeKeyId|SelfhostTypeId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "proof artifact schema code must not store session-local ids, proof-store records, stable identities, or store sidecar index entries",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "proof artifact schema code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|return Ok immediately after fingerprint|stable_index-only|stable index only|index entry alone|index hit is authority/,
    "proof artifact schema must not document or implement fingerprint-only acceptance",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "proof artifact policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof artifact contract passed");

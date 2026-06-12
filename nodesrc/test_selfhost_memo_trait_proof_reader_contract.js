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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_reader.nepl";
const source = readRepoFile(repoRoot, relPath);
const facade = readRepoFile(repoRoot, TY_FACADE);
const tySources = readRepoFile(repoRoot, "nodesrc/selfhost_ty_sources.js");
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_reader" as \*$/m,
    "ty facade must re-export the .neplproof header reader module",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost_ty_sources must include the .neplproof reader in root re-export checks",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost_ty_sources must include the .neplproof reader in split source checks",
);
assert.match(
    tySources,
    /memo_trait_artifact_word_codec\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_canonical_key_payload_codec\.nepl"[\s\S]*memo_trait_proof_index\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_decoded\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_preseed\.nepl"/,
    "source order must place decoded artifact before the reader and the reader before preseed layers",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_artifact_word_codec" as \*$/m,
    "reader must use the shared artifact word codec",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_canonical_key" as \*$/m,
    "record reader must not pull the full canonical key fingerprint producer into the binary reader hot path",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "reader must delegate header schema validation to proof artifact schema module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "record reader may use proof-store typed proof payload constructors without calling proof-store acceptance APIs",
);
assert.match(
    codeOnly,
    /SelfhostMemoTraitStoredProofKind[\s\S]*selfhost_memo_trait_stored_aggregate_proof_new/,
    "reader proof-store import must be limited to typed stored-proof payload variants and constructors",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_decoded" as \*$/m,
    "record reader must build a decoded artifact owner through the decoded artifact boundary",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_preseed" as \*$/m,
    "reader header boundary must not depend on proof-store preseed acceptance",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_reader[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "reader documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /header prefix だけを読みます[\s\S]*record-only decoded artifact reader[\s\S]*fixed-width record table[\s\S]*trailing bytes を拒否します/,
    "reader docs must distinguish header-only behavior from record-only decoded artifact bounds",
);
assert.match(
    source,
    /source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`[\s\S]*authority ではありません/,
    "reader docs must exclude source-derived and session-local identity authority",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofReaderErrorKind:[\s\S]*MagicMismatch[\s\S]*WordReadInvalid %SelfhostMemoTraitArtifactWordReadErrorKind[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*ProofKindTagInvalid[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*DecodedArtifactInvalid %SelfhostMemoTraitNeplProofDecodedArtifactErrorKind/,
    "reader errors must split header failure, tag decode failure, record schema rejection, and decoded artifact rejection",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_reader_magic[\s\S]*792013/,
    "reader must define a .neplproof-specific magic word distinct from nested payload magic",
);
assert.match(
    source,
    /fn selfhost_memo_trait_neplproof_reader_header_words[\s\S]*6/,
    "reader must document the six-word header prefix shape",
);
assert.match(
    source,
    /fn selfhost_memo_trait_neplproof_reader_record_words[\s\S]*31/,
    "reader must define the schema-version-1 fixed record width",
);
assert.match(
    source,
    /\[word layout\]:[\s\S]*`0`: canonical payload schema version[\s\S]*`4\.\.11`: `MemoKey` \/ `MemoValue` source identity[\s\S]*`12\.\.16`: solver policy[\s\S]*`17`: stored proof kind tag[\s\S]*`18\.\.20`: aggregate field evidence[\s\S]*`21\.\.24`: Copy \/ Drop \/ Eq \/ Hash proof status tag[\s\S]*`30`: record payload hash/,
    "reader docs must pin the artifact schema version 1 fixed record word offsets",
);
assert.match(
    source,
    /`record_count` は artifact の外部入力[\s\S]*selfhost_memo_trait_neplproof_reader_record_count_limit[\s\S]*16384/,
    "reader must bound artifact-controlled record count before allocation",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_word_result[\s\S]*selfhost_memo_trait_artifact_word_codec_word_at_index_result bytes word_index[\s\S]*WordReadInvalid kind/,
    "reader must map shared word read failures into reader-local typed errors",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_header_checked_result[\s\S]*selfhost_memo_trait_neplproof_header_result artifact_schema canonical_schema policy_schema record_count index_count[\s\S]*HeaderInvalid kind/,
    "reader must delegate schema and count validation to the existing artifact header validator",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_reader_header_result[\s\S]*reader_word_result bytes 0[\s\S]*ne magic selfhost_memo_trait_neplproof_reader_magic[\s\S]*MagicMismatch[\s\S]*reader_word_result bytes 5[\s\S]*reader_header_checked_result artifact_schema canonical_schema policy_schema record_count index_count/,
    "reader public API must read magic first, then all header fields, then call the checked header boundary",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_reader_error_kind_eq[\s\S]*WordReadInvalid a_kind[\s\S]*selfhost_memo_trait_artifact_word_codec_read_error_kind_eq a_kind b_kind[\s\S]*HeaderInvalid a_kind[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq a_kind b_kind[\s\S]*RecordInvalid a_kind[\s\S]*RecordPushInvalid a_kind[\s\S]*DecodedArtifactInvalid a_kind[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_error_kind_eq a_kind b_kind/,
    "reader equality helper must compare nested word, artifact, std, and decoded artifact error payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_stage0[\s\S]*accepted_header[\s\S]*magic_mismatch[\s\S]*short_header[\s\S]*schema_mismatch[\s\S]*accepted_artifact[\s\S]*unknown_proof_kind[\s\S]*trailing_bytes[\s\S]*invalid_record/,
    "reader stage0 must cover header failures plus accepted record body, unknown proof kind, trailing bytes, and invalid record payload",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_reader_decoded_artifact_from_record_bytes_result[\s\S]*selfhost_memo_trait_neplproof_reader_header_result bytes[\s\S]*selfhost_memo_trait_neplproof_reader_records_result bytes header[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_from_records records[\s\S]*DecodedArtifactInvalid e/,
    "reader public record-body API must read header, decode records, and delegate owner construction to decoded artifact boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_record_key_from_words_result[\s\S]*selfhost_memo_trait_neplproof_record_key_from_parts_result canonical_schema fingerprint_schema fingerprint_root_hash canonical_payload_hash policy/,
    "reader must delegate serialized fingerprint parts to the artifact schema boundary instead of constructing canonical fingerprints directly",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_record_words_loop[\s\S]*ge idx selfhost_memo_trait_neplproof_reader_record_words[\s\S]*selfhost_memo_trait_neplproof_reader_record_word_result bytes record_ordinal idx[\s\S]*selfhost_memo_trait_neplproof_reader_record_word_push/,
    "reader must read fixed-width record words with a linear loop instead of a deep nested match chain",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_record_word_push[\s\S]*Result::Err e:[\s\S]*field::get e "error"[\s\S]*v::free v::vec_push_error_vec e[\s\S]*RecordPushInvalid error/,
    "record word push failure must close the Vec owner carried by VecPushError before returning the typed reader error",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_stage0_record_word_value[\s\S]*selfhost_memo_trait_neplproof_reader_stage0_push_record_words_loop[\s\S]*selfhost_memo_trait_artifact_word_codec_push_word_std_result bytes word/,
    "stage0 record fixture writer must use field-value projection plus a linear push loop",
);
assert.match(
    source,
    /reader_record_only_bounds_result[\s\S]*ne header\.index_count header\.record_count[\s\S]*IndexCountUnsupported[\s\S]*TrailingBytes/,
    "record-only reader must reject unsupported serialized index count and trailing bytes",
);
assert.match(
    source,
    /reader_proof_kind_result[\s\S]*KeyAndValue[\s\S]*KeyOnlyUnsupported[\s\S]*ValueOnlyUnsupported[\s\S]*ProofKindTagInvalid/,
    "reader must decode proof kind tags into typed enum variants and reject unknown tags",
);
assert.match(
    source,
    /reader_source_kind_result[\s\S]*MemoKeyTrait[\s\S]*MemoValueTrait[\s\S]*SourceKindTagInvalid/,
    "reader must decode source-kind tags into typed enum variants and reject unknown source-kind tags",
);
assert.match(
    source,
    /reader_field_evidence_result[\s\S]*Known range[\s\S]*MissingLayout[\s\S]*GenericArgumentUnsubstituted[\s\S]*CycleLimitReached[\s\S]*FieldEvidenceTagInvalid/,
    "reader must decode field-evidence tags into typed enum variants and reject unknown field-evidence tags",
);
assert.match(
    source,
    /reader_proof_status_result[\s\S]*Proven[\s\S]*Missing[\s\S]*Impure[\s\S]*Unknown[\s\S]*ProofStatusTagInvalid/,
    "reader must decode operation proof status tags into typed enum variants and reject unknown proof-status tags",
);
assert.match(
    source,
    /reader_hazard_result[\s\S]*NoHazard[\s\S]*CacheReferenceEscape[\s\S]*ExternalHandle[\s\S]*OwnerToken[\s\S]*PublicMutableState[\s\S]*Unknown[\s\S]*HazardTagInvalid/,
    "reader must decode hazard tags into typed enum variants and reject unknown hazard tags",
);
assert.match(
    source,
    /reader_memo_result_result[\s\S]*eq tag 1[\s\S]*eq payload 0[\s\S]*Result::Ok Result::Ok unit[\s\S]*ResultPayloadInvalid[\s\S]*reader_reject_kind_result payload[\s\S]*ResultTagInvalid/,
    "reader must decode Result tags fail-closed and reject nonzero Ok payloads before constructing MemoKey/MemoValue results",
);
assert.match(
    source,
    /reader_reject_kind_result[\s\S]*MissingTypeRecord[\s\S]*FunctionUnsupported[\s\S]*NamedLayoutUnknown[\s\S]*AppliedLayoutUnknown[\s\S]*ParameterUnresolved[\s\S]*RejectKindTagInvalid/,
    "reader must decode reject-kind tags into typed enum variants and reject unknown reject-kind tags",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)|selfhost_memo_trait_neplproof_decoded_candidate_range_preseed/,
    "reader must not call proof-store acceptance/preseed APIs directly",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "reader code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostTypeId|SelfhostCanonicalTypeKeyId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "reader header boundary must not store session-local ids or proof-store records/index entries",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|index hit is authority|proof acceptance by index|record payload hash only/,
    "reader header boundary must not claim proof acceptance from header or index metadata",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "reader header boundary must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof reader contract passed");

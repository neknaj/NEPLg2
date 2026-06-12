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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const tySources = readRepoFile(repoRoot, "nodesrc/selfhost_ty_sources.js");
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_serializer" as \*$/m,
    "ty facade must re-export the .neplproof serializer module",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost TY root re-export source list must include the .neplproof serializer",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost TY split source list must include the .neplproof serializer",
);
assert.match(
    tySources,
    /memo_trait_proof_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_payload_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_serializer\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_preseed\.nepl"/,
    "serializer must sit after reader/payload reader and before preseed in the TY source order",
);

assert.match(
    source,
    /^#import "\.\/memo_trait_artifact_word_codec" as \*$/m,
    "serializer must use the shared artifact word codec instead of implementing a private endian writer",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "serializer must validate typed header and record payloads through the artifact schema boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_payload_reader" as \*$/m,
    "serializer stage0 must round-trip through the payload reader boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_reader" as \*$/m,
    "serializer must share the .neplproof magic word with the reader boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "serializer may use stored proof payload enum/type definitions from the proof store boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_producer" as \*$/m,
    "serializer may use aggregate proof evidence enum/type definitions from the producer boundary",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_preseed" as \*$/m,
    "serializer must not depend on proof-store preseed acceptance",
);

assert.match(
    source,
    /# ty\/memo_trait_proof_serializer[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "serializer documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /serialized sidecar index table[\s\S]*record table の直後[\s\S]*fixed-width 4 word/,
    "serializer docs must state that Phase 1 writes a fixed-width serialized sidecar index table after the record table",
);
assert.match(
    source,
    /payload hash、record payload hash、fingerprint、index hit[\s\S]*proof を受理しません/,
    "serializer docs must state that hash, fingerprint, and index hits are not proof acceptance authority",
);
assert.match(
    source,
    /source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId`[\s\S]*authority にしません/,
    "serializer docs must exclude source-derived identity and session/store-local ids from serialized authority",
);
assert.match(
    source,
    /output bytes は serializer が owner として消費[\s\S]*partial output owner[\s\S]*閉じ/,
    "serializer docs must describe output owner consumption and cleanup on failure",
);
assert.match(
    source,
    /payload bytes は caller が所有[\s\S]*serializer は payload bytes owner を閉じません/,
    "serializer docs must state that borrowed payload bytes are not freed by the serializer",
);
assert.match(
    source,
    /O\(1\)[\s\S]*payload entry write は payload byte 長 b に対して O\(b\)[\s\S]*source scan、module graph、proof store lookup、preseed、diagnostic rendering は行いません/,
    "serializer docs must state fixed-width costs and exclude source scans, proof-store lookup, preseed, and diagnostic rendering from the hot path",
);

assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofSerializerErrorKind:[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*IndexEntryInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*ProofPayloadInvalid[\s\S]*RecordWordOrdinalInvalid[\s\S]*IndexWordOrdinalInvalid[\s\S]*WordPushInvalid %StdErrorKind[\s\S]*PayloadByteMissing[\s\S]*PayloadBytePushInvalid %StdErrorKind/,
    "serializer errors must keep typed nested payloads and preserve record, index, stored-proof-payload, and payload-copy failures",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_serializer_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_serializer_std_error_kind_eq/,
    "serializer equality helper must compare nested artifact and StdErrorKind payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_push_word_result[\s\S]*selfhost_memo_trait_artifact_word_codec_push_word_std_result bytes word[\s\S]*WordPushInvalid e/,
    "serializer word writes must go through the shared artifact word codec and typed error mapping",
);
assert.doesNotMatch(
    codeOnly,
    /rem_s|div_s|bit_and|shl|shr/,
    "serializer must not implement its own endian writer or bit-level word codec",
);

assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_header_checked_result[\s\S]*selfhost_memo_trait_neplproof_header_result header\.artifact_schema_version header\.canonical_payload_schema_version header\.policy_schema_version header\.record_count header\.index_count[\s\S]*Result::Ok checked[\s\S]*HeaderInvalid e/,
    "header writer must delegate schema validation and leave serialized-index count validation to index/table boundaries",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_serializer_header_result[\s\S]*selfhost_memo_trait_neplproof_serializer_header_checked_result header[\s\S]*selfhost_memo_trait_neplproof_serializer_push_header_words bytes checked[\s\S]*v::free bytes[\s\S]*Result::Err e/,
    "public header writer must validate before writing and close the output owner when validation fails",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_push_header_words[\s\S]*selfhost_memo_trait_neplproof_reader_magic[\s\S]*header\.artifact_schema_version[\s\S]*header\.canonical_payload_schema_version[\s\S]*header\.policy_schema_version[\s\S]*header\.record_count[\s\S]*header\.index_count/,
    "header writer must emit magic plus the five reader header words in reader order",
);

assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_stored_proof_result[\s\S]*selfhost_memo_trait_neplproof_serializer_field_evidence_result proof\.fields[\s\S]*selfhost_memo_trait_neplproof_serializer_validate_record_result[\s\S]*selfhost_memo_trait_neplproof_record_key_result record\.key\.canonical_fingerprint record\.key\.canonical_payload_schema_version record\.key\.canonical_payload_hash record\.key\.policy[\s\S]*selfhost_memo_trait_neplproof_serializer_stored_proof_result record\.proof[\s\S]*selfhost_memo_trait_neplproof_record_result key record\.proof_kind proof record\.record_payload_hash/,
    "record writer must revalidate record key, reader-visible stored proof payload, and record body before writing bytes",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_field_evidence_result[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::Known range:[\s\S]*selfhost_memo_trait_aggregate_field_range_is_valid range[\s\S]*ProofPayloadInvalid/,
    "serializer must reject Known field ranges that the reader would reject",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_serializer_record_result[\s\S]*selfhost_memo_trait_neplproof_serializer_validate_record_result record[\s\S]*selfhost_memo_trait_neplproof_serializer_record_loop bytes checked 0[\s\S]*v::free bytes[\s\S]*Result::Err e/,
    "public record writer must validate before writing and close the output owner when validation fails",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_index_words[\s\S]*4/,
    "serializer must use the schema-version-1 fixed index-entry width of 4 words",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_validate_index_entry_result[\s\S]*selfhost_memo_trait_neplproof_index_entry_result entry\.canonical_fingerprint entry\.record_ordinal entry\.record_payload_hash record_count[\s\S]*IndexEntryInvalid e/,
    "index writer must validate fingerprint schema, record ordinal range, and record payload hash through the artifact boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_index_word_value_result[\s\S]*0:[\s\S]*canonical_fingerprint\.schema_version[\s\S]*1:[\s\S]*canonical_fingerprint\.root_hash[\s\S]*2:[\s\S]*record_ordinal[\s\S]*3:[\s\S]*record_payload_hash[\s\S]*IndexWordOrdinalInvalid/,
    "index word projection must follow the reader's 4-word serialized index layout and fail closed on invalid field ordinal",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_serializer_index_entry_result[\s\S]*serializer_validate_index_entry_result entry record_count[\s\S]*serializer_index_loop bytes checked 0[\s\S]*v::free bytes[\s\S]*Result::Err e/,
    "public index writer must validate before writing and close the output owner on validation failure",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_record_words[\s\S]*31/,
    "serializer must use the schema-version-1 fixed record width of 31 words",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_record_word_value_result[\s\S]*0:[\s\S]*canonical_payload_schema_version[\s\S]*1:[\s\S]*canonical_fingerprint\.schema_version[\s\S]*2:[\s\S]*canonical_fingerprint\.root_hash[\s\S]*3:[\s\S]*canonical_payload_hash[\s\S]*4:[\s\S]*sources\.memo_key\.kind[\s\S]*17:[\s\S]*proof_kind[\s\S]*18:[\s\S]*proof\.fields[\s\S]*21:[\s\S]*copy_proof[\s\S]*25:[\s\S]*hazard[\s\S]*26:[\s\S]*key_result[\s\S]*28:[\s\S]*value_result[\s\S]*30:[\s\S]*record_payload_hash[\s\S]*RecordWordOrdinalInvalid/,
    "record word projection must follow the reader's 31-word layout and fail closed on invalid field ordinal",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_serializer_record_loop[\s\S]*ge field_ordinal selfhost_memo_trait_neplproof_serializer_record_words[\s\S]*record_word_value_result record field_ordinal[\s\S]*serializer_push_word_result bytes word[\s\S]*selfhost_memo_trait_neplproof_serializer_record_loop next record add field_ordinal 1[\s\S]*v::free bytes[\s\S]*Result::Err e/,
    "record writer must use a linear fixed-width loop and close the output owner on internal ordinal failure",
);

assert.match(
    source,
    /serializer_source_kind_artifact_word[\s\S]*MemoKeyTrait[\s\S]*MemoValueTrait/,
    "serializer must project source kind enum artifact words explicitly",
);
assert.match(
    source,
    /serializer_proof_kind_artifact_word[\s\S]*KeyAndValue[\s\S]*KeyOnlyUnsupported[\s\S]*ValueOnlyUnsupported/,
    "serializer must project stored proof kind enum artifact words explicitly",
);
assert.match(
    source,
    /serializer_field_evidence_artifact_word[\s\S]*Known _range[\s\S]*MissingLayout[\s\S]*GenericArgumentUnsubstituted[\s\S]*CycleLimitReached/,
    "serializer must project aggregate field evidence enum artifact words explicitly",
);
assert.match(
    source,
    /serializer_proof_status_artifact_word[\s\S]*Proven[\s\S]*Missing[\s\S]*Impure[\s\S]*Unknown/,
    "serializer must project proof status enum artifact words explicitly",
);
assert.match(
    source,
    /serializer_hazard_artifact_word[\s\S]*NoHazard[\s\S]*CacheReferenceEscape[\s\S]*ExternalHandle[\s\S]*OwnerToken[\s\S]*PublicMutableState[\s\S]*Unknown/,
    "serializer must project hazard enum artifact words explicitly",
);
assert.match(
    source,
    /serializer_reject_kind_artifact_word[\s\S]*MissingTypeRecord[\s\S]*ErrorTypeUnsupported[\s\S]*I64Unsupported[\s\S]*F32KeyUnsupported[\s\S]*F64Unsupported[\s\S]*StrUnsupported[\s\S]*NeverUnsupported[\s\S]*FunctionUnsupported[\s\S]*NamedLayoutUnknown[\s\S]*AppliedLayoutUnknown[\s\S]*ParameterUnresolved/,
    "serializer must project MemoKey/MemoValue reject-kind artifact words explicitly",
);
assert.match(
    source,
    /serializer_memo_result_artifact_word[\s\S]*Result::Ok _unit:[\s\S]*1[\s\S]*Result::Err _kind:[\s\S]*2[\s\S]*serializer_memo_result_payload[\s\S]*Result::Ok _unit:[\s\S]*0[\s\S]*Result::Err kind:[\s\S]*serializer_reject_kind_artifact_word kind/,
    "serializer must encode Result unit/reject payloads in the same form the reader decodes",
);

assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_serializer_payload_entry_result[\s\S]*let payload_len %i32 v::len payload[\s\S]*serializer_push_word_result bytes payload_len[\s\S]*serializer_payload_bytes_loop with_len payload 0 payload_len/,
    "payload entry writer must write the borrowed payload byte length before copying bytes",
);
assert.match(
    source,
    /serializer_payload_bytes_loop[\s\S]*v::get payload idx[\s\S]*Option::Some byte[\s\S]*serializer_payload_byte_push bytes byte[\s\S]*Option::None:[\s\S]*v::free bytes[\s\S]*PayloadByteMissing/,
    "payload copy loop must read borrowed bytes defensively and close only the output owner on impossible short reads",
);
assert.match(
    source,
    /serializer_payload_byte_push[\s\S]*v::push bytes byte[\s\S]*field::get e "error"[\s\S]*v::free v::vec_push_error_vec e[\s\S]*PayloadBytePushInvalid error/,
    "payload byte push failure must recover and close the Vec owner returned by VecPushError",
);

assert.match(
    source,
    /SelfhostMemoTraitNeplProofSerializerStage0Summary:[\s\S]*accepted_writer[\s\S]*invalid_header[\s\S]*invalid_record[\s\S]*invalid_index[\s\S]*invalid_proof_payload[\s\S]*payload_reader_roundtrip/,
    "serializer stage0 summary must cover accepted write, invalid header, invalid record, invalid index entry, invalid stored-proof payload, and payload-reader roundtrip",
);
assert.match(
    source,
    /serializer_stage0_build_artifact_bytes[\s\S]*serializer_header_result bytes0 header[\s\S]*serializer_record_result bytes1 record[\s\S]*serializer_index_entry_result bytes2 index_entry header\.record_count[\s\S]*serializer_payload_entry_result bytes3 payload/,
    "serializer stage0 must build bytes through the public header, record, serialized index, and payload entry writers",
);
assert.match(
    source,
    /serializer_stage0_roundtrip[\s\S]*serializer_stage0_build_nominal_table[\s\S]*serializer_stage0_build_artifact_bytes header record index_entry payload[\s\S]*serializer_stage0_roundtrip_owned nominal_table bytes/,
    "serializer stage0 must round-trip produced indexed bytes through the payload reader with a stable nominal table",
);
assert.match(
    source,
    /serializer_stage0_roundtrip_owned[\s\S]*selfhost_memo_trait_neplproof_payload_reader_materialized_artifact_result &nominal_table &bytes[\s\S]*v::free bytes[\s\S]*selfhost_memo_trait_stable_nominal_key_table_free nominal_table[\s\S]*selfhost_memo_trait_neplproof_payload_reader_materialized_artifact_free artifact/,
    "serializer roundtrip must free produced bytes, the nominal table, and materialized artifact owners",
);
assert.match(
    source,
    /serializer_stage0_invalid_record[\s\S]*SelfhostMemoTraitNeplProofRecord key SelfhostMemoTraitStoredProofKind::KeyAndValue proof 0/,
    "serializer stage0 must exercise record validator rejection through a placeholder record payload hash",
);
assert.match(
    source,
    /invalid_index_source[\s\S]*SelfhostMemoTraitNeplProofIndexEntry index_entry\.canonical_fingerprint 1 index_entry\.record_payload_hash[\s\S]*invalid_index[\s\S]*stage0_write_unit_owned header record invalid_index_source/,
    "serializer stage0 must exercise index-entry validator rejection through an out-of-range record ordinal",
);
assert.match(
    source,
    /serializer_stage0_invalid_proof_payload_record[\s\S]*selfhost_memo_trait_aggregate_field_range_new 2147483647 1[\s\S]*SelfhostMemoTraitAggregateFieldEvidence::Known range[\s\S]*SelfhostMemoTraitNeplProofRecord key SelfhostMemoTraitStoredProofKind::KeyAndValue proof 3003/,
    "serializer stage0 must exercise reader-inverse rejection for invalid Known field ranges",
);
assert.match(
    source,
    /invalid_proof_payload[\s\S]*selfhost_memo_trait_neplproof_serializer_error_kind_eq[\s\S]*SelfhostMemoTraitNeplProofSerializerErrorKind::ProofPayloadInvalid/,
    "serializer doctest must assert invalid Known field ranges are rejected as ProofPayloadInvalid",
);

assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized)|selfhost_memo_trait_neplproof_decoded_candidate_range_preseed|selfhost_memo_trait_neplproof_record_append|selfhost_memo_trait_aggregate_proof_to_record/,
    "serializer must not call proof-store lookup/push/preseed/materialized acceptance or producer-gate APIs directly",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "serializer code must not use source text, spans, paths, display names, diagnostics, lexemes, or module paths as authority",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostTypeId|SelfhostCanonicalTypeKeyId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "serializer code must not persist session-local TypeId, store-local canonical ids, proof-store records, or stable index entries",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|hash only|hash-only|index hit is authority|proof acceptance by index|record payload hash only/,
    "serializer must not document or implement hash-only, fingerprint-only, or index-only proof acceptance",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "serializer policy must not introduce line-count, file-size, or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof serializer contract passed");

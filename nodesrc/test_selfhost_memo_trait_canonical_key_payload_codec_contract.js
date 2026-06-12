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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload_codec.nepl";
const source = readRepoFile(repoRoot, relPath);
const facade = readRepoFile(repoRoot, TY_FACADE);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_canonical_key_payload_codec" as \*$/m,
    "ty facade must re-export the canonical key payload bytes codec",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_artifact_word_codec" as \*$/m,
    "canonical key payload codec must share the artifact word codec instead of reimplementing low-level little-endian word rules",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost_ty_sources must include the codec in root re-export checks",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost_ty_sources must include the codec in split source checks",
);
assert.match(
    source,
    /# ty\/memo_trait_canonical_key_payload_codec[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "codec module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /serialized payload には、payload schema、node kind、primitive stable code、stable nominal key material、argument order だけ[\s\S]*SelfhostCanonicalTypeKeyId[\s\S]*SelfhostNamedTypeId[\s\S]*SelfhostTypeId[\s\S]*source text、span、path、display name、diagnostic text、lexeme/,
    "codec docs must state the allowed payload authority and explicitly reject store-local ids and source-derived authority",
);
assert.match(
    source,
    /record key 内の payload hash や bytes 内の hash 値を信用して受理する経路は持ちません/,
    "codec docs must state that bytes-stored hashes are not acceptance authority",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitCanonicalKeyPayloadDecoded:[\s\S]*schema_version %i32[\s\S]*arena %SelfhostCanonicalTypeKeyArena[\s\S]*root %SelfhostCanonicalTypeKeyId/,
    "decode result must materialize a canonical key arena and root key",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind:[\s\S]*PayloadSchemaMismatch[\s\S]*UnexpectedEnd[\s\S]*TrailingBytes[\s\S]*UnknownNodeTag[\s\S]*UnknownPrimitiveCode[\s\S]*NodeCountNegative[\s\S]*ArgCountNegative[\s\S]*NodeCountLimitExceeded[\s\S]*ArgCountLimitExceeded[\s\S]*RootOutOfRange[\s\S]*ArgTargetOutOfRange[\s\S]*InvalidArgumentRange[\s\S]*TypeParameterUnsupported[\s\S]*FunctionTypeUnsupported[\s\S]*NominalKeyMissing[\s\S]*NominalKeyDuplicate[\s\S]*NominalKeyInvalid %SelfhostMemoTraitStableNominalKeyErrorKind[\s\S]*WordHighBitUnsupported[\s\S]*OutOfMemory[\s\S]*PayloadHashInvalid %SelfhostMemoTraitCanonicalKeyPayloadErrorKind/,
    "decode errors must be typed enum variants for schema, bounds, unsupported nodes, nominal lookup, allocation, and hash projection failures",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_header_checked_result[\s\S]*NodeCountNegative[\s\S]*ArgCountNegative[\s\S]*NodeCountLimitExceeded[\s\S]*ArgCountLimitExceeded[\s\S]*RootOutOfRange[\s\S]*TrailingBytes/,
    "header decoder must fail closed on negative counts, count limits, root range, and trailing bytes",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_error_from_word_read[\s\S]*SelfhostMemoTraitArtifactWordReadErrorKind::UnexpectedEnd[\s\S]*SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind::UnexpectedEnd[\s\S]*SelfhostMemoTraitArtifactWordReadErrorKind::WordHighBitUnsupported[\s\S]*SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind::WordHighBitUnsupported/,
    "canonical key payload codec must map shared word read errors into its own typed decode error surface",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_word_at_byte_result[\s\S]*selfhost_memo_trait_artifact_word_codec_word_at_byte_result bytes byte_offset[\s\S]*selfhost_memo_trait_canonical_key_payload_codec_error_from_word_read/,
    "canonical key payload byte-offset reader must delegate to the shared artifact word codec",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_word_at_index_result[\s\S]*selfhost_memo_trait_artifact_word_codec_word_at_index_result bytes word_index[\s\S]*selfhost_memo_trait_canonical_key_payload_codec_error_from_word_read/,
    "canonical key payload word-index reader must delegate to the shared artifact word codec",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_push_word[\s\S]*selfhost_memo_trait_artifact_word_codec_push_word_std_result bytes word/,
    "canonical key payload stage0 writer must delegate to the shared artifact word codec",
);
assert.doesNotMatch(
    codeOnly,
    /let b0 %i32 cast b0_raw|let b1 %i32 cast b1_raw|gt b3 127|rem_s word 256|div_s word 256/,
    "canonical key payload codec must not keep a second copy of byte-to-word or word-to-byte arithmetic",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_node_result[\s\S]*eq tag 1[\s\S]*SelfhostCanonicalTypeKeyNode::Primitive[\s\S]*eq tag 2[\s\S]*SelfhostCanonicalTypeKeyNode::Named[\s\S]*eq tag 3[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*eq tag 4[\s\S]*TypeParameterUnsupported[\s\S]*eq tag 5[\s\S]*FunctionTypeUnsupported[\s\S]*UnknownNodeTag/,
    "node decoder must explicitly classify every Phase 1 tag and fail closed for unsupported Parameter and Function nodes",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_primitive_kind_result[\s\S]*eq code 101[\s\S]*SelfhostPrimitiveTypeKind::Error[\s\S]*eq code 111[\s\S]*SelfhostPrimitiveTypeKind::Never[\s\S]*UnknownPrimitiveCode/,
    "primitive code decoder must map stable primitive codes explicitly and reject unknown codes",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_codec_find_nominal_id_loop[\s\S]*selfhost_memo_trait_canonical_key_payload_codec_stable_nominal_key_eq record\.key key[\s\S]*NominalKeyDuplicate[\s\S]*NominalKeyMissing/,
    "codec must reverse-map stable nominal keys through the stable table and reject missing or duplicate records",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_decode_and_hash_result[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_result table bytes[\s\S]*selfhost_memo_trait_canonical_key_payload_decoded_hash_result table &decoded[\s\S]*selfhost_memo_trait_canonical_key_payload_decoded_free decoded[\s\S]*PayloadHashInvalid/,
    "decode-and-hash convenience API must decode to a materialized arena, call the existing hash producer, free decoded owner, and map hash failures",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_canonical_key_payload_decoded_hash_result[\s\S]*selfhost_memo_trait_canonical_key_payload_hash_result table arena root/,
    "decoded hash projection must delegate to memo_trait_canonical_key_payload.nepl instead of duplicating hash authority",
);
assert.match(
    source,
    /SelfhostMemoTraitCanonicalKeyPayloadCodecStage0Summary:[\s\S]*named_payload[\s\S]*applied_payload[\s\S]*schema_mismatch[\s\S]*trailing_bytes[\s\S]*unknown_primitive[\s\S]*unsupported_function[\s\S]*arg_target_out_of_range[\s\S]*invalid_argument_range[\s\S]*nominal_missing/,
    "stage0 summary must cover accepted named/applied payloads and representative fail-closed decode paths",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_proof_store"|#import "\.\/memo_trait_proof_artifact"|#import "\.\/memo_trait_proof_preseed"|#import "neplg2\/core\/(?:check|lower|hir|resource|backend)\//,
    "codec must stay below proof store, artifact/preseed, checker, HIR, Resource IR, and backend layers",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme/,
    "codec code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry|record_index/,
    "codec code must not persist arena-local TypeId, proof-store identities, proof-store records, or record indexes",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "codec policy must not introduce line-count, comment-length, or file-size restrictions",
);

console.log("selfhost memo trait canonical key payload codec contract passed");

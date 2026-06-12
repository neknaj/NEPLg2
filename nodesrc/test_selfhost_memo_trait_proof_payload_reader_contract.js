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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const tySources = readRepoFile(repoRoot, "nodesrc/selfhost_ty_sources.js");
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_payload_reader" as \*$/m,
    "ty facade must re-export the .neplproof payload section reader",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost TY root re-export source list must include the payload section reader",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost TY split source list must include the payload section reader",
);
assert.match(
    tySources,
    /memo_trait_proof_decoded\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_payload_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_serializer\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_preseed\.nepl"/,
    "payload section reader must sit after the record reader and before serializer/preseed in the TY source order",
);

assert.match(
    source,
    /^#import "core\/option" as \*$/m,
    "payload reader stage0 must import option constructors explicitly instead of relying on transitive imports",
);
assert.match(
    source,
    /^#import "\.\/key" as \*$/m,
    "payload reader must materialize decoded canonical payload roots into a shared canonical key arena",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key" as \*$/m,
    "payload reader must take the stable nominal key table required by the canonical payload codec",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key_payload_codec" as \*$/m,
    "payload reader must decode canonical payload bytes through the dedicated codec",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "payload reader must reuse artifact schema constants and typed record definitions",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_decoded" as \*$/m,
    "payload reader must return the decoded artifact owner produced by the decoded artifact boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_reader" as \*$/m,
    "payload reader must delegate header, fixed-width record, and serialized index decoding to the existing reader",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_preseed" as \*$/m,
    "payload reader must not depend on preseed acceptance",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "payload reader must not import proof-store acceptance or store-local identity layers",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_producer" as \*$/m,
    "payload reader must not pull producer-side proof construction into the artifact reader layer",
);

assert.match(
    source,
    /# ty\/memo_trait_proof_payload_reader[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "payload reader documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /indexed prefix の直後[\s\S]*`payload_byte_len` word[\s\S]*canonical payload codec bytes/,
    "payload reader docs must define the payload section layout after the indexed prefix",
);
assert.match(
    source,
    /prefix owner を確保して複写する前[\s\S]*selfhost_memo_trait_neplproof_reader_indexed_prefix_byte_count_result[\s\S]*過大な `record_count` \/ `index_count` は allocation 前に fail-closed/,
    "payload reader docs must require indexed prefix bounds validation before prefix owner allocation and copy",
);
assert.match(
    source,
    /payload hash、canonical fingerprint、policy、proof kind、store relation[\s\S]*preseed \/ proof store 側が再検査/,
    "payload reader docs must state that proof acceptance remains in preseed and proof-store layers",
);
assert.match(
    source,
    /`SelfhostTypeId`、source text、span、path suffix、display name、diagnostic text、lexeme、record payload hash 単独、fingerprint hit 単独、index hit 単独[\s\S]*authority ではありません/,
    "payload reader docs must exclude source-derived identity, session-local ids, hash-only acceptance, and index-only acceptance",
);
assert.match(
    source,
    /O\(b \+ k\)[\s\S]*materialized key id vector の構築は O\(n\)[\s\S]*FileSystem、source scan は行いません/,
    "payload reader docs must describe algorithmic cost and exclude filesystem/source scanning from the hot path",
);

assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofPayloadReaderMaterializedArtifact:[\s\S]*artifact %SelfhostMemoTraitNeplProofDecodedArtifact[\s\S]*materialized_key_arena %SelfhostCanonicalTypeKeyArena[\s\S]*materialized_key_ids %Vec SelfhostCanonicalTypeKeyId/,
    "payload reader result must bundle the decoded artifact owner, shared materialized key arena, and record-ordinal key id vector",
);
assert.match(
    source,
    /`materialized_key_ids\[i\]` は `artifact\.records\[i\]` の canonical payload bytes から materialize された root key[\s\S]*`materialized_key_arena` 内だけで意味[\s\S]*record ordinal、store-local id、serialized id として扱ってはいけません/,
    "payload reader docs must state that materialized key ids are arena-local projections and not record ordinals, store-local ids, or serialized ids",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_payload_reader_materialized_artifact_free[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_free[\s\S]*selfhost_canonical_type_key_arena_free[\s\S]*v::free field::get artifact "materialized_key_ids"/,
    "payload reader free boundary must close every owner in the materialized artifact bundle",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofPayloadReaderErrorKind:[\s\S]*HeaderReadInvalid %SelfhostMemoTraitNeplProofReaderErrorKind[\s\S]*RecordDecodeInvalid %SelfhostMemoTraitNeplProofReaderErrorKind[\s\S]*PayloadLengthWordInvalid %SelfhostMemoTraitArtifactWordReadErrorKind[\s\S]*PayloadDecodeInvalid %SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind[\s\S]*MaterializedKeyCopyInvalid %SelfhostCanonicalTypeKeyCopyErrorKind[\s\S]*TrailingBytes/,
    "payload reader errors must keep typed nested payloads for header, record, word, payload decode, materialization, and trailing bytes",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_payload_reader_error_kind_eq[\s\S]*selfhost_memo_trait_neplproof_reader_error_kind_eq[\s\S]*selfhost_memo_trait_artifact_word_codec_read_error_kind_eq[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_error_kind_eq[\s\S]*selfhost_canonical_type_key_copy_error_kind_eq/,
    "payload reader equality helper must compare nested typed error payloads",
);

assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_indexed_prefix_word_count[\s\S]*record_only_word_count header[\s\S]*header\.index_count[\s\S]*selfhost_memo_trait_neplproof_payload_reader_index_words/,
    "payload reader must compute the indexed prefix from header, fixed records, and serialized index entries",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_indexed_prefix_byte_count_result[\s\S]*selfhost_memo_trait_neplproof_reader_indexed_prefix_byte_count_result header[\s\S]*RecordDecodeInvalid e/,
    "payload reader must use the reader-owned indexed prefix byte-count preflight and preserve reader typed errors",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_decoded_artifact_result[\s\S]*selfhost_memo_trait_neplproof_payload_reader_indexed_prefix_byte_count_result header[\s\S]*Result::Ok prefix_byte_count[\s\S]*selfhost_memo_trait_neplproof_payload_reader_record_prefix_result bytes prefix_byte_count[\s\S]*selfhost_memo_trait_neplproof_reader_decoded_artifact_from_indexed_record_bytes_result &prefix[\s\S]*v::free prefix/,
    "payload reader must validate indexed prefix bounds before copying the prefix, delegate indexed record decoding to the existing reader, and free the prefix owner",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_length_result[\s\S]*selfhost_memo_trait_artifact_word_codec_word_at_byte_result bytes offset[\s\S]*PayloadLengthWordInvalid/,
    "payload length word reads must use the shared artifact word codec and typed error mapping",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_payload_end_result[\s\S]*lt payload_length 0[\s\S]*lt payload_start offset[\s\S]*lt payload_end payload_start[\s\S]*gt payload_end v::len bytes[\s\S]*PayloadBytesUnexpectedEnd/,
    "payload range calculation must fail closed on negative length, integer wrap, and short bytes",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_decode_one_result[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_result nominal_table &payload_bytes[\s\S]*selfhost_canonical_type_key_copy_from_arena decoded_arena materialized_key_arena decoded_root[\s\S]*selfhost_memo_trait_canonical_key_payload_decoded_free decoded[\s\S]*v::free payload_bytes/,
    "single payload decode must materialize through the codec, copy into the shared arena, and free temporary decoded payload owners",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_loop[\s\S]*ge record_ordinal record_count[\s\S]*TrailingBytes[\s\S]*Result::Ok SelfhostMemoTraitNeplProofPayloadReaderMaterializedArtifact artifact materialized_key_arena materialized_key_ids[\s\S]*selfhost_memo_trait_neplproof_payload_reader_decode_one_result/,
    "payload reader loop must process exactly record_count payload entries and reject trailing bytes",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_payload_reader_materialized_artifact_result[\s\S]*selfhost_memo_trait_neplproof_reader_header_result bytes[\s\S]*selfhost_memo_trait_neplproof_payload_reader_decoded_artifact_result bytes header[\s\S]*selfhost_canonical_type_key_arena_new[\s\S]*let key_ids_result %Result Vec SelfhostCanonicalTypeKeyId StdErrorKind v::new[\s\S]*selfhost_memo_trait_neplproof_payload_reader_indexed_prefix_byte_count_result header[\s\S]*Result::Ok payload_offset[\s\S]*selfhost_memo_trait_neplproof_payload_reader_loop/,
    "public payload reader API must read the header, decode indexed fixed tables, allocate materialized key storage, reuse the checked prefix offset, then scan payloads linearly",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_abort[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_free artifact[\s\S]*selfhost_canonical_type_key_arena_free arena[\s\S]*v::free key_ids/,
    "shared abort helper must close decoded artifact, materialized arena, and key id vector owners",
);

assert.match(
    source,
    /SelfhostMemoTraitNeplProofPayloadReaderStage0Summary:[\s\S]*accepted[\s\S]*short_payload[\s\S]*payload_decode_invalid[\s\S]*trailing_bytes/,
    "payload reader stage0 summary must cover accepted artifact, short payload bytes, invalid payload decode, and trailing bytes",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_stage0_build_nominal_table[\s\S]*selfhost_named_type_id_new 10[\s\S]*selfhost_memo_trait_stable_nominal_key_result \(some 31\) \(some 41\) \(some 1\) 1/,
    "payload reader stage0 must build a stable nominal table instead of relying on source names",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_stage0_record_word_value[\s\S]*selfhost_memo_trait_canonical_key_payload_schema_version[\s\S]*1001[\s\S]*2002[\s\S]*3003/,
    "payload reader stage0 must keep fixed record schema fields explicit for the reader contract",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_stage0_index_word_value[\s\S]*selfhost_memo_trait_canonical_key_payload_schema_version[\s\S]*1001[\s\S]*0[\s\S]*3003[\s\S]*selfhost_memo_trait_neplproof_payload_reader_stage0_push_index_loop/,
    "payload reader stage0 must write a serialized sidecar index entry that points to the fixed record",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_payload_reader_stage0_after_table[\s\S]*accepted[\s\S]*short_payload[\s\S]*payload_decode_invalid[\s\S]*trailing_bytes[\s\S]*SelfhostMemoTraitNeplProofPayloadReaderStage0Summary/,
    "payload reader stage0 body must run every representative accepted and rejected path",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_payload_reader_stage0[\s\S]*selfhost_memo_trait_neplproof_payload_reader_stage0_build_nominal_table[\s\S]*selfhost_memo_trait_neplproof_payload_reader_stage0_after_table nominal_table/,
    "payload reader public stage0 entry must build the nominal table and delegate to the smoke body",
);

assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_neplproof_(?:record_|decoded_).*preseed|selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)/,
    "payload reader must not call proof-store or preseed acceptance APIs directly",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "payload reader code must not use source text, spans, paths, display names, diagnostics, lexemes, or module paths as authority",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "payload reader code must not persist session-local TypeId or proof-store identities",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "payload reader policy must not introduce line-count, comment-length, or file-size restrictions",
);

console.log("selfhost memo trait proof payload reader contract passed");

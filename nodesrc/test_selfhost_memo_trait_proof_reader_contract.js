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
    /memo_trait_artifact_word_codec\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_canonical_key_payload_codec\.nepl"[\s\S]*memo_trait_proof_index\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_reader\.nepl",\s*"stdlib\/neplg2\/core\/ty\/ty\/memo_trait_proof_decoded\.nepl"/,
    "source order must place the shared word codec before byte codecs and the reader before decoded artifact/preseed layers",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_artifact_word_codec" as \*$/m,
    "reader must use the shared artifact word codec",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "reader must delegate header schema validation to proof artifact schema module",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "reader header boundary must not depend on proof-store implementation details",
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
    /header prefix だけを読みます[\s\S]*trailing bytes を拒否しません[\s\S]*record_count.*index_count/,
    "reader docs must define that this slice reads only the header prefix and leaves full artifact bounds to later record/index readers",
);
assert.match(
    source,
    /source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`[\s\S]*authority ではありません/,
    "reader docs must exclude source-derived and session-local identity authority",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofReaderErrorKind:[\s\S]*MagicMismatch[\s\S]*WordReadInvalid %SelfhostMemoTraitArtifactWordReadErrorKind[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind/,
    "reader errors must split magic mismatch, low-level word read failure, and header schema rejection",
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
    /pub fn selfhost_memo_trait_neplproof_reader_error_kind_eq[\s\S]*MagicMismatch[\s\S]*WordReadInvalid a_kind[\s\S]*selfhost_memo_trait_artifact_word_codec_read_error_kind_eq a_kind b_kind[\s\S]*HeaderInvalid a_kind[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq a_kind b_kind/,
    "reader equality helper must compare nested word and artifact error payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_reader_stage0[\s\S]*accepted_header[\s\S]*magic_mismatch[\s\S]*short_header[\s\S]*schema_mismatch/,
    "reader stage0 must cover accepted header, magic mismatch, short header, and schema mismatch",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)|selfhost_memo_trait_neplproof_decoded_artifact_from_records|selfhost_memo_trait_neplproof_decoded_candidate_range_preseed/,
    "reader header boundary must not call proof-store, decoded artifact construction, or preseed APIs directly",
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

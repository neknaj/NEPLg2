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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_artifact_word_codec.nepl";
const source = readRepoFile(repoRoot, relPath);
const facade = readRepoFile(repoRoot, TY_FACADE);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_artifact_word_codec" as \*$/m,
    "ty facade must re-export the shared artifact word codec",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost_ty_sources must include the shared artifact word codec in root re-export checks",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost_ty_sources must include the shared artifact word codec in split source checks",
);
assert.match(
    source,
    /# ty\/memo_trait_artifact_word_codec[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "shared artifact word codec documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /31-bit nonnegative[\s\S]*4 byte 目の high bit[\s\S]*WordHighBitUnsupported[\s\S]*schema version を上げ/,
    "word codec docs must define the 31-bit nonnegative word contract and future schema boundary",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitArtifactWordReadErrorKind:[\s\S]*UnexpectedEnd[\s\S]*WordHighBitUnsupported/,
    "word codec read errors must split short input from high-bit unsupported words",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitArtifactWordWriteErrorKind:[\s\S]*NegativeWordUnsupported[\s\S]*PushFailed %StdErrorKind/,
    "word codec write errors must split negative words from vector push failures",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_artifact_word_codec_word_at_byte_result[\s\S]*or lt byte_offset 0 ge add byte_offset 3 byte_len[\s\S]*WordHighBitUnsupported[\s\S]*Result::Ok add lo hi/,
    "byte-offset reader must check four-byte availability, reject high-bit words, and return the reconstructed word",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_artifact_word_codec_word_at_index_result[\s\S]*lt word_index 0[\s\S]*selfhost_memo_trait_artifact_word_codec_word_at_byte_result bytes mul word_index 4/,
    "word-index reader must be a thin checked adapter over byte-offset reader",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_artifact_word_codec_push_word_result[\s\S]*lt word 0[\s\S]*v::free bytes[\s\S]*NegativeWordUnsupported[\s\S]*rem_s word 256[\s\S]*selfhost_memo_trait_artifact_word_codec_push_byte bytes3 b3/,
    "word writer must close the owner on negative input and append four little-endian bytes on success",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_artifact_word_codec_push_word_std_result[\s\S]*selfhost_memo_trait_artifact_word_codec_write_error_to_std/,
    "word codec must expose a StdErrorKind adapter for existing stage0 callers without changing the low-level typed error",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_artifact_word_codec_read_error_kind_eq[\s\S]*UnexpectedEnd[\s\S]*WordHighBitUnsupported[\s\S]*pub fn selfhost_memo_trait_artifact_word_codec_write_error_kind_eq[\s\S]*NegativeWordUnsupported[\s\S]*PushFailed/,
    "word codec must provide typed equality helpers for doctest and contract checks",
);
assert.match(
    source,
    /selfhost_memo_trait_artifact_word_codec_stage0[\s\S]*66051[\s\S]*unexpected_end[\s\S]*high_bit_word[\s\S]*negative_write/,
    "word codec stage0 must exercise accepted roundtrip, short read, high-bit read rejection, and negative write rejection",
);
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_proof_store"|#import "\.\/memo_trait_proof_artifact"|#import "\.\/memo_trait_proof_preseed"|#import "neplg2\/core\/(?:check|lower|hir|resource|backend)\//,
    "shared word codec must stay below proof store, proof artifact schema, preseed, checker, HIR, Resource IR, and backend layers",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "shared word codec code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "shared word codec must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait artifact word codec contract passed");

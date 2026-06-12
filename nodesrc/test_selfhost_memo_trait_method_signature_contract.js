#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_method_signature.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const sourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";

const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const proofStore = read(proofStoreRelPath);
const memoTraitSource = read(sourceRelPath);

assert.doesNotMatch(
    facade,
    /^pub #import "\.\/module\/memo_trait_method_signature" as \*$/m,
    "module checker facade must not expose method signature normalizer before the stable public-surface gate consumes it",
);
assert.match(
    source,
    /# check\/module\/memo_trait_method_signature[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "method signature normalizer must document purpose, contract, current limits, complexity, and a doctest",
);
assert.match(
    source,
    /source text は canonical surface spelling の分類にだけ使います[\s\S]*accepted fingerprint は受理後の固定 role code だけから作ります/,
    "docs must separate canonical spelling classification from accepted fingerprint authority",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitMethodSignatureRole:[\s\S]*MemoKeyEq[\s\S]*MemoKeyHash32[\s\S]*MemoValueMark/,
    "method names must be normalized to typed method roles",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitMethodSignatureErrorKind:[\s\S]*BodySegmentRejected %SelfhostTraitBodyMethodSegmentErrorKind[\s\S]*MethodCountMismatch %SelfhostMemoTraitSourceKind[\s\S]*HeaderNameMismatch %SelfhostMemoTraitMethodSignatureRole[\s\S]*HeaderTypeMismatch %SelfhostMemoTraitMethodSignatureRole[\s\S]*HeaderLambdaMismatch %SelfhostMemoTraitMethodSignatureRole[\s\S]*DefaultBodyMismatch %SelfhostMemoTraitMethodSignatureRole/,
    "normalization failures must remain typed enum payloads",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitMethodSignatureEvidence:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*method_count %i32[\s\S]*normalized_signature_hash %Option i32/,
    "accepted evidence must keep kind, method count, and optional normalized fingerprint as typed fields",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_method_signature_result %impure fn str impure fn &Vec SelfhostToken impure fn SelfhostMemoTraitSourceKind impure fn SelfhostSyntaxRange Result SelfhostMemoTraitMethodSignatureEvidence SelfhostMemoTraitMethodSignatureErrorKind/,
    "public normalizer API must return a typed Result",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_method_signature_result"),
    /selfhost_trait_body_method_segment_list_from_envelope[\s\S]*selfhost_memo_trait_method_signature_result_with_segments[\s\S]*selfhost_trait_body_method_segment_list_free[\s\S]*BodySegmentRejected/,
    "public normalizer must consume segmenter evidence and close the segment list owner",
);
assert.doesNotMatch(
    code,
    /^pub fn selfhost_memo_trait_method_signature_result_with_segments\b/m,
    "segment-list entry must stay private so callers cannot bypass envelope segmentation provenance",
);
assert.match(
    source,
    /selfhost_memo_trait_method_signature_result_with_segments: segment list を method signature evidence へ[\s\S]*公開 API にしないことで[\s\S]*fake segment aggregate[\s\S]*trusted signature pipeline を迂回[\s\S]*fn selfhost_memo_trait_method_signature_result_with_segments/,
    "private segment-list helper must document why it is not an authority boundary",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_method_signature_key_result"),
    /selfhost_trait_body_method_segment_list_len list 2[\s\S]*MemoKeyEq[\s\S]*MemoKeyHash32[\s\S]*selfhost_memo_trait_method_signature_evidence_result SelfhostMemoTraitSourceKind::MemoKeyTrait 2/,
    "MemoKey must require the expected two-method role set before evidence is accepted",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_method_signature_value_result"),
    /selfhost_trait_body_method_segment_list_len list 1[\s\S]*MemoValueMark[\s\S]*selfhost_memo_trait_method_signature_evidence_result SelfhostMemoTraitSourceKind::MemoValueTrait 1/,
    "MemoValue must require the expected one-method role set before evidence is accepted",
);
assert.match(
    source,
    /"memo_key_eq"[\s\S]*"Self"[\s\S]*"bool"[\s\S]*"memo_key_hash32"[\s\S]*"i32"[\s\S]*"memo_value_mark"/,
    "normalizer must explicitly classify current stdlib method names and type atoms",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_method_signature_value_mark_body_ok"),
    /header\\body:[\s\S]*ExpectIdent[\s\S]*selfhost_memo_trait_method_signature_lexeme_at_same source tokens header 7 body 0/,
    "MemoValue default body must return the method binder rather than accepting a fixed free identifier",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_method_signature_hash"),
    /SelfhostMemoTraitSourceKind::MemoKeyTrait:[\s\S]*selfhost_memo_trait_method_signature_mix3 4111 4211 4212[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait:[\s\S]*selfhost_memo_trait_method_signature_mix2 4112 4311/,
    "normalized hash must be built from fixed role/default-body material",
);
assert.doesNotMatch(
    functionBlock(code, "selfhost_memo_trait_method_signature_hash"),
    /source\s+%str|\bsource\b|span|range|lexeme|str_slice|str_eq|path|display|diagnostic/i,
    "hash function must not read source text, spans, ranges, lexemes, paths, display names, or diagnostics",
);
assert.match(
    source,
    /selfhost_memo_trait_method_signature_lexeme_at_eq: range 内 token の spelling が期待値と一致するか[\s\S]*canonical surface spelling の分類にだけ使います[\s\S]*selfhost_memo_trait_method_signature_lexeme_at_eq[\s\S]*string_slice::str_slice_result[\s\S]*string_search::str_eq/,
    "lexeme helper may classify canonical surface spelling but must be documented as non-authority for fingerprints",
);
assert.match(
    source,
    /wrong_count_rejected[\s\S]*wrong_type_rejected[\s\S]*wrong_lambda_rejected[\s\S]*segment_rejected[\s\S]*NormalizedHashPlaceholder/,
    "stage0 coverage must include count, type, lambda, segment, and placeholder error surfaces",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_source_identity_new|SelfhostMemoTraitTrustedSourceRegistry|signature_available\s+true|public_surface_hash|stable_source_evidence/i,
    "method normalizer must not construct trusted source identities, registries, or public surface records directly",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_method_signature|SelfhostMemoTraitMethodSignature/,
    "proof store must not depend directly on checker-layer method signature evidence",
);
assert.doesNotMatch(
    memoTraitSource,
    /memo_trait_method_signature|SelfhostMemoTraitMethodSignature/,
    "memo trait source materializer must not consume method signature evidence before the public surface gate is wired",
);
assert.doesNotMatch(
    code,
    /line count|comment length|file size|500 行/,
    "method signature policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait method signature contract passed");

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

const relPath = "stdlib/neplg2/core/syntax/parser/trait_body_segmenter.nepl";
const segmenter = read(relPath);
const code = stripDocComments(segmenter);
const exprStartPredicate = read("stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl");
const proofStore = read("stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl");
const memoTraitSource = read("stdlib/neplg2/core/ty/ty/memo_trait_source.nepl");
const memoTraitPolicy = read("stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl");

assert.match(
    segmenter,
    /# parser\/trait_body_segmenter[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "trait body segmenter must document purpose, contract, current limits, complexity, and a doctest",
);
assert.match(
    segmenter,
    /expression body 用の `body_segmenter` は `KwFn` を expression start として扱わないため[\s\S]*trait method declaration には使えません/,
    "docs must explain why the expression body segmenter is not reused as the method declaration parser",
);
assert.match(
    segmenter,
    /pub struct SelfhostTraitBodyMethodSegment:[\s\S]*header %SelfhostSyntaxRange[\s\S]*default_body %SelfhostSyntaxRange/,
    "method segment must separate method header range from default body range",
);
assert.match(
    segmenter,
    /pub enum SelfhostTraitBodyMethodSegmentErrorKind:[\s\S]*EmptyEnvelope[\s\S]*InvalidEnvelope[\s\S]*TokenOutOfBounds[\s\S]*InvalidLayout[\s\S]*UnexpectedTraitBodyItem[\s\S]*MissingMethodBodyIntro[\s\S]*MethodDefaultBodyMissing[\s\S]*MethodSegmentUnavailable[\s\S]*OutOfMemory/,
    "trait method segmentation failures must be typed enum variants",
);
assert.match(
    functionBlock(segmenter, "selfhost_trait_body_method_segment_loop"),
    /TokenKind::KwFn:[\s\S]*selfhost_trait_body_method_segment_from_method_line_result[\s\S]*UnexpectedTraitBodyItem/,
    "trait body segmenter must accept only top-level KwFn method declarations in this slice",
);
assert.match(
    functionBlock(segmenter, "selfhost_trait_body_method_segment_from_method_line_result"),
    /selfhost_trait_body_method_segment_find_colon_loop[\s\S]*MissingMethodBodyIntro[\s\S]*MethodDefaultBodyMissing[\s\S]*selfhost_trait_body_method_segment_new header default_body/,
    "method line conversion must require a colon and a nonempty default body before producing segment evidence",
);
assert.match(
    functionBlock(segmenter, "selfhost_trait_body_method_segment_next_index"),
    /indented[\s\S]*selfhost_trait_body_method_segment_block_body_end_loop[\s\S]*selfhost_trait_body_method_segment_after_closing_dedent[\s\S]*selfhost_trait_body_method_segment_after_separator/,
    "scan continuation must skip the full indented default body instead of re-reading nested body tokens as top-level methods",
);
assert.match(
    functionBlock(segmenter, "selfhost_trait_body_method_segment_block_body_end_loop"),
    /TokenKind::Indent:[\s\S]*add depth 1[\s\S]*TokenKind::Dedent:[\s\S]*sub depth 1/,
    "nested default body scan must track indent/dedent depth",
);
assert.match(
    functionBlock(segmenter, "selfhost_trait_body_method_segment_list_from_envelope"),
    /SelfhostSyntaxRange::Empty:[\s\S]*EmptyEnvelope[\s\S]*not selfhost_syntax_range_is_valid envelope[\s\S]*InvalidEnvelope[\s\S]*TokenOutOfBounds/,
    "public entry must validate envelope shape and token bounds before scanning",
);
assert.doesNotMatch(
    code,
    /\b(?:SelfhostHir|SelfhostHirExprPayload|TypeId|DefId|CallReduce|selfhost_hir_expr_call|selfhost_expr_prefix_list_from_syntax_range)\b/,
    "trait body segmenter must not build HIR, reduce calls, or allocate semantic IDs",
);
assert.doesNotMatch(
    code,
    /str_slice|str_eq|selfhost_token_lexeme|source\s*:|source\s+%str|lexeme|path|display|diagnostic/i,
    "trait body segmentation must not classify or trust source text, lexemes, paths, display names, or diagnostics",
);
assert.doesNotMatch(
    code,
    /hash32|mix\s+xor|fingerprint|public_surface_hash|signature_hash|signature_available\s+true|selfhost_memo_trait_source_identity_new|SelfhostMemoTraitTrustedSourceRegistry|SelfhostMemoTraitDefinitionSourceRecord|SelfhostMemoTraitSourceKind::MemoKeyTrait|SelfhostMemoTraitSourceKind::MemoValueTrait/,
    "trait body segmentation must not produce fingerprints or trusted memo trait source identities",
);
assert.doesNotMatch(
    code,
    /#import "neplg2\/core\/ty|#import "neplg2\/core\/proof|#import "neplg2\/core\/check|#import "neplg2\/core\/hir|#import "neplg2\/core\/lower|#import "neplg2\/core\/resource|#import "neplg2\/core\/backend/,
    "parser-level trait method segmentation must not depend on checker, proof, or type layers",
);
assert.doesNotMatch(
    proofStore,
    /trait_body_segmenter|SelfhostTraitBodyMethodSegment/,
    "proof store must not depend directly on parser trait body method segment evidence",
);
assert.doesNotMatch(
    memoTraitSource,
    /trait_body_segmenter|SelfhostTraitBodyMethodSegment/,
    "memo trait source materialization must not consume raw trait body segment evidence before method signature normalization exists",
);
assert.doesNotMatch(
    memoTraitPolicy,
    /trait_body_segmenter|SelfhostTraitBodyMethodSegment/,
    "memo trait policy must remain independent from parser-level body segmentation evidence",
);
assert.match(
    exprStartPredicate,
    /TokenKind::KwFn:[\s\S]*false/,
    "expression body segmenter should continue to keep KwFn out of expression starts",
);
assert.doesNotMatch(
    code,
    /line count|comment length|file size|500 行/,
    "trait body segmentation policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost trait body segmenter contract passed");

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

const segmenter = read("stdlib/neplg2/core/syntax/parser/body_segmenter.nepl");
const range = read("stdlib/neplg2/core/syntax/parser/range.nepl");
const prefixRange = read("stdlib/neplg2/core/syntax/parser/module_parser/prefix_range.nepl");
const bodyRange = read("stdlib/neplg2/core/syntax/parser/module_parser/body_range.nepl");

const segmenterCode = stripDocComments(segmenter);

assert.doesNotMatch(
    segmenterCode,
    /\b(?:SelfhostHir|SelfhostHirExprPayload|TypeId|DefId|CallReduce|selfhost_hir_expr_call|selfhost_expr_prefix_list_from_syntax_range)\b/,
    "body segmenter must not build HIR, reduce calls, allocate semantic IDs, or feed nested bodies to prefix_expr directly",
);
assert.match(
    segmenter,
    /pub enum SelfhostBodySegmentKind:[\s\S]*ExpressionLine[\s\S]*BlockIntro/,
    "body segmenter must distinguish flat expression lines from block introductions",
);
assert.match(
    segmenter,
    /pub struct SelfhostBodySegment:[\s\S]*\bkind %SelfhostBodySegmentKind[\s\S]*\bhead %SelfhostSyntaxRange[\s\S]*\bbody %SelfhostSyntaxRange/,
    "body segment must carry a typed kind, head range, and nested body range",
);
assert.match(
    functionBlock(segmenter, "selfhost_body_segment_block_intro"),
    /selfhost_body_segment_new SelfhostBodySegmentKind::BlockIntro head body/,
    "block introduction construction must keep head and nested body ranges separate",
);
assert.match(
    functionBlock(segmenter, "selfhost_body_segment_block_body_end_loop"),
    /TokenKind::Indent:[\s\S]*add depth 1[\s\S]*TokenKind::Dedent:[\s\S]*sub depth 1/,
    "nested block scan must track indent/dedent depth",
);
assert.match(
    functionBlock(segmenter, "selfhost_body_segment_skip_separators"),
    /TokenKind::Newline:[\s\S]*TokenKind::Semicolon:/,
    "top-level segment scan must keep newline and semicolon separators out of expression segments",
);
assert.match(
    range,
    /pub fn selfhost_parser_syntax_range_from_indices %fn &Vec SelfhostToken fn i32 fn i32 fn i32 SelfhostSyntaxRange/,
    "common parser range helper must expose token-index to SelfhostSyntaxRange conversion",
);
assert.match(
    prefixRange,
    /pub #import "\.\.\/range" as \*/,
    "module parser prefix range must use the common parser range helper",
);
assert.match(
    bodyRange,
    /#import "\.\.\/range" as \*/,
    "module parser body range must use the common parser range helper",
);

console.log("selfhost body segmenter contract passed");

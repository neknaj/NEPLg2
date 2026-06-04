#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
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

const span = read("stdlib/neplg2/core/infra/span.nepl");
const text = read("stdlib/neplg2/core/infra/text.nepl");
const lineSpan = functionBlock(text, "source_text_line_span");

assert.match(
    span,
    /pub enum SelfhostSourceSpanBuildError:[\s\S]*NegativeFileId[\s\S]*NegativeStart[\s\S]*EndBeforeStart[\s\S]*DifferentFile/,
    "source span construction failures must stay typed",
);
assert.match(
    span,
    /pub fn source_span_new_result %fn i32 fn i32 fn i32 Result SelfhostSourceSpan SelfhostSourceSpanBuildError/,
    "checked source span constructor must return Result",
);
assert.match(
    span,
    /pub fn source_span_empty_result %fn i32 fn i32 Result SelfhostSourceSpan SelfhostSourceSpanBuildError/,
    "checked empty source span constructor must return Result",
);
assert.match(
    span,
    /pub fn source_span_len %fn SelfhostSourceSpan Option i32/,
    "source_span_len must not return negative lengths as plain i32",
);
assert.match(
    functionBlock(span, "source_span_len"),
    /source_span_is_valid\s+span[\s\S]*some\s+sub[\s\S]*else:[\s\S]*none/,
    "source_span_len must reject invalid spans with Option::None",
);
assert.doesNotMatch(
    span,
    /^pub fn source_span_new %fn /m,
    "unchecked construction must not be exposed under the safe source_span_new name",
);
assert.doesNotMatch(
    span,
    /^pub fn source_span_empty %fn /m,
    "unchecked empty construction must not be exposed under the safe source_span_empty name",
);
assert.doesNotMatch(
    span,
    /^pub fn source_span_join %fn /m,
    "unchecked join must not be exposed under the safe source_span_join name",
);
assert.match(
    lineSpan,
    /match\s+source_span_new_result\s+file_id\s+start\s+end:/,
    "source_text_line_span must construct spans through the checked constructor",
);
assert.doesNotMatch(
    lineSpan,
    /source_span_new_unchecked/,
    "source_text_line_span must not bypass checked span construction",
);

console.log("selfhost source span contract passed");

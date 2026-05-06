#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8");
}

function implementation(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}

function lineCount(src) {
    return src.trimEnd().split(/\r?\n/).length;
}

const rootRelPath = "stdlib/alloc/string.nepl";
const utf8RelPath = "stdlib/alloc/string/utf8.nepl";
const storageRelPath = "stdlib/alloc/string/storage.nepl";
const root = read(rootRelPath);
const utf8 = read(utf8RelPath);
const storage = read(storageRelPath);
const rootCode = implementation(root);
const utf8Code = implementation(utf8);
const storageCode = implementation(storage);

assert.match(rootCode, /pub\s+#import\s+"\.\/string\/utf8"\s+as\s+\*/, "alloc/string root must re-export UTF-8 validation APIs");
assert.match(rootCode, /pub\s+#import\s+"\.\/string\/storage"\s+as\s+\*/, "alloc/string root must re-export string storage APIs");
assert.doesNotMatch(rootCode, /fn\s+string_utf8_validate_mem\b/, "alloc/string root must not own raw UTF-8 validation");
assert.doesNotMatch(rootCode, /fn\s+string_utf8_validate_(?:two|three|four)\b/, "alloc/string root must not own UTF-8 sequence validators");
assert.doesNotMatch(rootCode, /fn\s+string_utf8_lead_kind\b/, "alloc/string root must not own UTF-8 leading byte classification");
assert.doesNotMatch(rootCode, /fn\s+string_from_utf8_mem_result\b/, "alloc/string root must not own raw-memory string construction");
assert.match(storageCode, /fn\s+string_from_utf8_mem_result\b[\s\S]*string_utf8_validate_mem\s+src\s+byte_len/, "alloc/string/storage must keep checked str construction at the string ownership boundary");

assert.match(utf8Code, /enum\s+StringUtf8LeadKind:[\s\S]*Ascii[\s\S]*Two[\s\S]*Three[\s\S]*Four[\s\S]*Invalid/, "alloc/string/utf8 must model leading byte categories as an enum");
assert.match(utf8Code, /fn\s+string_utf8_validate_mem\b[\s\S]*match\s+string_utf8_lead_kind\s+b0:[\s\S]*StringUtf8LeadKind::Ascii:[\s\S]*StringUtf8LeadKind::Two:[\s\S]*StringUtf8LeadKind::Three:[\s\S]*StringUtf8LeadKind::Four:[\s\S]*StringUtf8LeadKind::Invalid:/, "alloc/string/utf8 validation must branch with exhaustive enum match");

assert.ok(lineCount(utf8) <= 260, `${utf8RelPath} must stay below 260 lines`);

console.log("alloc/string utf8 boundary regression passed");

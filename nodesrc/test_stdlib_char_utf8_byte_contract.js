#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const charRelPath = "stdlib/core/char.nepl";
const bytebuilderAppendRelPath = "stdlib/alloc/io/bytebuilder/append.nepl";
const charSrc = fs.readFileSync(path.join(repoRoot, charRelPath), "utf8");
const bytebuilderAppendSrc = fs.readFileSync(path.join(repoRoot, bytebuilderAppendRelPath), "utf8");
const charCode = legacyTypeSyntaxView(charSrc);
const bytebuilderAppendCode = legacyTypeSyntaxView(bytebuilderAppendSrc);

function neplFunctionBody(src, name) {
    const re = new RegExp(`(?:pub\\s+)?fn\\s+${name}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?fn\\s+|\\nstruct\\s+|\\nenum\\s+|\\n#|$)`);
    const match = src.match(re);
    assert.ok(match, `${name} must be defined`);
    return match[0];
}

assert.match(
    charCode,
    /\bpub\s+fn\s+char_utf8_byte_at\b[\s\S]*Option<i32>/,
    "char_utf8_byte_at must expose byte absence as Option<i32>",
);
for (const name of ["char_utf8_byte1", "char_utf8_byte2", "char_utf8_byte3"]) {
    assert.doesNotMatch(
        charCode,
        new RegExp(`\\bpub\\s+fn\\s+${name}\\b`),
        `${name} must not be a public precondition-based byte accessor`,
    );
}

const byteAtBody = neplFunctionBody(charCode, "char_utf8_byte_at");
assert.match(byteAtBody, /\bmatch\s+idx:/, "char_utf8_byte_at must branch by byte index");
assert.match(byteAtBody, /\bOption::None\b|\bnone\b/, "char_utf8_byte_at must return None for absent bytes");
assert.match(byteAtBody, /\bchar_utf8_len\b/, "char_utf8_byte_at must derive absence from the encoded length");

assert.doesNotMatch(
    bytebuilderAppendCode,
    /\bchar_utf8_byte[123]\b/,
    "bytebuilder must not call raw UTF-8 tail byte helpers directly",
);
assert.doesNotMatch(
    bytebuilderAppendCode,
    /\bpub\s+fn\s+byte_builder_push_utf8_tail\b/,
    "byte_builder_push_utf8_tail must stay private because it takes an internal length precondition",
);
const pushAtBody = neplFunctionBody(bytebuilderAppendCode, "byte_builder_push_char_utf8_byte_at");
assert.match(
    pushAtBody,
    /\bmatch\s+char_utf8_byte_at\s+c\s+idx:/,
    "bytebuilder must consume char_utf8_byte_at through match",
);
assert.match(
    pushAtBody,
    /\bOption::Some\s+byte:[\s\S]*\bbyte_builder_push_u8\s+builder\s+byte\b/,
    "bytebuilder must write only present UTF-8 bytes",
);
assert.match(
    pushAtBody,
    /\bOption::None:[\s\S]*\bResult::Err\s+ByteBuilderError\s+builder\s+StdErrorKind::InvalidOperation\b/,
    "bytebuilder must preserve the builder owner when an impossible UTF-8 byte is absent",
);

console.log("stdlib char UTF-8 byte contract regression passed");

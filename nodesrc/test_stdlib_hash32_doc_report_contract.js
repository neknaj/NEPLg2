#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function source(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function precedingDoc(code, declarationNeedle) {
    const index = code.indexOf(declarationNeedle);
    assert.notEqual(index, -1, `missing declaration: ${declarationNeedle}`);
    const before = code.slice(0, index).split("\n");
    const doc = [];
    let cursor = before.length - 1;
    while (cursor >= 0 && before[cursor].trim() === "") {
        cursor -= 1;
    }
    while (cursor >= 0 && before[cursor].trimStart().startsWith("//:")) {
        doc.push(before[cursor]);
        cursor -= 1;
    }
    return doc.reverse().join("\n");
}

function assertIncludes(code, needle, message) {
    assert.ok(code.includes(needle), message);
}

function assertReportDoc(code, declarationNeedle, reportName) {
    const doc = precedingDoc(code, declarationNeedle);
    assertIncludes(doc, "neplg2:test[stdio, normalize_newlines]", `${reportName} must use stdout-normalized doctest metadata`);
    assertIncludes(doc, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
    assertIncludes(doc, "### [契約/けいやく]", `${reportName} must document stable contract`);
    assertIncludes(doc, "### [現在/げんざい]の[実装/じっそう]", `${reportName} must separate current implementation notes`);
    assertIncludes(doc, "### [計算量/けいさんりょう]", `${reportName} must document complexity`);
    return doc;
}

const hash32 = source("stdlib/alloc/hash/hash32.nepl");
const fnv = source("stdlib/alloc/hash/fnv1a32.nepl");
const sha256Api = source("stdlib/alloc/hash/sha256/api.nepl");

for (const [declaration, reportName] of [
    ["pub fn mix %fn i32 i32 \\x:", "hash32_mix_doc"],
    ["pub fn hash_bytes_loop %fn str fn i32 fn i32 fn Fnv1a32 Fnv1a32 \\s\\n\\idx\\h:", "hash32_hash_bytes_loop_doc"],
    ["pub fn hash32 %fn bool i32 \\key:", "hash32_bool_doc"],
    ["pub fn hash32 %fn i32 i32 \\key:", "hash32_i32_doc"],
    ["pub fn hash32 %fn u8 i32 \\key:", "hash32_u8_doc"],
    ["pub fn hash32 %fn i64 i32 \\key:", "hash32_i64_doc"],
    ["pub fn hash32 %fn str i32 \\s:", "hash32_str_doc"],
]) {
    assertReportDoc(hash32, declaration, reportName);
}

for (const [declaration, reportName] of [
    ["pub struct Fnv1a32:", "fnv1a32_struct_doc"],
    ["pub fn new_fnv1a32 %fn void Fnv1a32 \\void:", "new_fnv1a32_doc"],
    ["pub fn fnv1a32_update %fn Fnv1a32 fn i32 Fnv1a32 \\h\\byte:", "fnv1a32_update_doc"],
    ["pub fn fnv1a32_finalize %fn Fnv1a32 i32 \\h:", "fnv1a32_finalize_doc"],
]) {
    assertReportDoc(fnv, declaration, reportName);
}

assertReportDoc(sha256Api, "pub fn sha256_free %impure fn Sha256 unit \\ctx:", "sha256_free_doc");

for (const [code, reportName] of [
    [hash32, "hash32_module_doc"],
    [fnv, "fnv1a32_module_doc"],
]) {
    assertIncludes(code, `test_report_new "${reportName}"`, `${reportName} module doctest is missing`);
}

for (const snippet of [
    "signed `i32`",
    "32-bit bit pattern",
    "UTF-8 byte",
    "`str` overload は文字数ではなく UTF-8 byte",
    "`Option::Some b`",
    "`Option::None`",
    "`checked_string_byte_at s idx`",
    "`idx >= n`",
    "O(n - idx)",
    "MurmurHash3",
    "下位",
    "上位",
    "衝突",
]) {
    assertIncludes(hash32, snippet, `hash32 docs must pin hash, string, Option, and fold contracts: ${snippet}`);
}

for (const snippet of [
    "offset basis `0x811c9dc5`",
    "prime `16777619`",
    "0..255",
    "signed `i32`",
    "32-bit bit pattern",
    "`Fnv1a32` value として明示的に",
    "global state や platform API",
    "O(1)",
]) {
    assertIncludes(fnv, snippet, `fnv1a32 docs must pin state and byte contract: ${snippet}`);
}

for (const snippet of [
    "buffer 解放は副作用境界なので、この helper は `impure fn`",
    "`ctx` owner を消費し",
    "`Sha256.buffer` field",
    "sha256_free ctx",
]) {
    assertIncludes(sha256Api, snippet, `sha256_free docs must pin owner-closing impure boundary: ${snippet}`);
}

assert.doesNotMatch(
    hash32,
    /#import\s+"alloc\/string"\s+as\s+string/,
    "hash32 must keep using the narrow alloc/string/access import for string length",
);

console.log("stdlib hash32 doc report contract passed");

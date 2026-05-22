#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/ringbuffer.nepl";
const apiPath = "stdlib/alloc/collections/ringbuffer/api.nepl";
const rootSrc = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
const apiSrc = fs.readFileSync(path.join(repoRoot, apiPath), "utf8");

const rootCode = rootSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");
const code = apiSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(rootCode, /pub\s+#import\s+"\.\/ringbuffer\/api"\s+as\s+@merge/, "RingBuffer root must re-export API from a submodule");
assert.doesNotMatch(rootCode, /\bfn\s+/, "RingBuffer root facade must not keep observer bodies");

for (const [name, resultTy] of [
    ["len", "i32"],
    ["cap", "i32"],
    ["is_empty", "bool"],
]) {
    assert.match(
        code,
        new RegExp(`fn\\s+${name}\\s+<\\.T>\\s+<\\(&RingBuffer<\\.T>\\)->${resultTy}>\\s+\\(rb\\):`),
        `RingBuffer.${name} must borrow the owner and not require Copy for metadata-only observation`,
    );
    assert.doesNotMatch(
        code,
        new RegExp(`fn\\s+${name}\\s+<\\.T>\\s+<\\(RingBuffer<\\.T>\\)->${resultTy}>`),
        `RingBuffer.${name} must not consume the owner`,
    );
}

assert.match(code, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&RingBuffer<\.T>\)->Option<\.T>>\s+\(rb\):/, "RingBuffer.peek must borrow the owner");
assert.doesNotMatch(code, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(RingBuffer<\.T>\)->Option<\.T>>/, "RingBuffer.peek must not consume the owner");
assert.doesNotMatch(code, /fn\s+(?:len_ref|cap_ref|is_empty_ref|peek_ref)\b/, "RingBuffer must not keep duplicate *_ref observer surfaces");

for (const testPath of [
    "stdlib/tests/ringbuffer.n.md",
    "tests/stdlib/ringbuffer_collections.n.md",
    "tests/stdlib/pipe_collections.n.md",
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /len<i32>\s+&rb[0-9]?\b/, `${testPath} must exercise borrowed RingBuffer.len`);
    assert.match(testSrc, /free<i32>\s+rb[0-9]?\b/, `${testPath} must explicitly free observed RingBuffer owners`);
    assert.doesNotMatch(testSrc, /\b(?:len_ref|cap_ref|is_empty_ref|peek_ref)<i32>/, `${testPath} must not use removed RingBuffer *_ref observers`);
    assert.doesNotMatch(testSrc, /(?:len|peek)<i32>\s+rb[0-9]?\b/, `${testPath} must not call by-value RingBuffer observers`);
    assert.doesNotMatch(testSrc, /\brb[0-9]?\s+\|>\s+peek(?:<i32>)?\b/, `${testPath} must not pipe RingBuffer owners into peek`);
}

const pipeCollections = fs.readFileSync(path.join(repoRoot, "tests/stdlib/pipe_collections.n.md"), "utf8");
assert.match(pipeCollections, /peek<i32>\s+&rb2\b/, "pipe_collections must borrow RingBuffer.peek explicitly");

console.log("ringbuffer borrowed observer regression passed");

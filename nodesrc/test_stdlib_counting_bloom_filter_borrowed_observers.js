#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/counting_bloom_filter.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(
    code,
    /fn\s+len\s+<\.T,\.H>\s+<\(&CountingBloomFilter<\.T,\.H>\)->i32>\s+\(bf\):/,
    "CountingBloomFilter.len must borrow the owner",
);
assert.doesNotMatch(code, /fn\s+len\s+<\.T,\.H>\s+<\(CountingBloomFilter<\.T,\.H>\)->i32>/, "CountingBloomFilter.len must not consume the owner");

assert.match(
    code,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&CountingBloomFilter<\.T,\.H>,\.T\)->bool>\s+\(bf,\s*key\):/,
    "CountingBloomFilter.contains must borrow the owner",
);
assert.doesNotMatch(
    code,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(CountingBloomFilter<\.T,\.H>,\.T\)->bool>/,
    "CountingBloomFilter.contains must not consume the owner",
);

for (const testPath of ["stdlib/tests/counting_bloom_filter.n.md", "tests/stdlib/counting_bloom_filter_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /contains(?:<[^>]+>)?\s+&bf/, `${testPath} must exercise borrowed CountingBloomFilter.contains`);
    assert.match(testSrc, /len\s+&bf/, `${testPath} must exercise borrowed CountingBloomFilter.len`);
    assert.match(testSrc, /free\s+bf/, `${testPath} must explicitly free observed CountingBloomFilter owners`);
    assert.doesNotMatch(testSrc, /contains(?:<[^>]+>)?\s+bf[0-9]?\b/, `${testPath} must not call by-value CountingBloomFilter.contains`);
    assert.doesNotMatch(testSrc, /len\s+bf[0-9]?\b/, `${testPath} must not call by-value CountingBloomFilter.len`);
}

console.log("counting bloom filter borrowed observer regression passed");

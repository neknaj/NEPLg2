#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/alloc/collections/bloom_filter/api.nepl";
const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

assert.match(
    code,
    /fn\s+len\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&BloomFilter<\.T,\.H>\)->i32>\s+\(bf\):/,
    "BloomFilter.len must borrow the owner and remain Copy-only while drop traversal is incomplete",
);
assert.doesNotMatch(code, /fn\s+len\s+<[^>]+>\s+<\(BloomFilter<\.T,\.H>\)->i32>/, "BloomFilter.len must not consume the owner");

assert.match(
    code,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(&BloomFilter<\.T,\.H>,\.T\)->bool>\s+\(bf,\s*key\):/,
    "BloomFilter.contains must borrow the owner",
);
assert.doesNotMatch(
    code,
    /fn\s+contains\s+<\.T:\s*HashKey&Copy,\.H:\s*Hasher<\.T>&Copy>\s+<\(BloomFilter<\.T,\.H>,\.T\)->bool>/,
    "BloomFilter.contains must not consume the owner",
);

for (const testPath of ["stdlib/tests/bloom_filter.n.md", "tests/stdlib/bloom_filter_collections.n.md"]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), "utf8");
    assert.match(testSrc, /contains(?:<[^>]+>)?\s+&bf/, `${testPath} must exercise borrowed BloomFilter.contains`);
    assert.match(testSrc, /len\s+&bf/, `${testPath} must exercise borrowed BloomFilter.len`);
    assert.match(testSrc, /free\s+bf/, `${testPath} must explicitly free observed BloomFilter owners`);
    assert.doesNotMatch(testSrc, /contains(?:<[^>]+>)?\s+bf[0-9]?\b/, `${testPath} must not call by-value BloomFilter.contains`);
    assert.doesNotMatch(testSrc, /len\s+bf[0-9]?\b/, `${testPath} must not call by-value BloomFilter.len`);
}

console.log("bloom filter borrowed observer regression passed");

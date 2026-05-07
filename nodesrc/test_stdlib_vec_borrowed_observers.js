#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const vecRelPaths = [
    "stdlib/alloc/collections/vec.nepl",
    "stdlib/alloc/collections/vec/types.nepl",
    "stdlib/alloc/collections/vec/storage.nepl",
    "stdlib/alloc/collections/vec/access.nepl",
];
const vecSources = Object.fromEntries(vecRelPaths.map((relPath) => [relPath, fs.readFileSync(path.join(repoRoot, relPath), "utf8")]));
const vecCode = vecRelPaths
    .map((relPath) => vecSources[relPath]
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n"))
    .join("\n");

assert.match(vecSources["stdlib/alloc/collections/vec.nepl"], /#import\s+"\.\/vec\/access"\s+as\s+vec_access/, "Vec root must delegate borrowed observers to access module");

for (const name of [
    "len_ref",
    "cap_ref",
    "is_empty_ref",
    "get_ref",
    "data_ptr_ref",
    "data_mem_ptr_ref",
    "data_len_ref",
    "replace_ref",
]) {
    assert.doesNotMatch(vecCode, new RegExp(`fn\\s+${name}\\b`), `Vec must not keep duplicate ${name} observer surface`);
}

for (const [name, resultTy] of [
    ["len", "i32"],
    ["cap", "i32"],
    ["is_empty", "bool"],
    ["data_ptr", "i32"],
    ["data_mem_ptr", "MemPtr<\\.T>"],
    ["data_len", "VecDataLen<\\.T>"],
]) {
    assert.match(
        vecCode,
        new RegExp(`fn\\s+${name}\\s+<\\.T>\\s+<\\(&Vec<\\.T>\\)->${resultTy}>\\s+\\(v\\):`),
        `Vec.${name} must borrow the owner`,
    );
    assert.doesNotMatch(
        vecCode,
        new RegExp(`fn\\s+${name}\\s+<\\.T>\\s+<\\(Vec<\\.T>\\)->${resultTy}>`),
        `Vec.${name} must not consume the owner`,
    );
}

assert.match(vecCode, /fn\s+get\s+<\.T:\s*Copy>\s+<\(&Vec<\.T>,i32\)->Option<\.T>>\s+\(v,\s*idx\):/, "Vec.get must borrow the owner and require Copy");
assert.doesNotMatch(vecCode, /fn\s+get\s+<\.T>\s+<\(Vec<\.T>,i32\)->Option<\.T>>/, "Vec.get must not consume the owner");
assert.match(vecCode, /fn\s+replace\s+<\.T:\s*Copy>\s+<\(&Vec<\.T>,i32,\.T\)\*>\(\)>\s+\(v,\s*idx,\s*item\):/, "Vec.replace must borrow the owner and require Copy");
assert.doesNotMatch(vecCode, /fn\s+replace\s+<\.T>\s+<\(Vec<\.T>,i32,\.T\)->\(\)>/, "Vec.replace must not consume the owner");

for (const [name, signature] of [
    ["count", "<\\(&Vec<\\.T>, \\(\\.T\\)->bool\\)->i32>"],
    ["fold", "<\\(&Vec<\\.T>, \\.U, \\(\\.U,\\.T\\)->\\.U\\)->\\.U>"],
    ["reduce", "<\\(&Vec<\\.T>, \\(\\.T,\\.T\\)->\\.T\\)->Option<\\.T>>"],
    ["find", "<\\(&Vec<\\.T>, \\(\\.T\\)->bool\\)->Option<\\.T>>"],
    ["any", "<\\(&Vec<\\.T>, \\(\\.T\\)->bool\\)->bool>"],
    ["all", "<\\(&Vec<\\.T>, \\(\\.T\\)->bool\\)->bool>"],
]) {
    assert.match(
        vecCode,
        new RegExp(`fn\\s+${name}\\s+<\\.T:\\s*Copy[\\s\\S]*?${signature}\\s+\\(`),
        `Vec.${name} must traverse through a borrowed owner and copy values explicitly`,
    );
}

assert.match(vecCode, /struct\s+VecPop<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*item\s+<Option<\.T>>/, "Vec.pop must return a named owner-bearing result");
assert.match(vecCode, /fn\s+pop\s+<\.T>\s+<\(Vec<\.T>\)->VecPop<\.T>>/, "Vec.pop must not return an untyped Pair");
assert.match(vecCode, /struct\s+VecPartition<\.T>:[\s\S]*matched\s+<Vec<\.T>>[\s\S]*rest\s+<Vec<\.T>>/, "Vec.partition must return named owner-bearing fields");
assert.match(vecCode, /fn\s+partition\s+<\.T>\s+<\(Vec<\.T>, \(\.T\)->bool\)->Result<VecPartition<\.T>, StdErrorKind>>/, "Vec.partition must not return an untyped Pair");
assert.doesNotMatch(vecCode, /->\.Pair\b/, "Vec must not expose owner-bearing results as .Pair");

for (const testRelPath of [
    "stdlib/tests/vec.n.md",
    "tests/stdlib/vec_collections.n.md",
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testRelPath), "utf8");
    assert.doesNotMatch(testSrc, /\b(?:len_ref|cap_ref|is_empty_ref|get_ref|data_ptr_ref|data_mem_ptr_ref|data_len_ref|replace_ref)<i32>/, `${testRelPath} must not use removed Vec *_ref observers`);
    assert.match(testSrc, /\blen<i32>\s+&/, `${testRelPath} must exercise borrowed Vec.len`);
    assert.match(testSrc, /\bget<i32>\s+&/, `${testRelPath} must exercise borrowed Vec.get`);
    assert.match(testSrc, /\bfree<i32>\s+/, `${testRelPath} must explicitly free observed Vec owners`);
}

for (const relPath of [
    "examples/bf.nepl",
    "stdlib/alloc/collections/hashmap.nepl",
    "stdlib/alloc/collections/hashset.nepl",
    "stdlib/alloc/collections/vec/sort.nepl",
    "stdlib/alloc/collections/vec/sort/common.nepl",
    "stdlib/alloc/collections/vec/sort/simple.nepl",
    "stdlib/alloc/collections/vec/sort/quick.nepl",
    "stdlib/alloc/collections/vec/sort/heap.nepl",
    "stdlib/alloc/collections/vec/sort/merge.nepl",
    "tests/stdlib/selfhost_req.n.md",
]) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(src, /\b(?:v|vec)::(?:len_ref|cap_ref|is_empty_ref|get_ref|data_ptr_ref|data_mem_ptr_ref|data_len_ref|replace_ref)\b/, `${relPath} must not call removed Vec *_ref APIs`);
}

console.log("vec borrowed observer regression passed");

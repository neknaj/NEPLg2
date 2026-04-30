#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8");
}

const facade = read("stdlib/core/math.nepl");
const u8Module = read("stdlib/core/math/u8.nepl");

assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/u8"\s+as\s+\*/,
    "core/math.nepl must re-export the u8 math submodule",
);

for (const fnName of [
    "add_u8",
    "sub_u8",
    "mul_u8",
    "div_u_u8",
    "rem_u_u8",
    "eq_u8",
    "ne_u8",
    "lt_u_u8",
    "le_u_u8",
    "gt_u_u8",
    "ge_u_u8",
]) {
    assert.match(u8Module, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math/u8.nepl must define ${fnName}`);
    assert.doesNotMatch(facade, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math.nepl must not keep ${fnName}`);
}

for (const [name, signature] of [
    ["add", "<\\(u8,u8\\)->u8>"],
    ["sub", "<\\(u8,u8\\)->u8>"],
    ["mul", "<\\(u8,u8\\)->u8>"],
    ["div_u", "<\\(u8,u8\\)->u8>"],
    ["rem_u", "<\\(u8,u8\\)->u8>"],
    ["eq", "<\\(u8,u8\\)->bool>"],
    ["ne", "<\\(u8,u8\\)->bool>"],
    ["lt_u", "<\\(u8,u8\\)->bool>"],
    ["le_u", "<\\(u8,u8\\)->bool>"],
    ["gt_u", "<\\(u8,u8\\)->bool>"],
    ["ge_u", "<\\(u8,u8\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(u8Module, pattern, `core/math/u8.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

assert.match(facade, /\bfn\s+add\s+<\(i32,i32\)->i32>/, "core/math.nepl must keep i32 math implementation for now");
assert.match(facade, /\bfn\s+add\s+<\(i64,i64\)->i64>/, "core/math.nepl must keep i64 math implementation for now");

console.log("stdlib math module split regression passed");

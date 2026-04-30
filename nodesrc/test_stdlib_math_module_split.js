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
const i32Module = read("stdlib/core/math/i32.nepl");
const i64Module = read("stdlib/core/math/i64.nepl");
const f32Module = read("stdlib/core/math/f32.nepl");
const u8Module = read("stdlib/core/math/u8.nepl");
const boolModule = read("stdlib/core/math/bool.nepl");

assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/i32"\s+as\s+\*/,
    "core/math.nepl must re-export the i32 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/i64"\s+as\s+\*/,
    "core/math.nepl must re-export the i64 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/f32"\s+as\s+\*/,
    "core/math.nepl must re-export the f32 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/u8"\s+as\s+\*/,
    "core/math.nepl must re-export the u8 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/bool"\s+as\s+\*/,
    "core/math.nepl must re-export the bool math submodule",
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

for (const [name, signature] of [
    ["add", "<\\(i64,i64\\)->i64>"],
    ["sub", "<\\(i64,i64\\)->i64>"],
    ["mul", "<\\(i64,i64\\)->i64>"],
    ["div_s", "<\\(i64,i64\\)->i64>"],
    ["div_u", "<\\(i64,i64\\)->i64>"],
    ["rem_s", "<\\(i64,i64\\)->i64>"],
    ["rem_u", "<\\(i64,i64\\)->i64>"],
    ["and", "<\\(i64,i64\\)->i64>"],
    ["or", "<\\(i64,i64\\)->i64>"],
    ["xor", "<\\(i64,i64\\)->i64>"],
    ["shl", "<\\(i64,i64\\)->i64>"],
    ["shr_s", "<\\(i64,i64\\)->i64>"],
    ["shr_u", "<\\(i64,i64\\)->i64>"],
    ["rotl", "<\\(i64,i64\\)->i64>"],
    ["rotr", "<\\(i64,i64\\)->i64>"],
    ["eq", "<\\(i64,i64\\)->bool>"],
    ["ne", "<\\(i64,i64\\)->bool>"],
    ["lt", "<\\(i64,i64\\)->bool>"],
    ["lt_u", "<\\(i64,i64\\)->bool>"],
    ["le", "<\\(i64,i64\\)->bool>"],
    ["le_u", "<\\(i64,i64\\)->bool>"],
    ["gt", "<\\(i64,i64\\)->bool>"],
    ["gt_u", "<\\(i64,i64\\)->bool>"],
    ["ge", "<\\(i64,i64\\)->bool>"],
    ["ge_u", "<\\(i64,i64\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64Module, pattern, `core/math/i64.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["clz", "<\\(i64\\)->i64>"],
    ["ctz", "<\\(i64\\)->i64>"],
    ["popcnt", "<\\(i64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64Module, pattern, `core/math/i64.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(f32,f32\\)->f32>"],
    ["sub", "<\\(f32,f32\\)->f32>"],
    ["mul", "<\\(f32,f32\\)->f32>"],
    ["div", "<\\(f32,f32\\)->f32>"],
    ["min", "<\\(f32,f32\\)->f32>"],
    ["max", "<\\(f32,f32\\)->f32>"],
    ["copysign", "<\\(f32,f32\\)->f32>"],
    ["eq", "<\\(f32,f32\\)->bool>"],
    ["ne", "<\\(f32,f32\\)->bool>"],
    ["lt", "<\\(f32,f32\\)->bool>"],
    ["le", "<\\(f32,f32\\)->bool>"],
    ["gt", "<\\(f32,f32\\)->bool>"],
    ["ge", "<\\(f32,f32\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f32Module, pattern, `core/math/f32.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["sqrt", "<\\(f32\\)->f32>"],
    ["abs", "<\\(f32\\)->f32>"],
    ["neg", "<\\(f32\\)->f32>"],
    ["ceil", "<\\(f32\\)->f32>"],
    ["floor", "<\\(f32\\)->f32>"],
    ["trunc", "<\\(f32\\)->f32>"],
    ["nearest", "<\\(f32\\)->f32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f32Module, pattern, `core/math/f32.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["and", "<\\(bool,bool\\)->bool>"],
    ["or", "<\\(bool,bool\\)->bool>"],
    ["not", "<\\(bool\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(boolModule, pattern, `core/math/bool.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(i32,i32\\)->i32>"],
    ["sub", "<\\(i32,i32\\)->i32>"],
    ["mul", "<\\(i32,i32\\)->i32>"],
    ["div_s", "<\\(i32,i32\\)->i32>"],
    ["div_u", "<\\(i32,i32\\)->i32>"],
    ["rem_s", "<\\(i32,i32\\)->i32>"],
    ["mod_s", "<\\(i32,i32\\)->i32>"],
    ["rem_u", "<\\(i32,i32\\)->i32>"],
    ["and", "<\\(i32,i32\\)->i32>"],
    ["or", "<\\(i32,i32\\)->i32>"],
    ["xor", "<\\(i32,i32\\)->i32>"],
    ["shl", "<\\(i32,i32\\)->i32>"],
    ["shr_s", "<\\(i32,i32\\)->i32>"],
    ["shr_u", "<\\(i32,i32\\)->i32>"],
    ["rotl", "<\\(i32,i32\\)->i32>"],
    ["rotr", "<\\(i32,i32\\)->i32>"],
    ["eq", "<\\(i32,i32\\)->bool>"],
    ["ne", "<\\(i32,i32\\)->bool>"],
    ["lt", "<\\(i32,i32\\)->bool>"],
    ["lt_u", "<\\(i32,i32\\)->bool>"],
    ["le", "<\\(i32,i32\\)->bool>"],
    ["le_u", "<\\(i32,i32\\)->bool>"],
    ["gt", "<\\(i32,i32\\)->bool>"],
    ["gt_u", "<\\(i32,i32\\)->bool>"],
    ["ge", "<\\(i32,i32\\)->bool>"],
    ["ge_u", "<\\(i32,i32\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32Module, pattern, `core/math/i32.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["clz", "<\\(i32\\)->i32>"],
    ["ctz", "<\\(i32\\)->i32>"],
    ["popcnt", "<\\(i32\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32Module, pattern, `core/math/i32.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

assert.match(facade, /\bfn\s+add\s+<\(f64,f64\)->f64>/, "core/math.nepl must keep f64 math implementation for now");

console.log("stdlib math module split regression passed");

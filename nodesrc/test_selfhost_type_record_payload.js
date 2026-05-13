#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const tyPath = "stdlib/neplg2/core/ty/ty.nepl";
const ty = fs.readFileSync(path.join(repoRoot, tyPath), "utf8").replace(/\r\n/g, "\n");

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const declaration = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${escaped}\\s+`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${escaped}`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
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

function enumVariants(src, enumName) {
    return topLevelBlock(src, "enum", `${enumName}:`)
        .split("\n")
        .slice(1)
        .map((line) => /^    ([A-Za-z][A-Za-z0-9_]*)(?:\s+<[^>]+>)?$/.exec(line))
        .filter(Boolean)
        .map((match) => match[1]);
}

function assertRecordAccessorMatches(fnName) {
    const block = topLevelBlock(ty, "fn", fnName);
    assert.match(block, /\bmatch\s+record:/, `${fnName} must match SelfhostTypeRecord before reading payload`);
    assert.doesNotMatch(block, /\brecord\.(?:kind|first_arg|arg_count|result)\b/, `${fnName} must not read old flat record fields`);
    assert.match(block, /^\s*SelfhostTypeRecord::Primitive\b/m, `${fnName} must handle primitive records`);
    assert.match(block, /^\s*SelfhostTypeRecord::Function\b/m, `${fnName} must handle function records`);
}

assert.doesNotMatch(ty, /^(?:pub\s+)?struct SelfhostTypeRecord:/m, "SelfhostTypeRecord must not be a flat struct");
assert.doesNotMatch(ty, /\b(?:pub\s+)?fn\s+selfhost_type_id_invalid\b/, "type records must not need an invalid TypeId constructor");
assert.doesNotMatch(ty, /\b(?:pub\s+)?fn\s+selfhost_type_record_new\b/, "flat type record constructor must not be exposed");
assert.doesNotMatch(ty, /\bselfhost_type_record_primitive\b[\s\S]{0,120}\s-1\b/, "primitive type records must not store a negative argument range");
assert.doesNotMatch(ty, /\bSelfhostTypeRecord\s+kind\s+first_arg\s+arg_count\s+result\b/, "old flat type record construction must not return");

assert.deepEqual(
    enumVariants(ty, "SelfhostTypeRecord"),
    ["Primitive", "Function"],
    "SelfhostTypeRecord must split primitive and function payloads",
);
assert.deepEqual(
    enumVariants(ty, "SelfhostPrimitiveTypeKind"),
    ["Error", "Unit", "Bool", "I32", "I64", "U8", "Char", "Str", "F32", "F64", "Never"],
    "primitive type payload must exclude Function",
);
assert.match(ty, /(?:pub\s+)?struct SelfhostFunctionTypeRecord:[\s\S]*?\bfirst_arg\s+<i32>[\s\S]*?\barg_count\s+<i32>[\s\S]*?\bresult\s+<SelfhostTypeId>/, "function payload must own function-only fields");
assert.match(ty, /\bSelfhostTypeRecord::Primitive\s+kind\b/, "primitive record constructor must use the primitive payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Function\s+SelfhostFunctionTypeRecord\s+first_arg\s+arg_count\s+result\b/, "function record constructor must use the function payload variant");

assertRecordAccessorMatches("selfhost_type_arena_get_kind");
assertRecordAccessorMatches("selfhost_type_arena_function_arg_count");
assertRecordAccessorMatches("selfhost_type_arena_function_arg");
assertRecordAccessorMatches("selfhost_type_arena_function_result");

const recordsEqual = topLevelBlock(ty, "fn", "selfhost_type_arena_records_equal");
assert.match(recordsEqual, /\bmatch\s+a:/, "record equality must dispatch on the left record variant");
assert.match(recordsEqual, /\bmatch\s+b:/, "record equality must dispatch on the right record variant");
assert.doesNotMatch(recordsEqual, /\b[ab]\.(?:kind|first_arg|arg_count|result)\b/, "record equality must not read old flat fields");
for (const variant of ["Primitive", "Function"]) {
    assert.match(recordsEqual, new RegExp(`^\\s*SelfhostTypeRecord::${variant}\\b`, "m"), `record equality must handle ${variant}`);
}

console.log("selfhost type record payload regression passed");

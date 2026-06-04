#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readTySource } = require("./selfhost_ty_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const ty = legacyTypeSyntaxView(readTySource(repoRoot));

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
    enumVariants(ty, "SelfhostFunctionTypeArgRange"),
    ["Empty", "Range"],
    "function argument range must split zero-argument and nonempty payloads",
);
assert.deepEqual(
    enumVariants(ty, "SelfhostFunctionTypeArgRangeBuildError"),
    ["NegativeFirst", "NegativeCount", "NonCanonicalEmpty", "EndOverflow", "OutOfBounds"],
    "function argument range constructor errors must be typed and exhaustive",
);
assert.deepEqual(
    enumVariants(ty, "SelfhostPrimitiveTypeKind"),
    ["Error", "Unit", "Bool", "I32", "I64", "U8", "Char", "Str", "F32", "F64", "Never"],
    "primitive type payload must exclude Function",
);
assert.match(ty, /(?:pub\s+)?struct SelfhostFunctionTypeArgRangeItems:[\s\S]*?\bfirst_arg\s+<i32>[\s\S]*?\barg_count\s+<i32>/, "nonempty function argument range payload must own table fields");
assert.match(ty, /(?:pub\s+)?struct SelfhostFunctionTypeRecord:[\s\S]*?\bargs\s+<SelfhostFunctionTypeArgRange>[\s\S]*?\bresult\s+<SelfhostTypeId>/, "function payload must own a typed argument range and result");
assert.match(ty, /\bSelfhostTypeRecord::Primitive\s+kind\b/, "primitive record constructor must use the primitive payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Function\s+SelfhostFunctionTypeRecord\s+args\s+result\b/, "function record constructor must use the function payload variant");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_new_result\b[\s\S]{0,220}Result<SelfhostFunctionTypeArgRange,SelfhostFunctionTypeArgRangeBuildError>/, "function argument range must expose a checked constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_new_bounded_result\b[\s\S]{0,260}Result<SelfhostFunctionTypeArgRange,SelfhostFunctionTypeArgRangeBuildError>/, "function argument range must expose a table-bounded checked constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_is_valid\b[\s\S]*bool/, "function argument range must expose a validity predicate for defensive equality");

assertRecordAccessorMatches("selfhost_type_arena_get_kind");
assertRecordAccessorMatches("selfhost_type_arena_function_arg_count");
assertRecordAccessorMatches("selfhost_type_arena_function_arg");
assertRecordAccessorMatches("selfhost_type_arena_function_result");

const recordsEqual = topLevelBlock(ty, "fn", "selfhost_type_arena_records_equal");
assert.match(recordsEqual, /\bmatch\s+a:/, "record equality must dispatch on the left record variant");
assert.match(recordsEqual, /\bmatch\s+b:/, "record equality must dispatch on the right record variant");
assert.doesNotMatch(recordsEqual, /\b[ab]\.(?:kind|first_arg|arg_count|result)\b/, "record equality must not read old flat fields");
assert.match(ty, /\bselfhost_type_arena_function_arg_ranges_equal\b[\s\S]*selfhost_function_type_arg_range_is_valid/, "function type equality must reject invalid range payloads before raw recursion");
for (const variant of ["Primitive", "Function"]) {
    assert.match(recordsEqual, new RegExp(`^\\s*SelfhostTypeRecord::${variant}\\b`, "m"), `record equality must handle ${variant}`);
}

console.log("selfhost type record payload regression passed");

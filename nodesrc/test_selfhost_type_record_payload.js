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
    assert.match(block, /^\s*SelfhostTypeRecord::Named\b/m, `${fnName} must handle named records`);
    assert.match(block, /^\s*SelfhostTypeRecord::Parameter\b/m, `${fnName} must handle type parameter records`);
    assert.match(block, /^\s*SelfhostTypeRecord::Applied\b/m, `${fnName} must handle applied records`);
    assert.match(block, /^\s*SelfhostTypeRecord::Function\b/m, `${fnName} must handle function records`);
}

assert.doesNotMatch(ty, /^(?:pub\s+)?struct SelfhostTypeRecord:/m, "SelfhostTypeRecord must not be a flat struct");
assert.doesNotMatch(ty, /\b(?:pub\s+)?fn\s+selfhost_type_id_invalid\b/, "type records must not need an invalid TypeId constructor");
assert.doesNotMatch(ty, /\b(?:pub\s+)?fn\s+selfhost_type_record_new\b/, "flat type record constructor must not be exposed");
assert.doesNotMatch(ty, /\bselfhost_type_record_primitive\b[\s\S]{0,120}\s-1\b/, "primitive type records must not store a negative argument range");
assert.doesNotMatch(ty, /\bSelfhostTypeRecord\s+kind\s+first_arg\s+arg_count\s+result\b/, "old flat type record construction must not return");

assert.deepEqual(
    enumVariants(ty, "SelfhostTypeRecord"),
    ["Primitive", "Named", "Parameter", "Applied", "Function"],
    "SelfhostTypeRecord must split primitive, named, type parameter, applied, and function payloads",
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
assert.match(ty, /(?:pub\s+)?struct SelfhostNamedTypeRecord:[\s\S]*?\bnominal_id\s+<SelfhostNamedTypeId>/, "named payload must own a nominal type identity");
assert.match(ty, /(?:pub\s+)?struct SelfhostTypeParameterRecord:[\s\S]*?\bbinding\s+<SelfhostTypeParameterBinding>/, "type parameter payload must own a binder-indexed identity");
assert.match(ty, /(?:pub\s+)?struct SelfhostAppliedTypeArgRange:[\s\S]*?\bfirst_arg\s+<i32>[\s\S]*?\barg_count\s+<i32>/, "applied type argument range payload must own table fields");
assert.match(ty, /(?:pub\s+)?struct SelfhostAppliedTypeRecord:[\s\S]*?\bnominal_id\s+<SelfhostNamedTypeId>[\s\S]*?\bargs\s+<SelfhostAppliedTypeArgRange>/, "applied payload must own a nominal identity and typed argument range");
assert.match(ty, /\bSelfhostTypeRecord::Primitive\s+kind\b/, "primitive record constructor must use the primitive payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Named\s+SelfhostNamedTypeRecord\s+nominal_id\b/, "named record constructor must use the named payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Parameter\s+SelfhostTypeParameterRecord\s+binding\b/, "type parameter record constructor must use the parameter payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Applied\s+SelfhostAppliedTypeRecord\s+nominal_id\s+args\b/, "applied record constructor must use the applied payload variant");
assert.match(ty, /\bSelfhostTypeRecord::Function\s+SelfhostFunctionTypeRecord\s+args\s+result\b/, "function record constructor must use the function payload variant");
assert.match(ty, /\bpub\s+fn\s+selfhost_named_type_record_id\b[\s\S]{0,160}SelfhostNamedTypeId\b/, "named type record must expose its nominal identity through an accessor");
assert.match(ty, /\bpub\s+fn\s+selfhost_type_parameter_record_binding\b[\s\S]{0,180}SelfhostTypeParameterBinding\b/, "type parameter record must expose its binder identity through an accessor");
assert.match(ty, /\bpub\s+fn\s+selfhost_applied_type_arg_range_is_valid\b[\s\S]*bool/, "applied argument range must expose a validity predicate for defensive equality");
assert.match(ty, /\bpub\s+fn\s+selfhost_type_record_applied\b[\s\S]{0,180}SelfhostTypeRecord\b/, "applied type records must have a dedicated constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_type_record_parameter\b[\s\S]{0,220}SelfhostTypeRecord\b/, "type parameter records must have a dedicated constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_applied_type_record_id\b[\s\S]{0,180}SelfhostNamedTypeId\b/, "applied type record must expose its constructor identity through an accessor");
assert.match(ty, /\bpub\s+fn\s+selfhost_applied_type_record_args\b[\s\S]{0,180}SelfhostAppliedTypeArgRange\b/, "applied type record must expose its argument range through an accessor");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_new_result\b[\s\S]{0,220}Result<SelfhostFunctionTypeArgRange,SelfhostFunctionTypeArgRangeBuildError>/, "function argument range must expose a checked constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_new_bounded_result\b[\s\S]{0,260}Result<SelfhostFunctionTypeArgRange,SelfhostFunctionTypeArgRangeBuildError>/, "function argument range must expose a table-bounded checked constructor");
assert.match(ty, /\bpub\s+fn\s+selfhost_function_type_arg_range_is_valid\b[\s\S]*bool/, "function argument range must expose a validity predicate for defensive equality");

assertRecordAccessorMatches("selfhost_type_arena_get_kind");
assertRecordAccessorMatches("selfhost_type_arena_function_arg_count");
assertRecordAccessorMatches("selfhost_type_arena_function_arg");
assertRecordAccessorMatches("selfhost_type_arena_function_result");
assertRecordAccessorMatches("selfhost_type_arena_named_id");
assertRecordAccessorMatches("selfhost_type_arena_type_parameter_binding");
assertRecordAccessorMatches("selfhost_type_arena_applied_constructor_id");
assertRecordAccessorMatches("selfhost_type_arena_applied_arg_count");
assertRecordAccessorMatches("selfhost_type_arena_applied_arg");

const recordsEqual = topLevelBlock(ty, "fn", "selfhost_type_arena_records_equal");
assert.match(recordsEqual, /\bmatch\s+a:/, "record equality must dispatch on the left record variant");
assert.match(recordsEqual, /\bmatch\s+b:/, "record equality must dispatch on the right record variant");
assert.doesNotMatch(recordsEqual, /\b[ab]\.(?:kind|first_arg|arg_count|result)\b/, "record equality must not read old flat fields");
assert.match(ty, /\bselfhost_type_arena_function_arg_ranges_equal\b[\s\S]*selfhost_function_type_arg_range_is_valid/, "function type equality must reject invalid range payloads before raw recursion");
assert.match(ty, /\bselfhost_type_arena_applied_arg_ranges_equal\b[\s\S]*selfhost_applied_type_arg_range_is_valid/, "applied type equality must reject invalid range payloads before raw recursion");
assert.match(recordsEqual, /\bselfhost_type_parameter_binding_eq\b/, "record equality must compare type parameter records by binder identity");
for (const variant of ["Primitive", "Named", "Parameter", "Applied", "Function"]) {
    assert.match(recordsEqual, new RegExp(`^\\s*SelfhostTypeRecord::${variant}\\b`, "m"), `record equality must handle ${variant}`);
}

console.log("selfhost type record payload regression passed");

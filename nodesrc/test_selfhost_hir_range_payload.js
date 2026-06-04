#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readHirSource } = require("./selfhost_hir_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const hir = legacyTypeSyntaxView(readHirSource(repoRoot));

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const decl = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}`);
    const start = lines.findIndex((line) => decl.test(line));
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

function assertRangeAccessorMatches(fnName, enumName) {
    const block = topLevelBlock(hir, "fn", fnName);
    const localName = enumName === "SelfhostHirChildRange" ? "child_range" : "param_range";
    assert.match(block, new RegExp(`\\bmatch\\s+${localName}:`), `${fnName} must match ${enumName}`);
    assert.doesNotMatch(block, new RegExp(`\\b${localName}\\.(?:first_child|child_count|first_param|param_count)\\b`), `${fnName} must not read flat range fields`);
    assert.match(block, new RegExp(`^\\s*${enumName}::Empty\\b`, "m"), `${fnName} must handle Empty`);
    assert.match(block, new RegExp(`^\\s*${enumName}::Range\\b`, "m"), `${fnName} must handle Range`);
}

assert.doesNotMatch(hir, /^(?:pub\s+)?struct SelfhostHirChildRange:$/m, "child range must not be a flat struct");
assert.doesNotMatch(hir, /^(?:pub\s+)?struct SelfhostHirParamRange:$/m, "param range must not be a flat struct");
assert.deepEqual(enumVariants(hir, "SelfhostHirChildRange"), ["Empty", "Range"], "child range must split empty and nonempty payloads");
assert.deepEqual(enumVariants(hir, "SelfhostHirParamRange"), ["Empty", "Range"], "param range must split empty and nonempty payloads");
assert.deepEqual(
    enumVariants(hir, "SelfhostHirRangeBuildError"),
    ["NegativeFirst", "NegativeCount", "NonCanonicalEmpty", "EndOverflow", "OutOfBounds"],
    "HIR range constructor errors must be typed and exhaustive",
);
assert.match(hir, /(?:pub\s+)?struct SelfhostHirChildRangeItems:[\s\S]*?\bfirst_child\s+<i32>[\s\S]*?\bchild_count\s+<i32>/, "child range payload must own child table fields");
assert.match(hir, /(?:pub\s+)?struct SelfhostHirParamRangeItems:[\s\S]*?\bfirst_param\s+<i32>[\s\S]*?\bparam_count\s+<i32>/, "param range payload must own param table fields");
assert.doesNotMatch(hir, /\bpub\s+fn\s+selfhost_hir_child_range_new\s+/, "unchecked child range constructor must not keep the old public name");
assert.doesNotMatch(hir, /\bpub\s+fn\s+selfhost_hir_param_range_new\s+/, "unchecked param range constructor must not keep the old public name");
assert.match(hir, /\bpub\s+fn\s+selfhost_hir_child_range_new_result\b[\s\S]{0,180}Result<SelfhostHirChildRange,SelfhostHirRangeBuildError>/, "child range must expose a checked constructor");
assert.match(hir, /\bpub\s+fn\s+selfhost_hir_param_range_new_result\b[\s\S]{0,180}Result<SelfhostHirParamRange,SelfhostHirRangeBuildError>/, "param range must expose a checked constructor");
assert.match(hir, /\bpub\s+fn\s+selfhost_hir_child_range_new_bounded_result\b[\s\S]{0,220}Result<SelfhostHirChildRange,SelfhostHirRangeBuildError>/, "child range must expose a table-bounded checked constructor");
assert.match(hir, /\bpub\s+fn\s+selfhost_hir_param_range_new_bounded_result\b[\s\S]{0,220}Result<SelfhostHirParamRange,SelfhostHirRangeBuildError>/, "param range must expose a table-bounded checked constructor");
assert.doesNotMatch(hir, /\bselfhost_hir_child_range_new_unchecked\s+-1\s+0\b/, "empty child range must not be a negative sentinel");
assert.doesNotMatch(hir, /\bselfhost_hir_param_range_new_unchecked\s+-1\s+0\b/, "empty param range must not be a negative sentinel");
assert.doesNotMatch(hir, /\b(?:child_range|param_range)\.(?:first_child|child_count|first_param|param_count)\b/, "callers must use range accessors or match variants");
assert.match(hir, /pub\s+struct\s+SelfhostHirFunction:[\s\S]*?\bparams\s+<SelfhostHirParamRange>/, "HIR function record must store a typed parameter range");
assert.doesNotMatch(hir, /pub\s+struct\s+SelfhostHirFunction:[\s\S]*?\bfirst_param\s+<i32>[\s\S]*?\bparam_count\s+<i32>/, "HIR function record must not store raw parameter range fields");

assertRangeAccessorMatches("selfhost_hir_child_range_first", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_child_range_count", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_param_range_first", "SelfhostHirParamRange");
assertRangeAccessorMatches("selfhost_hir_param_range_count", "SelfhostHirParamRange");
assertRangeAccessorMatches("selfhost_hir_module_get_child", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_module_get_param", "SelfhostHirParamRange");

console.log("selfhost HIR range payload regression passed");

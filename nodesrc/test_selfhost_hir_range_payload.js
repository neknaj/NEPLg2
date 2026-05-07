#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const hirPath = "stdlib/neplg2/core/hir/hir.nepl";
const hir = fs.readFileSync(path.join(repoRoot, hirPath), "utf8").replace(/\r\n/g, "\n");

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const prefix = kind === "fn" ? `fn ${name} ` : `${kind} ${name}`;
    const start = lines.findIndex((line) => line.startsWith(prefix));
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

assert.doesNotMatch(hir, /^struct SelfhostHirChildRange:$/m, "child range must not be a flat struct");
assert.doesNotMatch(hir, /^struct SelfhostHirParamRange:$/m, "param range must not be a flat struct");
assert.deepEqual(enumVariants(hir, "SelfhostHirChildRange"), ["Empty", "Range"], "child range must split empty and nonempty payloads");
assert.deepEqual(enumVariants(hir, "SelfhostHirParamRange"), ["Empty", "Range"], "param range must split empty and nonempty payloads");
assert.match(hir, /struct SelfhostHirChildRangeItems:[\s\S]*?\bfirst_child\s+<i32>[\s\S]*?\bchild_count\s+<i32>/, "child range payload must own child table fields");
assert.match(hir, /struct SelfhostHirParamRangeItems:[\s\S]*?\bfirst_param\s+<i32>[\s\S]*?\bparam_count\s+<i32>/, "param range payload must own param table fields");
assert.doesNotMatch(hir, /\bselfhost_hir_child_range_new\s+-1\s+0\b/, "empty child range must not be a negative sentinel");
assert.doesNotMatch(hir, /\bselfhost_hir_param_range_new\s+-1\s+0\b/, "empty param range must not be a negative sentinel");
assert.doesNotMatch(hir, /\b(?:child_range|param_range)\.(?:first_child|child_count|first_param|param_count)\b/, "callers must use range accessors or match variants");

assertRangeAccessorMatches("selfhost_hir_child_range_first", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_child_range_count", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_param_range_first", "SelfhostHirParamRange");
assertRangeAccessorMatches("selfhost_hir_param_range_count", "SelfhostHirParamRange");
assertRangeAccessorMatches("selfhost_hir_module_get_child", "SelfhostHirChildRange");
assertRangeAccessorMatches("selfhost_hir_module_get_param", "SelfhostHirParamRange");

console.log("selfhost HIR range payload regression passed");

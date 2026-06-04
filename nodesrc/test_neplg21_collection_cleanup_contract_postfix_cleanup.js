#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "tests/stdlib/collection_cleanup_contract.n.md";
const filePath = path.join(repoRoot, relPath);
const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);

const oldPostfixCalls = [
    ["Vec cleanup operation", /\b(?:clear|free|new|with_capacity|push|get)<CleanupPayload>/],
    ["Vec proof or owner accessor", /\b(?:vec_current_copy_invariant|vec_pop_vec|vec_push_error_vec|vec_transform_error_vec|vec_sort_error_vec)<CleanupPayload>/],
    ["binary heap owner accessor", /\bbinary_heap_pop_heap<CleanupPayload>/],
    ["linear collection cleanup", /\bfree<CleanupPayload>/],
    ["map/set cleanup", /\bfree<(?:CleanupPayload,\s*i32|i32,\s*CleanupPayload|i32,\s*CleanupPayload,\s*DefaultHash32|NonCopyHashKey,\s*DefaultHash32|i32,\s*StatefulHasher|NonCopyHashKey,\s*i32,\s*DefaultHash32|i32,\s*i32,\s*StatefulHasher)>/],
    ["map insert cleanup", /\binsert<(?:i32,\s*CleanupPayload|i32,\s*CleanupPayload,\s*DefaultHash32)>/],
    ["list transform owner accessor", /\blist_transform_error_list<CleanupPayload>/],
    ["storage borrowed view", /\b(?:btreemap_storage_keys|btreemap_storage_values|btreeset_storage_keys|hashmap_storage_keys|hashmap_storage_values|hashset_storage_keys)<[^>\r\n]+>/],
    ["bloom cleanup", /\b(?:free|clear)<i32,\s*StatefulHasher>/],
];

const violations = [];
let inNeplFence = false;

lines.forEach((line, index) => {
    const trimmed = line.trim();
    if (trimmed === "```neplg2") {
        inNeplFence = true;
        return;
    }
    if (trimmed === "```") {
        inNeplFence = false;
        return;
    }
    if (!inNeplFence || trimmed.startsWith("//")) {
        return;
    }
    for (const [label, pattern] of oldPostfixCalls) {
        if (pattern.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${label}: ${trimmed}`);
        }
    }
});

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 collection cleanup contract fixtures must use receiver, argument, or result type evidence instead of selected generic postfix calls:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 collection cleanup contract postfix cleanup regression passed");

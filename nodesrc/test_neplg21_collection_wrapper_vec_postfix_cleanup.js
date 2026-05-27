#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const targetFiles = [
    "stdlib/alloc/collections/list/basic.nepl",
    "stdlib/alloc/collections/list/query.nepl",
    "stdlib/alloc/collections/list/storage.nepl",
    "stdlib/alloc/collections/list/transform.nepl",
    "stdlib/alloc/collections/stack/storage.nepl",
    "stdlib/alloc/collections/queue/storage.nepl",
    "stdlib/alloc/collections/queue/api.nepl",
    "stdlib/alloc/collections/ringbuffer/storage.nepl",
    "stdlib/alloc/collections/ringbuffer/api.nepl",
    "stdlib/alloc/collections/deque/storage.nepl",
    "stdlib/alloc/collections/deque/api.nepl",
    "stdlib/alloc/collections/binary_heap/storage.nepl",
    "stdlib/alloc/collections/binary_heap/api/push.nepl",
    "stdlib/alloc/collections/binary_heap/api/cleanup.nepl",
    "stdlib/alloc/collections/btreeset/storage.nepl",
    "stdlib/alloc/collections/btreemap/storage.nepl",
    "stdlib/alloc/collections/hashset/storage.nepl",
    "stdlib/alloc/collections/hashmap/storage.nepl",
];

const oldVecHelperCall = /\bvec::[A-Za-z_][A-Za-z0-9_]*</;
const violations = [];

for (const relPath of targetFiles) {
    const filePath = path.join(repoRoot, relPath);
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
        if (oldVecHelperCall.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 collection wrapper Vec cleanup must not reintroduce old helper postfix calls:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 collection wrapper Vec postfix cleanup regression passed");

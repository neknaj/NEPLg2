#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const targetRoots = [
    "stdlib/alloc/collections",
    "stdlib/alloc/io",
    "stdlib/alloc/string",
    "stdlib/core",
    "stdlib/nm",
    "stdlib/std",
    "stdlib/tests",
    "tests/stdlib",
];

const targetFiles = [
    "stdlib/neplg2/README.md",
];

const typeLikeOldApplication =
    /\b(?:Vec|Result|Option|MemPtr|RegionToken|OwnedBuffer|VecStorage|VecPushRejected|VecPushError|VecReplaceRejected|VecReplaceError|VecTransformError|VecPop|VecPartition|BinaryHeap|BinaryHeapPushError|BinaryHeapPop|Deque|DequePushError|DequePop|Queue|QueuePushError|QueuePop|RingBuffer|RingBufferPushError|RingBufferPop|Stack|StackPushError|StackPop|List|ListPushError|ListTransformError|HashMap|HashMapStorage|HashMapUpdateError|HashSet|HashSetStorage|HashSetUpdateError|BTreeMap|BTreeMapStorage|BTreeMapInsertError|BTreeSet|BTreeSetStorage|BTreeSetInsertError|BloomFilter|CountingBloomFilter|ByteBufStorage|ByteBuilderStorage|Diags|Diag|Outcome)<[^>]+>/;

function walk(dir, out) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walk(full, out);
        } else if (entry.isFile() && (entry.name.endsWith(".nepl") || entry.name.endsWith(".n.md"))) {
            out.push(full);
        }
    }
}

function isFence(line) {
    return /^\s*(?:(?:\/\/:)\s*)?```/.test(line);
}

function proseText(line, inFence, relPath) {
    if (inFence) {
        return null;
    }
    if (/^\s*\/\/:\|/.test(line)) {
        return null;
    }
    const docMatch = line.match(/^\s*\/\/:\s?(.*)$/);
    if (docMatch) {
        return docMatch[1];
    }
    if (relPath.endsWith(".nepl")) {
        return null;
    }
    if (!line.trim() || /^\s*```/.test(line)) {
        return null;
    }
    return line;
}

const files = [];
for (const relRoot of targetRoots) {
    walk(path.join(repoRoot, relRoot), files);
}
for (const relFile of targetFiles) {
    files.push(path.join(repoRoot, relFile));
}

const violations = [];

for (const filePath of files) {
    const relPath = path.relative(repoRoot, filePath).replaceAll(path.sep, "/");
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    let inFence = false;
    lines.forEach((line, index) => {
        if (isFence(line)) {
            inFence = !inFence;
            return;
        }
        const text = proseText(line, inFence, relPath);
        if (text && typeLikeOldApplication.test(text)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 prose type notation cleanup must not reintroduce angle-bracket type prose in migrated docs:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 prose type notation cleanup regression passed");

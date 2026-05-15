#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const internalMemPath = path.join(repoRoot, "stdlib", "core", "mem", "internal.nepl");
const source = fs.readFileSync(internalMemPath, "utf8").replace(/\r\n/g, "\n");

const regionNewDoc = extractDocBlockBeforeFunction(source, "region_new");

assertIncludes(
    regionNewDoc,
    "match allocator::alloc 4:",
    "region_new doctest must derive its MemPtr from allocator-issued storage",
);
assertIncludes(
    regionNewDoc,
    "match dealloc_region<u8> token:",
    "region_new doctest must consume the owner token it constructs",
);
assertNotMatches(
    regionNewDoc,
    /\bregion_new\s+(?:<[^>\n]+>\s*)?mem_ptr_wrap\b/,
    "region_new doctest must not construct owner tokens directly from fixed raw addresses",
);
assertNotMatches(
    regionNewDoc,
    /\bmem_ptr_wrap\s+(?!0\b)\d+\b/,
    "region_new doctest must not demonstrate fixed non-zero raw address wrapping",
);

console.log("stdlib core/mem internal region_new doctest policy passed");

function extractDocBlockBeforeFunction(text, functionName) {
    const fnPattern = new RegExp(`\\n(?:pub\\s+)?fn\\s+${functionName}\\b`);
    const fnMatch = fnPattern.exec(text);
    assert(fnMatch, `${functionName} function must exist`);

    const before = text.slice(0, fnMatch.index);
    const lines = before.split("\n");
    const docLines = [];
    for (let i = lines.length - 1; i >= 0; i -= 1) {
        const line = lines[i];
        if (line.startsWith("//:")) {
            docLines.push(line);
            continue;
        }
        if (line.trim() === "") {
            continue;
        }
        break;
    }
    docLines.reverse();
    const block = docLines.join("\n");
    assert(block.length > 0, `${functionName} doc block must exist`);
    return block;
}

function assertIncludes(haystack, needle, message) {
    assert(haystack.includes(needle), `${message}: missing ${JSON.stringify(needle)}`);
}

function assertNotMatches(haystack, pattern, message) {
    assert(!pattern.test(haystack), message);
}

function assert(condition, message) {
    if (condition) {
        return;
    }
    console.error(message);
    process.exit(1);
}

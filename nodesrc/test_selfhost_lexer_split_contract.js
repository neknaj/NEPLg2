#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const facadePath = "stdlib/neplg2/core/syntax/lexer.nepl";
const splitDir = "stdlib/neplg2/core/syntax/lexer";
const expectedModules = [
    "diagnostic",
    "byte",
    "literal",
    "token_build",
    "indent",
    "directive",
    "keyword",
    "raw_mode",
    "next",
    "error",
    "tokenize",
];

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const facade = read(facadePath);

for (const moduleName of expectedModules) {
    assert.match(
        facade,
        new RegExp(`pub\\s+#import\\s+"\\.\\/lexer\\/${moduleName}"\\s+as\\s+\\*`),
        `lexer facade must re-export ${moduleName}`,
    );
}

assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/m,
    "lexer facade must not own implementation declarations",
);

const splitFiles = fs.readdirSync(path.join(repoRoot, splitDir)).filter((name) => name.endsWith(".nepl"));
assert.deepEqual(
    splitFiles.sort(),
    expectedModules.map((moduleName) => `${moduleName}.nepl`).sort(),
    "lexer split modules changed without updating the split contract",
);

for (const file of splitFiles) {
    const rel = `${splitDir}/${file}`;
    const src = read(rel);
    assert.doesNotMatch(
        src,
        /#import\s+"neplg2\/core\/syntax\/lexer"\s+as\s+\*/,
        `${rel} must import precise peer modules instead of the facade`,
    );
}

console.log("selfhost lexer split contract regression passed");

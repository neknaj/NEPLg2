#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function readRepoFile(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8");
}

const preludeBase = readRepoFile("stdlib/std/prelude_base.nepl");
const copyFacade = readRepoFile("stdlib/core/traits/copy.nepl");
const primitiveCopy = readRepoFile("stdlib/core/traits/copy/primitive.nepl");

assert.match(
    preludeBase,
    /^#import "core\/traits\/copy" as @merge$/m,
    "default prelude must import the full Copy facade so MemPtr Copy remains available",
);
assert.doesNotMatch(
    preludeBase,
    /core\/traits\/copy\/primitive/,
    "default prelude must not bypass MemPtr Copy by importing only primitive Copy",
);
assert.match(
    copyFacade,
    /^#import "core\/mem\/types" as \*$/m,
    "Copy facade must depend only on mem/types for MemPtr impls",
);
assert.doesNotMatch(
    copyFacade,
    /^#import "core\/mem" as \*$/m,
    "Copy facade must not pull the allocator-oriented core/mem facade into default prelude",
);
assert.doesNotMatch(
    primitiveCopy,
    /^#import "core\/mem(?:\/|")/m,
    "primitive Copy module must stay independent from memory wrapper modules",
);

console.log("prelude Copy dependency policy passed");

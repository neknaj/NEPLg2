#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    PRELUDE_FACADE,
    PRELUDE_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_prelude_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, PRELUDE_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, PRELUDE_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "prelude facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "prelude facade must not contain implementation declarations",
);

for (const relPath of PRELUDE_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/builtins\/prelude\//, "./prelude/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${PRELUDE_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    const lineCount = source.trimEnd().split("\n").length;
    assert.ok(lineCount <= 450, `${relPath} must stay below the prelude split line budget, got ${lineCount}`);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/prelude" as \*|#import "neplg2\/core\/builtins\/prelude" as \*/,
        `${relPath} must not import the prelude facade`,
    );
}

console.log("selfhost prelude split contract passed");

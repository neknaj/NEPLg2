#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    DIAG_FACADE,
    DIAG_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_diag_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, DIAG_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, DIAG_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "diag facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "diag facade must not contain implementation declarations",
);

for (const relPath of DIAG_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/infra\/diag\//, "./diag/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${DIAG_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/diag" as \*|#import "neplg2\/core\/infra\/diag" as \*/,
        `${relPath} must not import the diag facade`,
    );
}

console.log("selfhost diag split contract passed");

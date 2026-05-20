#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    TY_FACADE,
    TY_ROOT_REEXPORT_FILES,
    TY_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, TY_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, TY_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "ty facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "ty facade must not contain implementation declarations",
);

for (const relPath of TY_ROOT_REEXPORT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/ty\/ty\//, "./ty/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${TY_FACADE} must re-export ${importPath}`,
    );
}

const kindFacadePath = "stdlib/neplg2/core/ty/ty/kind.nepl";
const kindFacade = readRepoFile(repoRoot, kindFacadePath);
assert.doesNotMatch(
    kindFacade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "ty kind facade must not contain implementation declarations",
);
for (const importPath of ["./kind/model", "./kind/eq", "./kind/name"]) {
    assert.match(
        kindFacade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${kindFacadePath} must re-export ${importPath}`,
    );
}

for (const relPath of TY_SPLIT_FILES) {
    const source = readRepoFile(repoRoot, relPath);
    const lineCount = source.trimEnd().split("\n").length;
    assert.ok(lineCount <= 450, `${relPath} must stay below the type split line budget, got ${lineCount}`);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/ty" as \*|#import "\.\.\/\.\.\/ty" as \*|#import "neplg2\/core\/ty\/ty" as \*/,
        `${relPath} must not import the ty facade`,
    );
}

console.log("selfhost ty split contract passed");

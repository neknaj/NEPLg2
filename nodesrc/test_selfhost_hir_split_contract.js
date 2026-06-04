#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    HIR_FACADE,
    HIR_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_hir_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, HIR_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, HIR_FACADE));

assert.equal(parsedFacade.doctests.length, 3, "HIR facade must keep the public doctests");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "HIR facade must not contain implementation declarations",
);

for (const relPath of HIR_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/hir\/hir\//, "./hir/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${HIR_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    assert.doesNotMatch(source, /#import "\.\.\/hir" as \*|#import "neplg2\/core\/hir\/hir" as \*/, `${relPath} must not import the facade`);
}

console.log("selfhost HIR split contract passed");

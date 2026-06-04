#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    TYPE_RESOLVER_FACADE,
    TYPE_RESOLVER_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_type_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, TYPE_RESOLVER_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, TYPE_RESOLVER_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "type resolver facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "type resolver facade must not contain implementation declarations",
);

for (const relPath of TYPE_RESOLVER_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/resolve\/type_resolver\//, "./type_resolver/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${TYPE_RESOLVER_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/type_resolver" as \*|#import "neplg2\/core\/resolve\/type_resolver" as \*/,
        `${relPath} must not import the type resolver facade`,
    );
}

console.log("selfhost type resolver split contract passed");

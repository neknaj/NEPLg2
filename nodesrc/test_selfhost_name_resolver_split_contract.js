#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    NAME_RESOLVER_FACADE,
    NAME_RESOLVER_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_name_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, NAME_RESOLVER_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, NAME_RESOLVER_FACADE));

assert.equal(parsedFacade.doctests.length, 2, "name resolver facade must keep the public doctests");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "name resolver facade must not contain implementation declarations",
);

for (const relPath of NAME_RESOLVER_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/resolve\/name_resolver\//, "./name_resolver/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${NAME_RESOLVER_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    const lineCount = source.trimEnd().split("\n").length;
    assert.ok(lineCount <= 450, `${relPath} must stay below the name resolver split line budget, got ${lineCount}`);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/name_resolver" as \*|#import "neplg2\/core\/resolve\/name_resolver" as \*/,
        `${relPath} must not import the name resolver facade`,
    );
}

console.log("selfhost name resolver split contract passed");

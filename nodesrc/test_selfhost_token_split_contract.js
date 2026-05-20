#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    TOKEN_FACADE,
    TOKEN_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_token_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, TOKEN_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, TOKEN_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "token facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "token facade must not contain implementation declarations",
);

for (const relPath of TOKEN_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/syntax\/token\//, "./token/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${TOKEN_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    const lineCount = source.trimEnd().split("\n").length;
    assert.ok(lineCount <= 450, `${relPath} must stay below the token split line budget, got ${lineCount}`);
    assert.doesNotMatch(source, /#import "\.\.\/token" as \*|#import "neplg2\/core\/syntax\/token" as \*/, `${relPath} must not import the facade`);
}

console.log("selfhost token split contract passed");

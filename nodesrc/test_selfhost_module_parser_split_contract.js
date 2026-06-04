#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const {
    MODULE_PARSER_FACADE,
    MODULE_PARSER_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_module_parser_sources");

const repoRoot = path.resolve(__dirname, "..");
const facade = readRepoFile(repoRoot, MODULE_PARSER_FACADE);
const parsedFacade = parseFile(path.join(repoRoot, MODULE_PARSER_FACADE));

assert.equal(parsedFacade.doctests.length, 1, "module parser facade must keep the public doctest");
assert.doesNotMatch(
    facade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "module parser facade must not contain implementation declarations",
);

for (const relPath of MODULE_PARSER_SPLIT_FILES) {
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/syntax\/parser\/module_parser\//, "./module_parser/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${importPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}" as \\*$`, "m"),
        `${MODULE_PARSER_FACADE} must re-export ${importPath}`,
    );

    const source = readRepoFile(repoRoot, relPath);
    assert.doesNotMatch(
        source,
        /#import "\.\.\/module_parser" as \*|#import "neplg2\/core\/syntax\/parser\/module_parser" as \*/,
        `${relPath} must not import the module parser facade`,
    );
}

console.log("selfhost module parser split contract passed");

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

const reservedNames = ["void", "fn", "impure", "unit", "bool", "i32", "i64", "u8", "char", "str", "f32", "f64", "never"];
for (const [relPath, functionName] of [
    ["stdlib/neplg2/core/resolve/type_resolver/constructor.nepl", "selfhost_type_constructor_name_is_reserved"],
    ["stdlib/neplg2/core/resolve/type_resolver/typeparam/env.nepl", "selfhost_type_parameter_name_is_reserved"],
]) {
    const source = readRepoFile(repoRoot, relPath);
    const escaped = functionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const body = source.match(new RegExp(`^fn ${escaped}\\b[\\s\\S]*?(?=\\n(?:pub )?fn |\\n(?:pub )?(?:struct|enum|impl) |(?![\\s\\S]))`, "m"))?.[0] ?? "";
    assert.ok(body, `${relPath} must define ${functionName}`);
    const comparedNames = [...body.matchAll(/string_search::str_eq name "([^"]+)"/g)].map((match) => match[1]);
    assert.deepEqual(comparedNames, reservedNames, `${functionName} must compare exactly the reserved-name set once and in canonical order`);
    assert.equal((body.match(/\bor:/g) ?? []).length, reservedNames.length - 1, `${functionName} must compose the pure comparisons with exactly twelve eager or calls`);
    assert.doesNotMatch(body, /\b(?:if:|match\s)/, `${functionName} must not lower its fixed pure set to Resource control branches`);
}

console.log("selfhost type resolver split contract passed");

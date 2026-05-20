#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { readTySource } = require("./selfhost_ty_sources");
const { readNameResolverSource } = require("./selfhost_name_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function enumVariants(src, enumName) {
    const lines = src.split("\n");
    const start = lines.findIndex((line) => line === `enum ${enumName}:` || line === `pub enum ${enumName}:`);
    assert.notEqual(start, -1, `${enumName} enum not found`);
    const variants = [];
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            break;
        }
        const match = /^    ([A-Za-z][A-Za-z0-9_]*)$/.exec(lines[i]);
        if (match) {
            variants.push(match[1]);
        }
    }
    assert.ok(variants.length > 0, `${enumName} variants must be discovered`);
    return variants;
}

function assertEnumEqUsesMatchesSource(src, label, enumName, eqName) {
    assert.doesNotMatch(
        src,
        new RegExp(`(?:pub\\s+)?fn\\s+${eqName.replace(/_eq$/, "_tag")}\\b`),
        `${label} must not expose a numeric tag helper for ${enumName}`,
    );
    assert.doesNotMatch(src, /\bselfhost_[A-Za-z0-9_]+_kind_tag\b/, `${label} must not use kind tag helpers`);

    const block = functionBlock(src, eqName);
    assert.match(block, /\bmatch\s+a:/, `${eqName} must dispatch on the left enum value`);
    assert.match(block, /\bmatch\s+b:/, `${eqName} must dispatch on the right enum value`);
    assert.doesNotMatch(block, /^\s*_:/m, `${eqName} must not use wildcard arms`);
    assert.doesNotMatch(block, /\beq\s+selfhost_[A-Za-z0-9_]+_kind_tag\b/, `${eqName} must not compare numeric tags`);

    for (const variant of enumVariants(src, enumName)) {
        assert.match(
            block,
            new RegExp(`^\\s*${enumName}::${variant}:\\s*$`, "m"),
            `${eqName} is missing ${enumName}::${variant}`,
        );
    }
}

function assertEnumEqUsesMatches(rel, enumName, eqName) {
    assertEnumEqUsesMatchesSource(read(rel), rel, enumName, eqName);
}

assertEnumEqUsesMatchesSource(readTySource(repoRoot), "stdlib/neplg2/core/ty/ty", "SelfhostTypeKind", "selfhost_type_kind_eq");
assertEnumEqUsesMatches("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprKind", "selfhost_hir_expr_kind_eq");
assertEnumEqUsesMatches("stdlib/neplg2/core/builtins/prelude.nepl", "SelfhostBuiltinKind", "selfhost_builtin_kind_eq");
assertEnumEqUsesMatchesSource(readNameResolverSource(repoRoot), "stdlib/neplg2/core/resolve/name_resolver", "SelfhostDefKind", "selfhost_def_kind_eq");

console.log("selfhost model numeric kind tag regression passed");

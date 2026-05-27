#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const targetFiles = [
    "stdlib/neplg2/cli/driver.nepl",
    "stdlib/neplg2/cli/args/options.nepl",
    "stdlib/neplg2/cli/args/parse.nepl",
    "stdlib/neplg2/core/hir/hir.nepl",
    "stdlib/neplg2/core/hir/hir/arena.nepl",
    "stdlib/neplg2/core/ty/ty/arena.nepl",
    "stdlib/neplg2/core/module/import_spec.nepl",
    "stdlib/neplg2/core/module/import_scan.nepl",
    "stdlib/neplg2/core/module/graph.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/scope.nepl",
    "stdlib/neplg2/core/infra/diag/collection.nepl",
    "stdlib/neplg2/core/syntax/ast/module_ast.nepl",
    "stdlib/std/fs/dir/read_fd.nepl",
    "stdlib/std/fs/path/normalize.nepl",
    "stdlib/std/fs/path/normalize/range_stack.nepl",
    "stdlib/std/fs/path/entry.nepl",
];

const oldVecAliasHelperCall = /\bv::[A-Za-z_][A-Za-z0-9_]*</;
const violations = [];

for (const relPath of targetFiles) {
    const filePath = path.join(repoRoot, relPath);
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
        if (oldVecAliasHelperCall.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 selfhost/stdlib Vec alias cleanup must not reintroduce old v:: helper postfix calls:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 selfhost/stdlib Vec alias postfix cleanup regression passed");

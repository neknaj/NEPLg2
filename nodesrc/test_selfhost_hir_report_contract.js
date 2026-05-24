#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");
const { HIR_FACADE, readHirSource } = require("./selfhost_hir_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const relPath = HIR_FACADE;
const file = path.join(repoRoot, relPath);
const source = legacyTypeSyntaxView(readHirSource(repoRoot));
const parsed = parseFile(file);

const expectedCheckCounts = [6, 7, 7];
const accessors = [
    "selfhost_hir_module_expr_alloc_expr_id",
    "selfhost_hir_module_expr_alloc_into_module",
    "selfhost_hir_module_function_alloc_function_id",
    "selfhost_hir_module_function_alloc_into_module",
    "selfhost_hir_module_child_range_alloc_child_range",
    "selfhost_hir_module_child_range_alloc_into_module",
    "selfhost_hir_module_param_range_alloc_param_range",
    "selfhost_hir_module_param_range_alloc_into_module",
];

assert.equal(parsed.doctests.length, expectedCheckCounts.length, "selfhost HIR doctest count changed");

for (const name of accessors) {
    assert.match(source, new RegExp(`\\bpub fn ${name}\\b`), `${name} accessor must stay public`);
}

assert.match(
    source,
    /pub fn selfhost_hir_module_expr_alloc_expr_id <\(&SelfhostHirModuleExprAlloc\)->SelfhostHirExprId>/,
    "expr allocation id accessor must borrow the wrapper and return only the Copy id",
);
assert.match(
    source,
    /pub fn selfhost_hir_module_expr_alloc_into_module <\(SelfhostHirModuleExprAlloc\)\*>SelfhostHirModule>/,
    "expr allocation module accessor must consume the wrapper",
);

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

parsed.doctests.forEach((doctest, index) => {
    const name = `selfhost HIR doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(
        doctest.stdout,
        expectedStdout(expectedCheckCounts[index]),
        `${name} must pin the std/test report stdout`,
    );
    assert.doesNotMatch(
        doctest.code,
        /\bfield::get\b/,
        `${name} must use public HIR allocation accessors, not owner-backed fields`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `${name} must print the report before returning its exit code`,
    );
});

console.log("selfhost HIR report contract passed");

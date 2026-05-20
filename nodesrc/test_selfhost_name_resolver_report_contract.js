#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");
const { readNameResolverSource } = require("./selfhost_name_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/resolve/name_resolver.nepl";
const file = path.join(repoRoot, relPath);
const source = readNameResolverSource(repoRoot);
const parsed = parseFile(file);

const expectedCheckCounts = [5, 3];

assert.equal(parsed.doctests.length, expectedCheckCounts.length, "selfhost name resolver doctest count changed");
assert.match(
    source,
    /\bpub fn selfhost_name_scope_add_result_def_id <\(&SelfhostNameScopeAddResult\)->SelfhostDefId>/,
    "name scope add-result DefId accessor must borrow the wrapper and return only the Copy id",
);
assert.match(
    source,
    /\bpub fn selfhost_name_scope_add_result_into_scope <\(SelfhostNameScopeAddResult\)\*>SelfhostNameScope>/,
    "name scope add-result scope accessor must consume the wrapper",
);
assert.doesNotMatch(
    source,
    /\bfield::get\s+scope_add\s+"(?:scope|def_id)"/,
    "stage0 smoke path must use public add-result accessors, not owner-backed fields",
);

function expectedStdout(count) {
    const statuses = Array.from({ length: count }, () => "ok").join(",");
    const rows = Array.from({ length: count }, (_value, index) => `[${index}] ok`);
    return [`Checked [${statuses}]`, ...rows, ""].join("\n");
}

parsed.doctests.forEach((doctest, index) => {
    const name = `selfhost name resolver doctest#${index + 1}`;

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
        `${name} must use public name-scope add-result accessors, not owner-backed fields`,
    );
    assert.match(
        doctest.code,
        /checks_print_report[\s\S]*checks_exit_code/,
        `${name} must print the report before returning its exit code`,
    );
});

console.log("selfhost name resolver report contract passed");

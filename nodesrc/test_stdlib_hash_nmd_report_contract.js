#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "stdlib", "tests", "hash.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 1, "stdlib/tests/hash.n.md doctest count changed");

const doctest = parsed.doctests[0];
assert.equal(doctest.ret, null, "hash_main must not use ret-only success reporting");
assert.equal(doctest.exit_code, 0, "hash_main must pin exit_code: 0");
assert.deepEqual(doctest.tags, ["stdio", "normalize_newlines"], "hash_main must be a stdout-normalized stdio fixture");
assert.match(
    doctest.stdout,
    /^test_report name="hash_main" count=9 failed=0\n/,
    "hash_main must pin canonical stdout report and assertion count",
);
assert.match(doctest.stdout, /label="fnv1a32 finalize"/, "FNV assertion label must be pinned");
assert.match(doctest.stdout, /label="hash32 trait stable"/, "Hash trait stable label must be pinned");
assert.match(doctest.stdout, /label="sha256 empty bytes"/, "SHA-256 empty digest label must be pinned");
assert.match(doctest.stdout, /label="sha256 abc bytes"/, "SHA-256 abc digest label must be pinned");
assert.match(doctest.stdout, /label="sha256 multi bytes"/, "SHA-256 multi digest label must be pinned");
assert.match(doctest.code, /test_report_new "hash_main"/, "hash_main must construct a named TestReport");
assert.match(doctest.code, /test_report_print_stdout\b/, "hash_main must print the report");
assert.match(doctest.code, /test_report_exit_code\b/, "hash_main must derive exit code from the shown report");
assert.match(doctest.code, /\bsha256_digest_matches_loop\b/, "hash_main must keep byte-level SHA-256 proof in source");
assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, "hash_main must not hide report details behind checks_exit_code");
assert.doesNotMatch(doctest.code, /\bchecks_print_report\b/, "hash_main must not use legacy Checks report output");
assert.doesNotMatch(doctest.code, /\bchecks_new\b/, "hash_main must not use legacy Checks construction");

console.log("stdlib hash.n.md report contract passed");

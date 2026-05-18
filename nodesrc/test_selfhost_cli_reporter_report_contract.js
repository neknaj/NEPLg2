#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { parseFile } = require("./parser");

const repoRoot = path.resolve(__dirname, "..");
const file = path.join(repoRoot, "tests", "stdlib", "selfhost_cli_reporter.n.md");
const parsed = parseFile(file);

assert.equal(parsed.doctests.length, 3, "selfhost_cli_reporter doctest count changed");

const expectedReport = ["Checked [ok,ok]", "[0] ok", "[1] ok", ""].join("\n");

for (const index of [0, 2]) {
    const doctest = parsed.doctests[index];
    const name = `selfhost_cli_reporter doctest#${index + 1}`;

    assert.equal(doctest.ret, null, `${name} must not use ret: as an exit-code substitute`);
    assert.equal(doctest.exit_code, 0, `${name} must fix exit_code`);
    assert.deepEqual(
        doctest.tags,
        ["stdio", "normalize_newlines"],
        `${name} must be a stdout-normalized stdio fixture`,
    );
    assert.equal(doctest.stdout, expectedReport, `${name} must fix the std/test report stdout`);
    assert.match(
        doctest.code,
        /checks_print_report\s+checks[\s\S]*checks_exit_code\s+shown/,
        `${name} must print the report before returning its exit code`,
    );
}

assert.match(
    parsed.doctests[0].code,
    /#import\s+"neplg2\/cli\/reporter\/render\/single"\s+as\s+\*/,
    "single diagnostic render doctest must avoid importing the full reporter facade",
);
assert.match(
    parsed.doctests[2].code,
    /#import\s+"neplg2\/cli\/reporter\/render\/collection"\s+as\s+\*/,
    "collection render doctest must avoid importing the full reporter facade",
);

const writer = parsed.doctests[1];
assert.equal(writer.ret, null, "selfhost_cli_reporter doctest#2 must not use ret: as an exit-code substitute");
assert.equal(writer.exit_code, 0, "selfhost_cli_reporter doctest#2 must fix exit_code");
assert.deepEqual(
    writer.tags,
    ["stdio", "normalize_newlines"],
    "selfhost_cli_reporter doctest#2 must be a stdout-normalized stdio fixture",
);
assert.equal(
    writer.stdout,
    "{\"severity\":\"error\",\"code\":\"parser.token.index_unavailable\",\"message\":\"bad input\",\"primary_label\":null,\"note\":null}",
    "selfhost_cli_reporter doctest#2 must keep JSON stdout as the observable output",
);
assert.equal(
    writer.stderr,
    "error[parser.token.index_unavailable]: bad input\n",
    "selfhost_cli_reporter doctest#2 must keep human stderr as the observable output",
);
assert.match(
    writer.code,
    /#import\s+"neplg2\/cli\/reporter\/write"\s+as\s+\*/,
    "writer doctest must import the stdio write boundary without the full reporter facade",
);

console.log("selfhost CLI reporter report contract passed");

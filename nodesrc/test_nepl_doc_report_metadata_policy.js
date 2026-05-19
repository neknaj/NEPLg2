#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    assertReportMetadataPolicySelfTest,
    scanReportMetadataPolicy,
} = require("./report_metadata_policy");

const repoRoot = path.resolve(__dirname, "..");
const roots = ["tests", "tutorials", "stdlib", "examples"];

assertReportMetadataPolicySelfTest();

const { violations, reportDoctestCount } = scanReportMetadataPolicy({
    repoRoot,
    roots,
    extension: ".nepl",
});

assert(reportDoctestCount > 0, ".nepl report metadata policy must scan active report doctests");
assert.deepEqual(
    violations,
    [],
    `.nepl report doctests must pin stdout, exit_code, and stdio metadata:\n${violations.join("\n")}`,
);

console.log(".nepl report metadata policy passed");

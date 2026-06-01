#!/usr/bin/env node
"use strict";

const assert = require("assert");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const repoRoot = path.resolve(__dirname, "..");
const testsJsPath = path.join(repoRoot, "nodesrc", "tests.js");
const testsJs = fs.readFileSync(testsJsPath, "utf8").replace(/\r\n/g, "\n");

function hashShardKey(s) {
    return crypto.createHash("sha256").update(String(s), "utf8").digest().readUInt32BE(0);
}

assert.match(testsJs, /\[--shard INDEX\/TOTAL\]/, "tests.js help must document shard syntax");
assert.match(testsJs, /allCases\s*=\s*sortCasesForSharding\(allCases\);[\s\S]*allCases\s*=\s*applyCaseShard\(allCases,\s*shard\);[\s\S]*const wasmCases = allCases\.filter/, "tests.js must shard the stable original doctest set before deriving wasm and LLVM cases");
assert.match(testsJs, /shard:\s*shardSummary\(shard,\s*allCasesBeforeShard,\s*allCasesAfterShard\)/, "tests.js JSON scan metadata must record shard bounds");

const tmpDir = path.join(repoRoot, "tmp", `tests-js-shard-policy-${process.pid}`);
fs.mkdirSync(tmpDir, { recursive: true });
const fixturePath = path.join(tmpDir, "one_case.n.md");
fs.writeFileSync(fixturePath, [
    "# one case",
    "",
    "neplg2:test",
    "```neplg2",
    "#entry main",
    "#indent 4",
    "fn main %fn unit i32 \\unit:",
    "    0",
    "```",
    "",
].join("\n"));

const relFixture = path.relative(repoRoot, fixturePath);
const fixtureId = `${relFixture}::doctest#1`;
const remainder = hashShardKey(`${relFixture}::1::${fixtureId}`) % 2;
const emptyShardIndex = remainder === 0 ? 2 : 1;
const outPath = path.join(tmpDir, "empty-shard.json");
const emptyShard = spawnSync(process.execPath, [
    testsJsPath,
    "-i",
    fixturePath,
    "--no-tree",
    "-o",
    outPath,
    "--shard",
    `${emptyShardIndex}/2`,
    "-j",
    "1",
], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.equal(emptyShard.status, 0, `empty shard must be a successful zero-result report\nstdout:\n${emptyShard.stdout}\nstderr:\n${emptyShard.stderr}`);
const emptyJson = JSON.parse(fs.readFileSync(outPath, "utf8"));
assert.deepEqual(emptyJson.summary, { total: 0, passed: 0, failed: 0, errored: 0 });
assert.equal(emptyJson.scan.shard.cases_before, 1);
assert.equal(emptyJson.scan.shard.cases_after, 0);

const invalidShard = spawnSync(process.execPath, [
    testsJsPath,
    "-i",
    fixturePath,
    "--no-tree",
    "-o",
    path.join(tmpDir, "invalid-shard.json"),
    "--shard",
    "0/2",
    "-j",
    "1",
], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.notEqual(invalidShard.status, 0, "invalid shard index must be rejected");
assert.match(`${invalidShard.stdout}\n${invalidShard.stderr}`, /--shard index must be in 1\.\.TOTAL/);

console.log("nodesrc tests.js shard policy regression passed");

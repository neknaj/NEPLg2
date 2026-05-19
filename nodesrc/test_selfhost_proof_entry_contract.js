#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

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

const proofFacade = read("stdlib/neplg2/core/proof.nepl");
const proofFact = read("stdlib/neplg2/core/proof/fact.nepl");
const proofObligation = read("stdlib/neplg2/core/proof/obligation.nepl");
const proofQuery = read("stdlib/neplg2/core/proof/query.nepl");
const proofSolver = read("stdlib/neplg2/core/proof/solver.nepl");
const moduleChecker = read("stdlib/neplg2/core/check/module.nepl");
const checker = read("stdlib/neplg2/core/check/checker.nepl");

assert.match(proofFacade, /pub #import "\.\/proof\/fact" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/obligation" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/query" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/solver" as \*/);

assert.match(proofFact, /pub enum SelfhostProofDomain:/, "proof domain must be a typed enum");
assert.match(proofFact, /pub enum SelfhostProofFact:/, "proof facts must be typed enum payloads");
assert.match(proofObligation, /pub enum SelfhostProofObligation:/, "proof obligations must be typed enum payloads");
assert.match(proofQuery, /pub enum SelfhostProofResultKind:/, "proof results must use enum kind");
assert.match(proofQuery, /fact <SelfhostProofFact>/, "proof query must carry a typed fact");
assert.match(proofQuery, /obligation <SelfhostProofObligation>/, "proof query must carry a typed obligation");

const solverBlock = functionBlock(proofSolver, "selfhost_proof_fact_supports_obligation");
assert.match(solverBlock, /\bmatch\s+obligation:/, "solver must match on obligation enum");
assert.match(solverBlock, /\bmatch\s+fact:/, "solver must match on fact enum");
assert.doesNotMatch(solverBlock, /^\s*_:/m, "solver must not hide new fact/obligation variants behind wildcard arms");
assert.doesNotMatch(solverBlock, /"[A-Za-z0-9_.:-]+"/, "proof solver must not depend on string codes or module names");

assert.match(moduleChecker, /#import "neplg2\/core\/proof" as \*/, "module checker must depend on the generic proof facade");
assert.match(
    moduleChecker,
    /selfhost_proof_source_span_valid\s+item\.span/,
    "module item span validation must go through the proof solver",
);
assert.doesNotMatch(
    moduleChecker,
    /source_span_is_valid\s+item\.span/,
    "module checker must not bypass proof for module item span validation",
);
assert.doesNotMatch(
    checker,
    /#import "neplg2\/core\/proof"/,
    "checker facade should stay orchestration-only and avoid direct proof implementation coupling",
);

console.log("selfhost proof entry contract passed");

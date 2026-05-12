#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const CORE_SRC = path.join(ROOT, "nepl-core", "src");
const PLAN = path.join(ROOT, "doc", "neplg2", "parser_backend_responsibility_split_plan.md");
const RUNNER = path.join(ROOT, "nodesrc", "run_source_policy_regressions.js");

function read(filePath) {
    return fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function lineCount(text) {
    return text.split("\n").length;
}

function assertContains(text, needle, label) {
    assert(text.includes(needle), `${label} must contain ${needle}`);
}

function assertLineLimit(relativePath, limit) {
    const filePath = path.join(ROOT, relativePath);
    const lines = lineCount(read(filePath));
    assert(lines <= limit, `${relativePath} has ${lines} lines; responsibility freeze limit is ${limit}`);
}

const plan = read(PLAN);
for (const marker of [
    "ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587",
    "## Parser split stages",
    "## Backend split stages",
    "## Monomorphize split stages",
    "P1: token stream / recovery",
    "B2: WASM backend",
    "B3: LLVM backend",
    "M1: trait impl index",
    "Source policy",
]) {
    assertContains(plan, marker, "parser/backend responsibility split plan");
}

const parser = read(path.join(CORE_SRC, "parser.rs"));
const parserDocLine = "//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).";
assert(
    parser.indexOf(`${parserDocLine}\n${parserDocLine}`) === -1,
    "parser.rs must not repeat the module-level documentation line",
);

assertLineLimit("nepl-core/src/parser.rs", 4234);
assertLineLimit("nepl-core/src/codegen_wasm.rs", 2574);
assertLineLimit("nepl-core/src/codegen_llvm.rs", 4189);
assertLineLimit("nepl-core/src/monomorphize.rs", 1425);
assertLineLimit("nepl-core/src/monomorphize/trait_identity.rs", 45);
assertLineLimit("nepl-core/src/monomorphize/trait_lookup.rs", 90);

const monomorphizeRoot = read(path.join(CORE_SRC, "monomorphize.rs"));
assertContains(monomorphizeRoot, "mod trait_identity;", "monomorphize root");
assertContains(monomorphizeRoot, "mod trait_lookup;", "monomorphize root");

const runner = read(RUNNER);
assertContains(
    runner,
    "nodesrc/test_parser_backend_responsibility_policy.js",
    "source policy runner",
);

console.log("parser/backend responsibility policy ok");

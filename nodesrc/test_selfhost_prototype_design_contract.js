#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function commentBlockBeforeDeclaration(source, declarationName) {
    const lines = source.split("\n");
    const declarationPattern = new RegExp(`\\b(?:pub\\s+)?fn\\s+${declarationName}\\b`);
    const declarationIndex = lines.findIndex((line) => declarationPattern.test(line));
    assert.notEqual(declarationIndex, -1, `${declarationName} declaration must exist`);
    const block = [];
    for (let index = declarationIndex - 1; index >= 0; index -= 1) {
        const line = lines[index];
        if (!line.startsWith("//:")) {
            break;
        }
        block.push(line);
    }
    return block.reverse().join("\n");
}

const checklist = read("doc/neplg2/self_host_zenn_review_checklist.md");
const prompt = read("doc/neplg2/self_host_zenn_review_prompt.md");
const executionPlan = read("doc/neplg2/self_host_execution_plan.md");
const design = read("doc/neplg2/self_host_neplg21_compiler_design.md");
const docIssue = read("issues/items/ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41.md");
const moduleSolver = read("stdlib/neplg2/core/proof/solver/module.nepl");

for (const [label, source] of [
    ["checklist", checklist],
    ["prompt", prompt],
    ["execution plan", executionPlan],
    ["design", design],
    ["documentation issue", docIssue],
]) {
    assert.ok(
        source.includes("https://zenn.dev/bem130/articles/1b352797de94e7"),
        `${label} must keep the Zenn policy URL as the selfhost prototype authority`,
    );
}

for (const needle of [
    "暫定実装",
    "妥協内容",
    "fail-closed",
    "解除条件",
    "対応 issue",
]) {
    assert.ok(checklist.includes(needle), `checklist must require prototype workaround tracking: ${needle}`);
}
assert.ok(
    prompt.includes("Blocker を「後で見る」とだけ書いて merge してはならない"),
    "review prompt must reject deferring blockers without a tracked root-cause issue",
);
assert.ok(
    executionPlan.includes("self-host 限定 workaround は作らない"),
    "execution plan must reject selfhost-only workarounds for Rust/compiler design defects",
);

for (const declarationName of [
    "selfhost_proof_solve_raw_backend_transition",
    "selfhost_proof_solve_module_directive_transition",
    "selfhost_proof_solve_module_declaration_header",
]) {
    const doc = commentBlockBeforeDeclaration(moduleSolver, declarationName);
    assert.ok(doc.includes("[契約/けいやく]"), `${declarationName} must separate stable contract from implementation status`);
    assert.ok(doc.includes("[現状/げんじょう]"), `${declarationName} must record current limitations separately from contract`);
    assert.ok(doc.includes("[使用例/しようれい]"), `${declarationName} must include a typical example section`);
    assert.ok(doc.includes("neplg2:test"), `${declarationName} must include a runnable doctest for the fixed public API`);
}

assert.doesNotMatch(
    moduleSolver,
    /暫定設計|workaround|TODO|後で見る/,
    "accepted selfhost module solver slice must not contain untracked provisional design markers",
);

console.log("selfhost prototype design contract passed");

#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readHirSource } = require("./selfhost_hir_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const hir = legacyTypeSyntaxView(readHirSource(repoRoot));

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const decl = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}`);
    const start = lines.findIndex((line) => decl.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
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

assert.match(hir, /#import "core\/option" as \*/, "HIR expression ID absence must use Option");
assert.doesNotMatch(hir, /\bfn\s+selfhost_hir_expr_id_invalid\b/, "HIR expression IDs must not expose an invalid constructor");
assert.doesNotMatch(hir, /\bselfhost_hir_expr_id_new\s+-1\b/, "HIR expression IDs must not construct -1 sentinels");

const pending = topLevelBlock(hir, "fn", "selfhost_hir_expr_id_pending");
assert.match(pending, /Option<SelfhostHirExprId>/, "pending state must return Option<SelfhostHirExprId>");
assert.match(pending, /\bnone<SelfhostHirExprId>/, "pending state must be Option::None");
assert.doesNotMatch(pending, /\bSelfhostHirExprId\b\s+-1\b|\bselfhost_hir_expr_id_new\s+-1\b/, "pending state must not use an invalid ID payload");

const assigned = topLevelBlock(hir, "fn", "selfhost_hir_expr_id_assigned");
assert.match(assigned, /Option<SelfhostHirExprId>/, "assigned state must return Option<SelfhostHirExprId>");
assert.match(assigned, /\bsome<SelfhostHirExprId>\s+expr_id\b/, "assigned state must wrap the stable ID in Some");

const stage0 = topLevelBlock(hir, "fn", "selfhost_hir_stage0");
assert.match(stage0, /\blet\s+pending\s+<Option<SelfhostHirExprId>>\s+selfhost_hir_expr_id_pending\b/, "stage0 must exercise pending typed absence");
assert.match(stage0, /\bmatch\s+assigned:/, "stage0 must inspect assigned state through Option matching");
assert.match(stage0, /\bOption::Some\s+assigned_id:/, "stage0 must handle assigned Some payload");
assert.match(stage0, /\bOption::None:/, "stage0 must handle assigned None payload");
assert.match(stage0, /\bis_none<SelfhostHirExprId>\s+pending\b/, "stage0 must verify pending is None");

console.log("selfhost HIR expression ID absence regression passed");

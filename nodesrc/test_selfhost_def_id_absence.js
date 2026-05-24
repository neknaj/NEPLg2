#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readNameResolverSource } = require("./selfhost_name_resolver_sources");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const resolver = legacyTypeSyntaxView(readNameResolverSource(repoRoot));

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

assert.match(resolver, /#import "core\/option" as \*/, "resolver DefId absence must use Option");
assert.doesNotMatch(resolver, /\bfn\s+selfhost_def_id_invalid\b/, "DefId must not expose an invalid constructor");
assert.doesNotMatch(resolver, /\bselfhost_def_id_new\s+-1\b/, "DefId must not construct -1 sentinels");
assert.match(resolver, /(?:pub\s+)?struct SelfhostNameBinding:[\s\S]*?\bdef_id\s+<Option<SelfhostDefId>>/, "binding def_id must be optional until scope insertion assigns it");

const pending = topLevelBlock(resolver, "fn", "selfhost_def_id_pending");
assert.match(pending, /Option<SelfhostDefId>/, "pending state must return Option<SelfhostDefId>");
assert.match(pending, /\bnone<SelfhostDefId>/, "pending state must be Option::None");
assert.doesNotMatch(pending, /\bSelfhostDefId\b\s+-1\b|\bselfhost_def_id_new\s+-1\b/, "pending state must not use an invalid ID payload");

const assigned = topLevelBlock(resolver, "fn", "selfhost_def_id_assigned");
assert.match(assigned, /Option<SelfhostDefId>/, "assigned state must return Option<SelfhostDefId>");
assert.match(assigned, /\bsome<SelfhostDefId>\s+def_id\b/, "assigned state must wrap the stable ID in Some");

const bindingPending = topLevelBlock(resolver, "fn", "selfhost_name_binding_pending");
assert.match(bindingPending, /\bselfhost_def_id_pending\b/, "pending binding constructor must use typed DefId absence");

const addBinding = topLevelBlock(resolver, "fn", "selfhost_name_scope_add_binding");
assert.match(addBinding, /\blet\s+def_id\s+<SelfhostDefId>\s+selfhost_def_id_new\s+selfhost_name_scope_len\s+&scope\b/, "scope insertion must allocate the stable DefId");
assert.match(addBinding, /\bselfhost_name_binding_new\s+binding\.name\s+selfhost_def_id_assigned\s+def_id\s+binding\.kind\s+binding\.span\b/, "scope insertion must store assigned DefId as Some");
assert.doesNotMatch(addBinding, /\bbinding\.def_id\b/, "scope insertion must not trust the pre-insertion binding DefId");

const bindingEq = topLevelBlock(resolver, "fn", "selfhost_name_binding_def_id_eq");
assert.match(bindingEq, /\bmatch\s+binding\.def_id:/, "binding DefId checks must match Option payloads");
assert.match(bindingEq, /\bOption::Some\s+actual:/, "binding DefId checks must handle assigned IDs");
assert.match(bindingEq, /\bOption::None:/, "binding DefId checks must handle pending IDs");

const stage0 = topLevelBlock(resolver, "fn", "selfhost_name_resolver_stage0");
assert.match(stage0, /\bselfhost_name_binding_pending\s+"main"\s+SelfhostDefKind::Function\b/, "stage0 must create pending binding without invalid DefId");

console.log("selfhost DefId absence regression passed");

#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const monoPath = "stdlib/neplg2/core/mono/mono.nepl";
const mono = fs.readFileSync(path.join(repoRoot, monoPath), "utf8").replace(/\r\n/g, "\n");

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const prefix = kind === "fn" ? `fn ${name} ` : `${kind} ${name}`;
    const start = lines.findIndex((line) => line.startsWith(prefix));
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

assert.match(mono, /#import "core\/option" as \*/, "mono instance absence must use Option");
assert.doesNotMatch(mono, /\bfn\s+selfhost_mono_instance_id_invalid\b/, "mono instance IDs must not expose an invalid constructor");
assert.doesNotMatch(mono, /\bfn\s+selfhost_mono_instance_id_is_valid\b/, "mono instance IDs must not rely on validity checks for absence");
assert.doesNotMatch(mono, /\bselfhost_mono_instance_id_new\s+-1\b/, "mono instance IDs must not construct -1 sentinels");

const pending = topLevelBlock(mono, "fn", "selfhost_mono_instance_id_pending");
assert.match(pending, /Option<SelfhostMonoInstanceId>/, "pending state must return Option<SelfhostMonoInstanceId>");
assert.match(pending, /\bnone<SelfhostMonoInstanceId>/, "pending state must be Option::None");
assert.doesNotMatch(pending, /\bSelfhostMonoInstanceId\b\s+-1\b|\bselfhost_mono_instance_id_new\s+-1\b/, "pending state must not use an invalid ID payload");

const assigned = topLevelBlock(mono, "fn", "selfhost_mono_instance_id_assigned");
assert.match(assigned, /Option<SelfhostMonoInstanceId>/, "assigned state must return Option<SelfhostMonoInstanceId>");
assert.match(assigned, /\bsome<SelfhostMonoInstanceId>\s+instance_id\b/, "assigned state must wrap the stable ID in Some");

const stage0 = topLevelBlock(mono, "fn", "selfhost_mono_stage0");
assert.match(stage0, /\blet\s+pending\s+<Option<SelfhostMonoInstanceId>>\s+selfhost_mono_instance_id_pending\b/, "stage0 must exercise pending typed absence");
assert.match(stage0, /\bmatch\s+assigned:/, "stage0 must inspect assigned state through Option matching");
assert.match(stage0, /\bOption::Some\s+assigned_id:/, "stage0 must handle assigned Some payload");
assert.match(stage0, /\bOption::None:/, "stage0 must handle assigned None payload");
assert.match(stage0, /\bis_none<SelfhostMonoInstanceId>\s+pending\b/, "stage0 must verify pending is None");

console.log("selfhost mono instance absence regression passed");

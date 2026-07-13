#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/neplg2/core/check/expr/enum_member.nepl"), "utf8").replace(/\r\n/g, "\n");
const arena = fs.readFileSync(path.join(root, "stdlib/neplg2/core/ty/ty/arena.nepl"), "utf8").replace(/\r\n/g, "\n");
const facade = fs.readFileSync(path.join(root, "stdlib/neplg2/core/check/expr.nepl"), "utf8").replace(/\r\n/g, "\n");

assert.match(facade, /pub #import "\.\/expr\/enum_member" as \*/, "check facade must export enum member connector");
assert.match(arena, /pub fn selfhost_type_arena_nominal_id[\s\S]*SelfhostTypeRecord::Named[\s\S]*SelfhostTypeRecord::Applied/, "TypeArena must expose a shared Named/Applied nominal projection");
assert.match(source, /pub enum SelfhostCheckedEnumMemberErrorKind:[\s\S]*ScrutineeTypeMissing[\s\S]*ScrutineeNotNominal[\s\S]*OwnerMismatch[\s\S]*Lookup/, "connector must preserve typed failure domains");
assert.match(source, /selfhost_type_arena_get_record arena scrutinee_type[\s\S]*owner_matches[\s\S]*OwnerMismatch[\s\S]*selfhost_resolved_enum_session_member_lookup_result/, "owner mismatch must be rejected before member lookup");
assert.doesNotMatch(source, /split_member_tail|split_qualified|string_slice/, "connector must not reinterpret source qualifier spelling");
console.log("selfhost enum member checker contract passed");

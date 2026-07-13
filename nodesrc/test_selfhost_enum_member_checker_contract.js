#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/neplg2/core/check/expr/enum_member.nepl"), "utf8").replace(/\r\n/g, "\n");
const arena = fs.readFileSync(path.join(root, "stdlib/neplg2/core/ty/ty/arena.nepl"), "utf8").replace(/\r\n/g, "\n");
const facade = fs.readFileSync(path.join(root, "stdlib/neplg2/core/check/expr.nepl"), "utf8").replace(/\r\n/g, "\n");

function topLevelFunction(name) {
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const match = source.match(new RegExp(`^pub fn ${escaped}\\b[\\s\\S]*?(?=\\n(?:pub )?(?:struct|enum|fn|impl)\\s+|\\n$)`, "m"));
    assert.ok(match, `missing public function ${name}`);
    return match[0];
}

assert.match(facade, /pub #import "\.\/expr\/enum_member" as \*/, "check facade must export enum member connector");
assert.match(source, /pub enum SelfhostCheckedEnumMemberErrorKind:[\s\S]*ScrutineeTypeMissing[\s\S]*ScrutineeNotNominal[\s\S]*OwnerMismatch[\s\S]*Lookup/, "connector must preserve typed failure domains");
const connector = topLevelFunction("selfhost_check_enum_member_resolve_result");
assert.match(connector, /selfhost_type_arena_get_record arena scrutinee_type[\s\S]*owner_matches[\s\S]*OwnerMismatch[\s\S]*selfhost_resolved_enum_session_member_lookup_result/, "owner mismatch must be rejected before member lookup");
assert.equal((connector.match(/selfhost_resolved_enum_session_member_lookup_result/g) || []).length, 1, "connector must invoke member lookup exactly once");
assert.match(source, /pub struct SelfhostCheckedEnumMember:\n    scrutinee_type %SelfhostTypeId\n    member_id %SelfhostResolvedEnumMemberId\n    span %SelfhostSourceSpan/, "checked evidence must contain only typed identity and diagnostic span");
assert.doesNotMatch(source, /split_member_tail|split_qualified|string_slice/, "connector must not reinterpret source qualifier spelling");
assert.doesNotMatch(source, /#import .*\/(?:hir|resource|codegen)|ResourceIrEnumerated/, "checker connector must not depend on lowering/backend or issue production origin");
console.log("selfhost enum member checker contract passed");

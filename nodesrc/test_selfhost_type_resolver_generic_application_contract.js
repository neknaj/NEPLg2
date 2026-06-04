#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readTypeResolverSource } = require("./selfhost_type_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const source = readTypeResolverSource(repoRoot);

function topLevelBlock(src, kind, name) {
    const pattern = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|fn|impl)\\s+|\\n#|\\n//: neplg2:test|\\n$)`, "m");
    const match = src.match(pattern);
    assert.ok(match, `missing top-level ${kind} ${name}`);
    return match[0];
}

assert.match(
    source,
    /pub enum SelfhostResolvedTypeNode:[\s\S]*Primitive[\s\S]*Named[\s\S]*Applied[\s\S]*Function/,
    "resolved type tree must represent generic type applications as Applied nodes",
);
assert.match(
    source,
    /pub struct SelfhostResolvedTypeTree:[\s\S]*nodes %Vec SelfhostResolvedTypeNode[\s\S]*function_args %Vec SelfhostResolvedTypeNodeId[\s\S]*type_args %Vec SelfhostResolvedTypeNodeId/,
    "resolved type tree must store applied type arguments in a separate node-id table",
);
assert.match(
    source,
    /pub enum SelfhostTypeReduceErrorKind:[\s\S]*GenericTypeArgumentMissing[\s\S]*TrailingItems/,
    "constructor-aware reduction must distinguish missing generic arguments from trailing items",
);
assert.doesNotMatch(
    source,
    /generic application は後続 slice|generic application is a later slice/,
    "type resolver comments must not describe generic application as a future slice",
);

const reduceWithConstructors = topLevelBlock(source, "fn", "selfhost_type_prefix_list_reduce_with_constructors");
assert.match(
    reduceWithConstructors,
    /selfhost_type_prefix_list_validate_at_with_constructors/,
    "constructor-aware reducer must validate with constructor arity before building",
);
assert.match(
    reduceWithConstructors,
    /selfhost_type_prefix_list_build_at_with_constructors/,
    "constructor-aware reducer must build with constructor arity",
);

const validateNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_validate_named_with_constructors");
assert.match(
    validateNamed,
    /constructor\.arity[\s\S]*selfhost_type_prefix_list_validate_generic_args_loop/,
    "named type validation must consume arity-driven generic arguments",
);

const buildNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_build_named_with_constructors");
assert.match(
    buildNamed,
    /constructor\.arity[\s\S]*selfhost_type_prefix_list_build_generic_args_loop[\s\S]*selfhost_type_reduce_add_applied_named_from_params/,
    "named type build must lower arity-driven generic arguments into Applied nodes",
);

const addApplied = topLevelBlock(source, "fn", "selfhost_type_reduce_add_applied_named_from_params");
assert.match(
    addApplied,
    /selfhost_resolved_applied_type_arg_range_new_unchecked[\s\S]*SelfhostResolvedTypeNode::Applied/,
    "constructor-aware reduction must create an applied node with a typed argument range",
);

const projectPlain = topLevelBlock(source, "fn", "selfhost_type_project_node");
assert.match(
    projectPlain,
    /SelfhostResolvedTypeNode::Applied\s+applied:[\s\S]*UnsupportedNamedType/,
    "plain projection must fail closed for Applied nodes without a constructor table boundary",
);

const projectApplied = topLevelBlock(source, "fn", "selfhost_type_project_applied_with_constructors");
assert.match(
    projectApplied,
    /selfhost_type_project_applied_args_with_constructors[\s\S]*selfhost_type_arena_add_applied_named/,
    "constructor-aware projection must project applied arguments and store an applied arena record",
);

const projectWithConstructors = topLevelBlock(source, "fn", "selfhost_type_project_node_with_constructors");
assert.match(
    projectWithConstructors,
    /SelfhostResolvedTypeNode::Applied\s+applied:[\s\S]*selfhost_type_project_applied_with_constructors/,
    "constructor-aware projection must dispatch Applied nodes to the applied projection path",
);

console.log("selfhost type resolver generic application contract passed");

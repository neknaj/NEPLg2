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
assert.match(
    source,
    /pub enum SelfhostTypeConstructorKind:[\s\S]*Type[\s\S]*TypeConstructor %i32/,
    "constructor table must store checked constructor kind instead of passing raw arity through reducers",
);
assert.match(
    source,
    /pub enum SelfhostTypeConstructorTableErrorKind:[\s\S]*NegativeConstructorArity[\s\S]*DuplicateTypeConstructor[\s\S]*ReservedTypeConstructorName[\s\S]*OutOfMemory/,
    "constructor header validation must report typed table errors before entries reach the reducer",
);
const addChecked = topLevelBlock(source, "fn", "selfhost_type_constructor_table_add_checked");
assert.match(
    addChecked,
    /selfhost_type_constructor_kind_from_arity_checked/,
    "constructor table insertion must normalize arity through the checked kind helper",
);
for (const kind of ["DuplicateTypeConstructor", "ReservedTypeConstructorName"]) {
    assert.match(addChecked, new RegExp(kind), `constructor table insertion must check ${kind} at the table boundary`);
}
const kindFromArity = topLevelBlock(source, "fn", "selfhost_type_constructor_kind_from_arity_checked");
assert.match(
    kindFromArity,
    /NegativeConstructorArity[\s\S]*SelfhostTypeConstructorKind::Type[\s\S]*SelfhostTypeConstructorKind::TypeConstructor/,
    "constructor kind normalization must reject negative arity and encode zero/nonzero arity as checked kind",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_type_constructor_table_add\b/,
    "unchecked constructor table insertion must not remain as the public path",
);
for (const legacyReducerApi of [
    "selfhost_type_prefix_list_validate_at_with_constructors",
    "selfhost_type_prefix_list_build_at_with_constructors",
    "selfhost_type_prefix_list_validate_at_with_constructors_and_type_parameters",
    "selfhost_type_prefix_list_build_at_with_constructors_and_type_parameters",
]) {
    assert.doesNotMatch(
        source,
        new RegExp(`${legacyReducerApi}\\b`),
        `${legacyReducerApi} must not remain because constructor lookup must pass through SelfhostTypeBoundPlan`,
    );
}
assert.doesNotMatch(
    source,
    /constructor\.arity/,
    "reducers and projection must use checked constructor kind accessors instead of raw arity fields",
);
assert.match(
    source,
    /pub enum SelfhostTypeBoundName:[\s\S]*Constructor %SelfhostTypeConstructor[\s\S]*TypeParameter %SelfhostTypeParameter[\s\S]*Conflict[\s\S]*Unresolved/,
    "constructor and type-parameter lookup must be represented as a bound plan item before validation/build",
);
assert.doesNotMatch(
    source,
    /generic application は後続 slice|generic application is a later slice/,
    "type resolver comments must not describe generic application as a future slice",
);

const reduceWithConstructors = topLevelBlock(source, "fn", "selfhost_type_prefix_list_reduce_with_constructors");
assert.match(
    reduceWithConstructors,
    /selfhost_type_bound_plan_from_reduce_plan_with_constructors[\s\S]*selfhost_type_prefix_list_validate_at_bound/,
    "constructor-aware reducer must bind constructor lookup once before validation",
);
assert.match(
    reduceWithConstructors,
    /selfhost_type_prefix_list_build_at_bound/,
    "constructor-aware reducer must build from the same bound plan used by validation",
);

const validateNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_validate_bound_named");
assert.match(
    validateNamed,
    /selfhost_type_constructor_kind_arg_count[\s\S]*selfhost_type_prefix_list_validate_generic_args_loop_bound/,
    "named type validation must consume kind-driven generic arguments from a bound constructor",
);

const buildNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_build_bound_named");
assert.match(
    buildNamed,
    /selfhost_type_constructor_kind_arg_count[\s\S]*selfhost_type_prefix_list_build_generic_args_loop_bound[\s\S]*selfhost_type_reduce_add_applied_named_from_params/,
    "named type build must lower kind-driven generic arguments into Applied nodes",
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
    source,
    /pub enum SelfhostTypeProjectErrorKind:[\s\S]*GenericConstructorArgumentArityMismatch/,
    "projection must distinguish malformed applied constructor arity from generic constructor bare use",
);
assert.match(
    projectApplied,
    /selfhost_type_constructor_table_get constructors applied\.nominal_id[\s\S]*GenericConstructorArgumentArityMismatch[\s\S]*UnknownNamedType/,
    "constructor-aware projection must re-check constructor identity and arity before writing Applied into TypeArena",
);
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

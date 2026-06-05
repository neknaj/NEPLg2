#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TYPE_RESOLVER_FACADE,
    TYPE_RESOLVER_SPLIT_FILES,
    readRepoFile,
    readTypeResolverSource,
} = require("./selfhost_type_resolver_sources");

const repoRoot = path.resolve(__dirname, "..");
const source = readTypeResolverSource(repoRoot);
const facade = readRepoFile(repoRoot, TYPE_RESOLVER_FACADE);

function escapeRegExp(text) {
    return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function topLevelBlock(src, kind, name) {
    const pattern = new RegExp(`^(?:pub\\s+)?${kind}\\s+${escapeRegExp(name)}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|fn|impl)\\s+|\\n#|\\n//: neplg2:test|\\n$)`, "m");
    const match = src.match(pattern);
    assert.ok(match, `missing top-level ${kind} ${name}`);
    return match[0];
}

for (const relPath of [
    "stdlib/neplg2/core/resolve/type_resolver/typeparam/id.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/typeparam/model.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/typeparam/env.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/typeparam.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/resolved/typeparam.nepl",
]) {
    assert.ok(
        TYPE_RESOLVER_SPLIT_FILES.includes(relPath),
        `${relPath} must be part of the type resolver source policy list`,
    );
    const importPath = relPath
        .replace(/^stdlib\/neplg2\/core\/resolve\/type_resolver\//, "./type_resolver/")
        .replace(/\.nepl$/, "");
    assert.match(
        facade,
        new RegExp(`^pub #import "${escapeRegExp(importPath)}" as \\*$`, "m"),
        `${TYPE_RESOLVER_FACADE} must re-export ${importPath}`,
    );
}

assert.match(
    source,
    /pub enum SelfhostResolvedTypeNode:[\s\S]*Named[\s\S]*Parameter[\s\S]*Applied/,
    "resolved type tree must keep generic type parameters distinct from named and applied types",
);
assert.match(
    source,
    /pub enum SelfhostTypeReduceErrorKind:[\s\S]*TypeParameterConstructorNameConflict/,
    "type parameter and constructor name ambiguity must have a typed reduce error",
);
assert.match(
    source,
    /pub enum SelfhostTypeBoundName:[\s\S]*TypeParameter %SelfhostTypeParameter[\s\S]*Conflict/,
    "type parameter lookup and constructor conflicts must be fixed in the bound plan before validate/build",
);
assert.match(
    source,
    /SelfhostResolvedTypeNode::Parameter parameter:[\s\S]*selfhost_type_project_parameter arena parameter/,
    "projection must lower resolved type parameters into a dedicated TypeArena parameter record",
);
assert.doesNotMatch(
    source,
    /SelfhostTypeProjectErrorKind::UnsupportedTypeParameter/,
    "type parameter projection must no longer fail closed after the binder-indexed arena record is available",
);
assert.match(
    source,
    /selfhost_type_project_parameter_binding_for_current_binder[\s\S]*selfhost_type_parameter_binding_new_unchecked 0 selfhost_type_parameter_id_index parameter\.parameter_id/,
    "resolver-local type parameter ids must be normalized into binder-depth-zero parameter bindings at projection",
);

const boundNameWithTypeParameters = topLevelBlock(
    source,
    "fn",
    "selfhost_type_bound_name_with_constructors_and_type_parameters",
);
assert.match(
    boundNameWithTypeParameters,
    /selfhost_type_parameter_env_find_span[\s\S]*selfhost_type_constructor_table_find_span[\s\S]*SelfhostTypeBoundName::Conflict/,
    "binding must detect names that are both type parameters and constructors before validation/build",
);

const validateNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_validate_bound_named");
assert.match(
    validateNamed,
    /SelfhostTypeBoundName::TypeParameter _parameter:[\s\S]*Result::Ok add idx 1/,
    "validation must consume a bound type parameter as exactly one type expression",
);

const buildNamed = topLevelBlock(source, "fn", "selfhost_type_prefix_list_build_bound_named");
assert.match(
    buildNamed,
    /SelfhostTypeBoundName::TypeParameter parameter:[\s\S]*SelfhostResolvedTypeNode::Parameter[\s\S]*parameter\.parameter_id/,
    "build must lower a matching type parameter into a Parameter node",
);
assert.match(
    buildNamed,
    /SelfhostTypeBoundName::Conflict:[\s\S]*TypeParameterConstructorNameConflict/,
    "build must reject constructor/type-parameter name conflicts instead of silently choosing one",
);
assert.match(
    buildNamed,
    /SelfhostTypeBoundName::Unresolved:[\s\S]*SelfhostResolvedTypeNode::Named/,
    "names absent from both tables must remain unresolved named nodes for downstream diagnostics",
);

const reduceWithTypeParameters = topLevelBlock(
    source,
    "fn",
    "selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters",
);
assert.match(
    reduceWithTypeParameters,
    /selfhost_type_bound_plan_from_reduce_plan_with_constructors_and_type_parameters[\s\S]*selfhost_type_prefix_list_validate_at_bound/,
    "type-parameter-aware reducer must bind constructor and parameter lookup before validation",
);
assert.match(
    reduceWithTypeParameters,
    /selfhost_type_prefix_list_build_at_bound/,
    "type-parameter-aware reducer must build with the same bound plan used by validation",
);

console.log("selfhost type resolver type parameter contract passed");

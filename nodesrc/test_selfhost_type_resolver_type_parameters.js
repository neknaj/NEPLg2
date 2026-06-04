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
    /SelfhostTypeProjectErrorKind::UnsupportedTypeParameter/,
    "projection must fail closed until type parameters have a binder-indexed arena record",
);

const validateNamed = topLevelBlock(
    source,
    "fn",
    "selfhost_type_prefix_list_validate_named_with_constructors_and_type_parameters",
);
assert.match(
    validateNamed,
    /selfhost_type_parameter_env_find_span[\s\S]*selfhost_type_constructor_table_find_span[\s\S]*TypeParameterConstructorNameConflict/,
    "validation must reject names that are both type parameters and constructors",
);
assert.match(
    validateNamed,
    /Option::Some _parameter:[\s\S]*Option::None:[\s\S]*Result::Ok add idx 1/,
    "validation must consume a type parameter as exactly one type expression",
);

const buildNamed = topLevelBlock(
    source,
    "fn",
    "selfhost_type_prefix_list_build_named_with_constructors_and_type_parameters",
);
assert.match(
    buildNamed,
    /Option::Some parameter:[\s\S]*SelfhostResolvedTypeNode::Parameter[\s\S]*parameter\.parameter_id/,
    "build must lower a matching type parameter into a Parameter node",
);
assert.match(
    buildNamed,
    /Option::Some _constructor:[\s\S]*TypeParameterConstructorNameConflict/,
    "build must reject constructor/type-parameter name conflicts instead of silently choosing one",
);
assert.match(
    buildNamed,
    /Option::None:[\s\S]*SelfhostResolvedTypeNode::Named/,
    "names absent from both tables must remain unresolved named nodes for downstream diagnostics",
);

const reduceWithTypeParameters = topLevelBlock(
    source,
    "fn",
    "selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters",
);
assert.match(
    reduceWithTypeParameters,
    /selfhost_type_prefix_list_validate_at_with_constructors_and_type_parameters/,
    "type-parameter-aware reducer must validate with the same environment it uses to build",
);
assert.match(
    reduceWithTypeParameters,
    /selfhost_type_prefix_list_build_at_with_constructors_and_type_parameters/,
    "type-parameter-aware reducer must build with the type parameter environment",
);

console.log("selfhost type resolver type parameter contract passed");

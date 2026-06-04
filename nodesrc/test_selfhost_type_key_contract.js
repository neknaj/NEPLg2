#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readRepoFile, readTySource } = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const keyPath = "stdlib/neplg2/core/ty/ty/key.nepl";
const facadePath = "stdlib/neplg2/core/ty/ty.nepl";
const key = readRepoFile(repoRoot, keyPath);
const facade = readRepoFile(repoRoot, facadePath);
const ty = readTySource(repoRoot);

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const declaration = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${escaped}\\s+`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${escaped}`);
    const start = lines.findIndex((line) => declaration.test(line));
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

function enumVariants(src, enumName) {
    return topLevelBlock(src, "enum", `${enumName}:`)
        .split("\n")
        .slice(1)
        .map((line) => /^    ([A-Za-z][A-Za-z0-9_]*)(?:\s+%[A-Za-z][A-Za-z0-9_]*)?$/.exec(line))
        .filter(Boolean)
        .map((match) => match[1]);
}

assert.match(facade, /^pub #import "\.\/ty\/key" as \*$/m, "ty facade must re-export canonical type key API");
assert.match(ty, /SelfhostCanonicalTypeKeyArena/, "selfhost ty source aggregate must include canonical key module");

assert.deepEqual(
    enumVariants(key, "SelfhostCanonicalTypeKeyNode"),
    ["Primitive", "Named", "Applied", "Function"],
    "canonical type key node must cover primitive, named, applied, and function payloads",
);
assert.deepEqual(
    enumVariants(key, "SelfhostCanonicalTypeKeyProjectErrorKind"),
    [
        "MissingTypeRecord",
        "MissingTypeArgument",
        "MissingFunctionArgument",
        "InvalidAppliedArgumentRange",
        "InvalidFunctionArgumentRange",
        "OutOfMemory",
        "InternalInvariant",
    ],
    "projection errors must be typed and exhaustive",
);

const nodeBlock = topLevelBlock(key, "enum", "SelfhostCanonicalTypeKeyNode:");
assert.doesNotMatch(nodeBlock, /\bSelfhostTypeId\b/, "canonical key nodes must not contain arena-local SelfhostTypeId");
assert.match(
    key,
    /pub struct SelfhostCanonicalTypeKeyArena:[\s\S]*?\bnodes\s+%Vec SelfhostCanonicalTypeKeyNode[\s\S]*?\bargs\s+%Vec SelfhostCanonicalTypeKeyId/,
    "canonical key arena must own node and argument tables",
);
assert.match(
    key,
    /pub struct SelfhostCanonicalTypeKeyProjectError:[\s\S]*?\btype_id\s+%SelfhostTypeId/,
    "projection errors may carry the source arena-local TypeId as diagnostic payload",
);

const projectNode = topLevelBlock(key, "fn", "selfhost_canonical_type_key_project_node");
for (const variant of ["Primitive", "Named", "Applied", "Function"]) {
    assert.match(
        projectNode,
        new RegExp(`^\\s*SelfhostTypeRecord::${variant}\\b`, "m"),
        `canonical projection must handle ${variant} type records`,
    );
}
assert.match(projectNode, /\bselfhost_canonical_type_key_push_node\b/, "projection must push canonical key nodes through the arena helper");

const equalityBlock = topLevelBlock(key, "fn", "selfhost_canonical_type_key_equal");
assert.doesNotMatch(equalityBlock, /\bselfhost_type_id_eq\b/, "canonical key equality must not use arena-local TypeId equality");
assert.match(
    key,
    /\bselfhost_canonical_type_key_project_applied\b[\s\S]*selfhost_applied_type_arg_range_is_valid/,
    "applied projection must validate its source argument range",
);
assert.match(
    key,
    /\bselfhost_canonical_type_key_project_function\b[\s\S]*selfhost_function_type_arg_range_is_valid/,
    "function projection must validate its source argument range",
);
assert.match(
    key,
    /\bselfhost_canonical_type_key_project_type_args_loop\b[\s\S]*selfhost_type_arena_applied_arg\b[\s\S]*selfhost_canonical_type_key_project_node/,
    "applied projection must recursively project each type argument once",
);
assert.match(
    key,
    /\bselfhost_canonical_type_key_project_function_args_loop\b[\s\S]*selfhost_type_arena_function_arg\b[\s\S]*selfhost_canonical_type_key_project_node/,
    "function projection must recursively project each function argument once",
);

console.log("selfhost canonical type key contract passed");

#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const preludePath = "stdlib/neplg2/core/builtins/prelude.nepl";
const prelude = fs.readFileSync(path.join(repoRoot, preludePath), "utf8").replace(/\r\n/g, "\n");

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const start = lines.findIndex((line) => line.startsWith(`${kind} ${name}`));
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

function enumVariants(src, name) {
    const block = topLevelBlock(src, "enum", `${name}:`);
    return block
        .split("\n")
        .slice(1)
        .map((line) => /^    ([A-Za-z][A-Za-z0-9_]*)(?:\s+<[^>]+>)?$/.exec(line))
        .filter(Boolean)
        .map((match) => match[1]);
}

function assertAccessorsMatchSignature(fnName) {
    const block = topLevelBlock(prelude, "fn", fnName);
    assert.match(block, /\bmatch\s+builtin\.signature:/, `${fnName} must dispatch through SelfhostBuiltinSignature`);
    assert.doesNotMatch(block, /\bbuiltin\.arg[0-9]\b/, `${fnName} must not read fixed argument slots`);
    assert.doesNotMatch(block, /\bbuiltin\.arg_count\b/, `${fnName} must not read fixed arg_count`);
    assert.doesNotMatch(block, /\bbuiltin\.result\b/, `${fnName} must not read a fixed result slot`);
    for (const variant of ["Unary", "Binary", "Ternary"]) {
        assert.match(
            block,
            new RegExp(`^\\s*SelfhostBuiltinSignature::${variant}\\b`, "m"),
            `${fnName} must handle SelfhostBuiltinSignature::${variant}`,
        );
    }
}

const functionStruct = topLevelBlock(prelude, "struct", "SelfhostBuiltinFunction:");
assert.match(functionStruct, /\bsignature\s+<SelfhostBuiltinSignature>/, "SelfhostBuiltinFunction must store a signature payload");
assert.doesNotMatch(functionStruct, /\barg[0-9]\s+<SelfhostTypeKind>/, "SelfhostBuiltinFunction must not store fixed argument slots");
assert.doesNotMatch(functionStruct, /\barg_count\s+<i32>/, "SelfhostBuiltinFunction must not store a numeric arity slot");
assert.doesNotMatch(functionStruct, /\bresult\s+<SelfhostTypeKind>/, "SelfhostBuiltinFunction must not store a fixed result slot");

assert.deepEqual(
    enumVariants(prelude, "SelfhostBuiltinSignature"),
    ["Unary", "Binary", "Ternary"],
    "SelfhostBuiltinSignature must model each supported arity as an enum variant",
);
assert.match(prelude, /struct SelfhostBuiltinUnarySignature:[\s\S]*?\barg0\s+<SelfhostTypeKind>[\s\S]*?\bresult\s+<SelfhostTypeKind>/, "unary signature must carry exactly its argument and result");
assert.match(prelude, /struct SelfhostBuiltinBinarySignature:[\s\S]*?\barg0\s+<SelfhostTypeKind>[\s\S]*?\barg1\s+<SelfhostTypeKind>[\s\S]*?\bresult\s+<SelfhostTypeKind>/, "binary signature must carry its arguments and result");
assert.match(prelude, /struct SelfhostBuiltinTernarySignature:[\s\S]*?\barg0\s+<SelfhostTypeKind>[\s\S]*?\barg1\s+<SelfhostTypeKind>[\s\S]*?\barg2\s+<SelfhostTypeKind>[\s\S]*?\bresult\s+<SelfhostTypeKind>/, "ternary signature must carry its arguments and result");

assert.doesNotMatch(prelude, /\bSelfhostTypeKind::Error\b/, "builtin metadata must not use Error as a placeholder argument");
assert.doesNotMatch(prelude, /\barg_count\s+<i32>/, "builtin metadata must not reintroduce numeric arity storage");

assertAccessorsMatchSignature("selfhost_builtin_function_arg_count");
assertAccessorsMatchSignature("selfhost_builtin_function_arg_kind");
assertAccessorsMatchSignature("selfhost_builtin_function_result_kind");

console.log("selfhost builtin signature payload regression passed");

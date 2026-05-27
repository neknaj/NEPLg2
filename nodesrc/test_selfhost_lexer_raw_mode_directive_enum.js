#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const lexerPaths = [
    "stdlib/neplg2/core/syntax/lexer.nepl",
    "stdlib/neplg2/core/syntax/lexer/raw_mode.nepl",
    "stdlib/neplg2/core/syntax/lexer/directive.nepl",
    "stdlib/neplg2/core/syntax/lexer/indent.nepl",
    "stdlib/neplg2/core/syntax/lexer/tokenize.nepl",
];

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);

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
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?enum\\s+${enumName}:$`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${enumName} enum not found`);

    const variants = [];
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            break;
        }
        const match = /^    ([A-Za-z][A-Za-z0-9_]*)$/.exec(lines[i]);
        if (match) {
            variants.push(match[1]);
        }
    }
    assert.ok(variants.length > 0, `${enumName} variants must be discovered`);
    return variants;
}

function assertEnumVariants(src, enumName, expected) {
    assert.deepEqual(enumVariants(src, enumName), expected, `${enumName} variants changed unexpectedly`);
}

function assertFunctionCoversEnum(src, functionName, enumName, variants) {
    const block = functionBlock(src, functionName);
    assert.match(block, new RegExp(`\\bmatch\\s+[^\\n]*${enumName === "SelfhostLexerRawMode" ? "raw_mode" : ""}`));
    assert.doesNotMatch(block, /^\s*_:/m, `${functionName} must not use wildcard arms for ${enumName}`);
    for (const variant of variants) {
        assert.match(
            block,
            new RegExp(`^\\s*${enumName}::${variant}:\\s*$`, "m"),
            `${functionName} is missing ${enumName}::${variant}`,
        );
    }
}

const lexer = lexerPaths.map(read).join("\n");
const lexerCode = lexer
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join("\n");

const rawModeVariants = ["None", "Wasm", "LlvmIr"];
const directiveVariants = [
    "Entry",
    "IndentWidth",
    "Target",
    "Import",
    "Use",
    "IfTarget",
    "IfProfile",
    "Capability",
    "Wasm",
    "LlvmIr",
    "Include",
    "Extern",
    "Intrinsic",
    "Prelude",
    "NoPrelude",
    "Unknown",
];

assertEnumVariants(lexer, "SelfhostLexerRawMode", rawModeVariants);
assertEnumVariants(lexer, "SelfhostLexerDirectiveKind", directiveVariants);

assert.doesNotMatch(
    lexer,
    /\bfn\s+lex_token_pending_raw_mode\s+<\(SelfhostToken\)->i32>/,
    "raw block pending mode must not be encoded as i32",
);
assert.doesNotMatch(lexer, /\bfn\s+lex_raw_kind\s+<\(i32\)->TokenKind>/, "raw mode kind mapping must be typed");
assert.doesNotMatch(
    lexer,
    /\bfn\s+lex_raw_token\s+<\(str,i32,i32,i32,i32\)->SelfhostToken>/,
    "raw token construction must accept SelfhostLexerRawMode",
);
assert.doesNotMatch(lexer, /\bgt\s+(?:raw_mode|pending_raw_mode)\s+0\b/, "raw mode activity must not use numeric comparison");
assert.doesNotMatch(lexer, /\blet\s+token_pending\s+<i32>/, "token pending raw mode must be typed");
assert.doesNotMatch(lexer, /\blet\s+next_pending\s+<i32>/, "next pending raw mode must be typed");
assert.doesNotMatch(
    lexer,
    /\blex_all_loop\b[^\n]*\b(?:raw_mode|pending_raw_mode)\b[^\n]*\s0\s*$/,
    "lex_all_loop must not receive numeric raw mode state",
);
assert.doesNotMatch(
    lexerCode,
    /\bfield::get\s+stack\s+"data"\b|\bget\s+stack\s+"data"\b/,
    "self-host lexer must not read Vec.data directly; use Vec owner APIs instead",
);

const stackDropTop = functionBlock(lexer, "lex_stack_drop_top");
assert.match(
    stackDropTop,
    /\bdrop_last\s+stack\b/,
    "lex_stack_drop_top must drop the indent stack top through the public Vec owner API",
);

assertFunctionCoversEnum(lexer, "lex_raw_mode_is_active", "SelfhostLexerRawMode", rawModeVariants);
assertFunctionCoversEnum(lexer, "lex_raw_kind", "SelfhostLexerRawMode", rawModeVariants);
assert.doesNotMatch(
    functionBlock(lexer, "lex_raw_kind"),
    /if:\s*\n\s*eq\s+raw_mode\s+1[\s\S]*TokenKind::LlvmIrText/,
    "lex_raw_kind must not map non-Wasm modes through numeric fallback logic",
);

const directiveToken = functionBlock(lexer, "lex_directive_token");
assert.match(
    directiveToken,
    /\bmatch\s+lex_directive_kind_at\s+source\s+n\s+start:/,
    "lex_directive_token must dispatch through SelfhostLexerDirectiveKind",
);
assert.doesNotMatch(directiveToken, /^\s*_:/m, "lex_directive_token must not use wildcard arms for directives");
for (const variant of directiveVariants) {
    assert.match(
        directiveToken,
        new RegExp(`^\\s*SelfhostLexerDirectiveKind::${variant}:\\s*$`, "m"),
        `lex_directive_token is missing SelfhostLexerDirectiveKind::${variant}`,
    );
}

const directiveKindAt = functionBlock(lexer, "lex_directive_kind_at");
assert.match(
    directiveKindAt,
    /\bmatch\s+lex_directive_match_key_at\s+source\s+n\s+start:/,
    "lex_directive_kind_at must keep byte-key bucketing separate from directive semantics",
);
assert.doesNotMatch(
    directiveToken,
    /\blex_directive_word_at\b|\bstring_search::str_starts_with_at\b/,
    "lex_directive_token must not inline string/prefix classification",
);

console.log("selfhost lexer raw mode and directive enum regression passed");

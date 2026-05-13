#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(file, name) {
    const src = read(file);
    const lines = src.split("\n");
    const start = lines.findIndex((line) =>
        line.startsWith(`fn ${name} `) || line.startsWith(`pub fn ${name} `)
    );
    assert.notEqual(start, -1, `${name} not found in ${file}`);

    const topLevelDef = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevelDef.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function enumVariants(file, enumName) {
    const src = read(file);
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?enum\\s+${enumName}:$`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${enumName} enum not found`);

    const variants = [];
    for (let i = start + 1; i < lines.length; i += 1) {
        const line = lines[i];
        if (/^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/.test(line)) {
            break;
        }
        const match = /^    ([A-Za-z][A-Za-z0-9_]*)$/.exec(line);
        if (match) {
            variants.push(match[1]);
        }
    }
    assert.ok(variants.length > 0, `${enumName} variants must be discovered`);
    return variants;
}

function assertTokenKindExhaustiveMatch(block, label, variants) {
    assert.match(block, /\bmatch\s+kind:/, `${label} must dispatch directly on TokenKind`);
    assert.doesNotMatch(block, /^\s*_:/m, `${label} must not use wildcard arms for TokenKind`);
    for (const variant of variants) {
        assert.match(
            block,
            new RegExp(`^\\s*TokenKind::${variant}:\\s*$`, "m"),
            `${label} is missing TokenKind::${variant}`,
        );
    }
}

const parser = read("stdlib/neplg2/core/syntax/parser/module_parser.nepl");
const tokenKindVariants = enumVariants("stdlib/neplg2/core/syntax/token.nepl", "TokenKind");
const actionVariants = enumVariants(
    "stdlib/neplg2/core/syntax/parser/module_parser.nepl",
    "SelfhostParserTokenAction",
);

assert.doesNotMatch(
    parser,
    /#import\s+"alloc\/hash\/hash32"/,
    "module_parser must not import hash32 for TokenKind classification",
);
assert.doesNotMatch(
    parser,
    /\bselfhost_parser_string_match_key\b|\bselfhost_parser_item_kind_from_name\b|\bhash32\b/,
    "module_parser must not classify TokenKind through string/hash keys",
);
assert.doesNotMatch(
    parser,
    /\btoken_kind_name\b/,
    "module_parser must keep token_kind_name at reporting/parity boundaries, not parser dispatch",
);

assertTokenKindExhaustiveMatch(
    functionBlock("stdlib/neplg2/core/syntax/parser/module_parser.nepl", "selfhost_parser_token_action"),
    "selfhost_parser_token_action",
    tokenKindVariants,
);

assertTokenKindExhaustiveMatch(
    functionBlock("stdlib/neplg2/core/syntax/parser/module_parser.nepl", "selfhost_parser_item_kind_from_token"),
    "selfhost_parser_item_kind_from_token",
    tokenKindVariants,
);

const moduleLoop = functionBlock("stdlib/neplg2/core/syntax/parser/module_parser.nepl", "selfhost_parse_module_loop");
assert.match(
    moduleLoop,
    /\bmatch\s+selfhost_parser_token_action\s+kind:/,
    "selfhost_parse_module_loop must dispatch through typed parser token actions",
);
assert.doesNotMatch(moduleLoop, /^\s*_:/m, "selfhost_parse_module_loop must not use wildcard action arms");
for (const variant of actionVariants) {
    assert.match(
        moduleLoop,
        new RegExp(`^\\s*SelfhostParserTokenAction::${variant}:\\s*$`, "m"),
        `selfhost_parse_module_loop is missing SelfhostParserTokenAction::${variant}`,
    );
}

console.log("selfhost parser TokenKind match regression passed");

#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { readRepoFile } = require("./selfhost_module_parser_sources");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return readRepoFile(repoRoot, rel);
}

function functionBlock(src, name) {
    const lines = src.split(/\r?\n/);
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
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

const diag = read("stdlib/neplg2/core/infra/diag/code.nepl");
const state = read("stdlib/neplg2/core/syntax/parser/module_parser/state.nepl");
const diagnostic = read("stdlib/neplg2/core/syntax/parser/module_parser/diagnostic.nepl");
const loop = read("stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl");

assert.match(
    diag,
    /pub enum SelfhostParserDiagnosticCode:[\s\S]*RawBlockExpectedIndent[\s\S]*RawTextOutsideBlock[\s\S]*RawBlockUnclosed[\s\S]*InvalidDedent/,
    "parser invalid states must have typed diagnostic variants",
);
assert.match(
    functionBlock(diag, "selfhost_parser_diag_code_name"),
    /SelfhostParserDiagnosticCode::RawBlockUnclosed:[\s\S]*"parser\.raw_block\.unclosed"[\s\S]*SelfhostParserDiagnosticCode::InvalidDedent:[\s\S]*"parser\.indent\.invalid_dedent"/,
    "parser invalid-state diagnostic variants must have stable code names",
);
assert.doesNotMatch(
    state,
    /pub fn selfhost_parser_depth_dec\b/,
    "parser must not keep the saturating depth decrement helper",
);
assert.match(
    state,
    /pub fn selfhost_parser_depth_after_dedent %fn i32 Option i32/,
    "dedent transition must return typed absence for invalid top-level dedent",
);
assert.match(
    functionBlock(state, "selfhost_parser_depth_after_dedent"),
    /gt\s+depth\s+0[\s\S]*some\s+sub\s+depth\s+1[\s\S]*else:[\s\S]*none/,
    "dedent transition must reject depth zero instead of saturating to zero",
);
assert.match(
    diagnostic,
    /selfhost_parser_unclosed_raw_block_error[\s\S]*SelfhostParserDiagnosticCode::RawBlockUnclosed/,
    "EOF inside active raw mode must report RawBlockUnclosed",
);
assert.match(
    diagnostic,
    /selfhost_parser_invalid_dedent_error[\s\S]*SelfhostParserDiagnosticCode::InvalidDedent/,
    "top-level excess dedent must report InvalidDedent",
);
assert.match(
    functionBlock(loop, "selfhost_parse_dedent"),
    /selfhost_parser_depth_after_dedent\s+depth[\s\S]*Option::Some\s+next_depth[\s\S]*selfhost_parse_module_loop[\s\S]*Option::None:[\s\S]*selfhost_parser_invalid_dedent_error/,
    "normal-mode dedent must use the typed depth transition and reject invalid dedent",
);
assert.match(
    functionBlock(loop, "selfhost_parse_module_loop"),
    /ge\s+idx\s+n[\s\S]*selfhost_parser_raw_mode_is_pending\s+mode[\s\S]*selfhost_parser_pending_end_error\s+ast[\s\S]*selfhost_parser_raw_mode_is_active\s+mode[\s\S]*selfhost_parser_unclosed_raw_block_end_error\s+ast/,
    "token stream exhaustion without EOF must not accept pending or active raw parser state",
);
assert.match(
    functionBlock(loop, "selfhost_parse_module_loop"),
    /SelfhostParserTokenAction::End:[\s\S]*selfhost_parser_raw_mode_is_pending\s+mode[\s\S]*selfhost_parser_pending_error\s+&token\s+ast[\s\S]*selfhost_parser_raw_mode_is_active\s+mode[\s\S]*selfhost_parser_unclosed_raw_block_error\s+&token\s+ast/,
    "EOF token must not accept pending or active raw parser state",
);

console.log("selfhost parser invalid state contract passed");

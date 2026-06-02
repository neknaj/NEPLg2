#!/usr/bin/env node
// nodesrc/test_selfhost_lexer_rust_parity.js
// 目的:
// - self-host lexer が追従する Rust analyze_lex JSON の代表 token stream を固定する。
// - kind / value / byte span を正規化し、directive / keyword / literal / doc / raw block の差分を
//   parser 実装前に lexer issue として切り分けられるようにする。

const assert = require('node:assert/strict');
const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');

function tok(kind, start, end, value = null) {
    return { kind, start, end, value };
}

function normalizeToken(token) {
    return {
        kind: token?.kind || '',
        start: Number(token?.span?.start ?? -1),
        end: Number(token?.span?.end ?? -1),
        value: Object.prototype.hasOwnProperty.call(token || {}, 'value') ? String(token.value) : null,
    };
}

function assertTokenStream(id, actual, expected) {
    assert.equal(actual.length, expected.length, `${id}: token length`);
    for (let i = 0; i < expected.length; i++) {
        assert.deepEqual(actual[i], expected[i], `${id}: token[${i}]`);
    }
}

const fixtures = [
    {
        id: 'directives_keywords_literals',
        source: `#target core
#import "std/test" as *
#use "std/prelude"
#if[target=core]
#if[profile=debug]
#capability io
#prelude "std/prelude"
#no_prelude
#intrinsic "unreachable" <> ()
fn main %fn void i32 \\void:
    let mut x 0x2a;
    set x 1.5;
    if cond true then 'a' else "s"
    Result::Ok x
`,
        expected: [
            tok('DirTarget', 0, 11, 'core'),
            tok('Newline', 12, 12),
            tok('DirImport', 13, 35, '"std/test" as *'),
            tok('Newline', 36, 36),
            tok('DirUse', 37, 54, '"std/prelude"'),
            tok('Newline', 55, 55),
            tok('DirIfTarget', 56, 71, 'core'),
            tok('Newline', 72, 72),
            tok('DirIfProfile', 73, 90, 'debug'),
            tok('Newline', 91, 91),
            tok('DirCapability', 92, 105, 'io'),
            tok('Newline', 106, 106),
            tok('DirPrelude', 107, 128, '"std/prelude"'),
            tok('Newline', 129, 129),
            tok('DirNoPrelude', 130, 140),
            tok('Newline', 141, 141),
            tok('DirIntrinsic', 142, 152),
            tok('StringLiteral', 153, 166, 'unreachable'),
            tok('LAngle', 167, 168),
            tok('RAngle', 168, 169),
            tok('LParen', 170, 171),
            tok('RParen', 171, 172),
            tok('Newline', 172, 172),
            tok('KwFn', 173, 175),
            tok('Ident', 176, 180, 'main'),
            tok('Percent', 181, 182),
            tok('KwFn', 182, 184),
            tok('VoidMarker', 185, 189),
            tok('Ident', 190, 193, 'i32'),
            tok('Backslash', 194, 195),
            tok('VoidMarker', 195, 199),
            tok('Colon', 199, 200),
            tok('Newline', 200, 200),
            tok('Indent', 201, 201),
            tok('KwLet', 205, 208),
            tok('KwMut', 209, 212),
            tok('Ident', 213, 214, 'x'),
            tok('IntLiteral', 215, 219, '0x2a'),
            tok('Semicolon', 219, 220),
            tok('Newline', 220, 220),
            tok('KwSet', 225, 228),
            tok('Ident', 229, 230, 'x'),
            tok('FloatLiteral', 231, 234, '1.5'),
            tok('Semicolon', 234, 235),
            tok('Newline', 235, 235),
            tok('KwIf', 240, 242),
            tok('KwCond', 243, 247),
            tok('BoolLiteral', 248, 252, 'true'),
            tok('KwThen', 253, 257),
            tok('CharLiteral', 258, 261, '97'),
            tok('KwElse', 262, 266),
            tok('StringLiteral', 267, 270, 's'),
            tok('Newline', 270, 270),
            tok('Ident', 275, 281, 'Result'),
            tok('PathSep', 281, 283),
            tok('Ident', 283, 285, 'Ok'),
            tok('Ident', 286, 287, 'x'),
            tok('Newline', 287, 287),
            tok('Dedent', 288, 288),
            tok('Eof', 288, 288),
        ],
    },
    {
        id: 'doc_mlstr_and_raw_blocks',
        source: `//: doc
##: text
#wasm:
    local.get 0
#llvmir:
    ret i32 0
`,
        expected: [
            tok('DocComment', 0, 7, 'doc'),
            tok('Newline', 0, 0),
            tok('MlstrLine', 8, 16, 'text'),
            tok('Newline', 16, 16),
            tok('DirWasm', 17, 22),
            tok('Newline', 23, 23),
            tok('Indent', 24, 24),
            tok('WasmText', 28, 39, 'local.get 0'),
            tok('Newline', 39, 39),
            tok('Dedent', 40, 40),
            tok('DirLlvmIr', 40, 47),
            tok('Newline', 48, 48),
            tok('Indent', 49, 49),
            tok('LlvmIrText', 53, 62, 'ret i32 0'),
            tok('Newline', 62, 62),
            tok('Dedent', 63, 63),
            tok('Eof', 63, 63),
        ],
    },
];

async function main() {
    const loaded = await loadCompilerFromCandidates(candidateDistDirs(''));
    const api = loaded.api;
    assert.equal(typeof api.analyze_lex, 'function', 'analyze_lex API is required');

    for (const fixture of fixtures) {
        const result = api.analyze_lex(fixture.source);
        assert.equal(result?.stage, 'lex', `${fixture.id}: stage`);
        assert.equal(result?.ok, true, `${fixture.id}: ok`);
        assert.deepEqual(result?.diagnostics || [], [], `${fixture.id}: diagnostics`);
        const actual = Array.isArray(result?.tokens) ? result.tokens.map(normalizeToken) : [];
        assertTokenStream(fixture.id, actual, fixture.expected);
    }

    console.log(JSON.stringify({
        ok: true,
        fixtures: fixtures.length,
        total_tokens: fixtures.reduce((n, f) => n + f.expected.length, 0),
    }, null, 2));
}

main().catch((e) => {
    console.error(String(e?.stack || e?.message || e));
    process.exit(1);
});

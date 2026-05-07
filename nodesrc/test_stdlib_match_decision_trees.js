#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function functionBlock(file, name) {
    const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
    const lines = src.split(/\r?\n/);
    const start = lines.findIndex((line) =>
        line.startsWith(`fn ${name} `) || line.startsWith(`pub fn ${name} `)
    );
    assert.notEqual(start, -1, `${name} not found in ${file}`);

    const topLevelDef = /^(?:pub\s+)?(?:fn|struct|enum)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevelDef.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join('\n');
}

function assertLiteralMatch({ file, name, scrutinee, literals }) {
    const block = functionBlock(file, name);
    assert.match(block, new RegExp(`\\bmatch\\s+${scrutinee}:`), `${name} must dispatch with match`);
    assert.doesNotMatch(block, /^\s+if:\s*$/m, `${name} must not regress to an if decision tree`);
    for (const literal of literals) {
        const escaped = String(literal).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        assert.match(block, new RegExp(`^\\s*${escaped}:\\s*$`, 'm'), `${name} is missing literal arm ${literal}`);
    }
    assert.match(block, /^\s*_:\s*$/m, `${name} must keep an explicit wildcard/default arm`);
}

function assertHasLiteralMatch({ file, name, scrutinee, literals }) {
    const block = functionBlock(file, name);
    assert.match(block, new RegExp(`\\bmatch\\s+${scrutinee}:`), `${name} must dispatch with match`);
    for (const literal of literals) {
        const escaped = String(literal).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        assert.match(block, new RegExp(`^\\s*${escaped}:\\s*$`, 'm'), `${name} is missing literal arm ${literal}`);
    }
    assert.match(block, /^\s*_:\s*$/m, `${name} must keep an explicit wildcard/default arm`);
}

function assertScalarKeyMatch({ file, name, scrutinee, literals }) {
    const block = functionBlock(file, name);
    assert.match(block, scrutinee, `${name} must dispatch through a scalar match key`);
    assert.doesNotMatch(block, /^\s+if:\s*$/m, `${name} must not regress to an if decision tree`);
    for (const literal of literals) {
        assert.match(block, new RegExp(`^\\s*${literal}:\\s*$`, 'm'), `${name} is missing key arm ${literal}`);
    }
    assert.match(block, /^\s*_:\s*$/m, `${name} must keep an explicit wildcard/default arm`);
}

assertLiteralMatch({
    file: 'stdlib/alloc/encoding/json/escape.nepl',
    name: 'json_escape_kind',
    scrutinee: 'ch',
    literals: ["'\\\\'", "'\"'", "'\\n'", "'\\r'", "'\\t'", "'\\b'", "'\\f'"],
});

const nmJsonEscapeFile = 'stdlib/nm/json_escape.nepl';

const nmJsonEscapeBlock = functionBlock(nmJsonEscapeFile, 'json_escape');
assert.match(
    nmJsonEscapeBlock,
    /json_escape_into\s+string_builder_new\s+s/,
    'json_escape must delegate through the builder escape boundary'
);

const nmJsonEscapeIntoBlock = functionBlock(nmJsonEscapeFile, 'json_escape_into');
assert.match(
    nmJsonEscapeIntoBlock,
    /json_escape_mem_into\s+sb\s+string_data_ptr\s+s\s+len\s+s/,
    'json_escape_into must delegate through the byte-range escape boundary'
);

const nmJsonEscapeMemIntoBlock = functionBlock(nmJsonEscapeFile, 'json_escape_mem_into');
assert.match(
    nmJsonEscapeMemIntoBlock,
    /json_escape_byte_into\s+out\s+ch/,
    'json_escape_mem_into must dispatch each byte through json_escape_byte_into'
);

assertLiteralMatch({
    file: nmJsonEscapeFile,
    name: 'json_escape_byte_into',
    scrutinee: 'ch',
    literals: ["'\\\\'", "'\"'", "'\\n'", "'\\r'", "'\\t'", "'\\b'", "'\\f'"],
});

assertLiteralMatch({
    file: 'stdlib/nm/html_escape.nepl',
    name: 'html_escape_kind',
    scrutinee: 'ch',
    literals: ["'&'", "'<'", "'>'", "'\"'", "'\\''"],
});

assertLiteralMatch({
    file: 'stdlib/nm/html_heading.nepl',
    name: 'html_heading_kind',
    scrutinee: 'level',
    literals: [1, 2, 3, 4, 5],
});

assertLiteralMatch({
    file: 'stdlib/alloc/string/search/compare.nepl',
    name: 'str_is_space',
    scrutinee: 'b',
    literals: ["' '", "'\\t'", "'\\n'", "'\\r'"],
});

assertLiteralMatch({
    file: 'stdlib/std/streamio/scanner/cursor.nepl',
    name: 'stream_scanner_is_leading_skip_byte',
    scrutinee: 'byte',
    literals: ["'\\0'", "' '", "'\\n'", "'\\r'", "'\\t'"],
});

assertLiteralMatch({
    file: 'stdlib/std/streamio/scanner/cursor.nepl',
    name: 'stream_scanner_is_token_separator',
    scrutinee: 'byte',
    literals: ["' '", "'\\n'", "'\\r'", "'\\t'"],
});

assertLiteralMatch({
    file: 'stdlib/std/streamio/scanner/cursor.nepl',
    name: 'stream_scanner_is_exponent_marker',
    scrutinee: 'byte',
    literals: ["'e'", "'E'"],
});

assertHasLiteralMatch({
    file: 'stdlib/neplg2/core/infra/text.nepl',
    name: 'source_text_trim_line_end',
    scrutinee: 'last',
    literals: ["'\\n'", "'\\r'"],
});

assertLiteralMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind',
    scrutinee: 'string_access::len lexeme',
    literals: [2, 3, 4, 5, 6, 8],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len2',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [26222, 26982, 25711],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len3',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [27749, 28021, 29541, 26223, 28789],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len4',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [25455, 29800, 25964, 25966, 26989, 29810],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len5',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [30568, 28001, 25196, 21621, 28012, 26209],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len6',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [29556, 29810],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/core/syntax/lexer.nepl',
    name: 'lex_keyword_kind_len8',
    scrutinee: /\bmatch\s+lex_keyword_match_key\s+lexeme:/,
    literals: [28271],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/cli/args/classify.nepl',
    name: 'selfhost_cli_arg_kind',
    scrutinee: /\bmatch\s+selfhost_cli_string_match_key\s+arg:/,
    literals: [
        1105069866, 236190827, 1825648204, 1059669684, 829133248,
        1018434800, 1600605192, 559754234, 1262805978, 1216879227,
        1726806915, 296239959, 390736299, 163286432, 1353831004,
    ],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/cli/args/classify.nepl',
    name: 'selfhost_cli_parse_target_value',
    scrutinee: /\bmatch\s+selfhost_cli_string_match_key\s+value:/,
    literals: [1210069335, 495192238, 53171433, 1580422520, 37322532, 343592226],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/cli/args/classify.nepl',
    name: 'selfhost_cli_parse_emit_value',
    scrutinee: /\bmatch\s+selfhost_cli_string_match_key\s+value:/,
    literals: [1210069335, 139754043, 149843404, 343592226, 1495051790, 688645933],
});

assertScalarKeyMatch({
    file: 'stdlib/neplg2/cli/args/classify.nepl',
    name: 'selfhost_cli_parse_profile_value',
    scrutinee: /\bmatch\s+selfhost_cli_string_match_key\s+value:/,
    literals: [97528863, 322158401],
});

console.log('stdlib match decision tree regression passed');

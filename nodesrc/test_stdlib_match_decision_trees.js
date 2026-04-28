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

assertLiteralMatch({
    file: 'stdlib/alloc/encoding/json.nepl',
    name: 'json_escape_kind',
    scrutinee: 'ch',
    literals: ["'\\\\'", "'\"'", "'\\n'", "'\\r'", "'\\t'", "'\\b'", "'\\f'"],
});

assertLiteralMatch({
    file: 'stdlib/nm/parser.nepl',
    name: 'nm_json_escape_kind',
    scrutinee: 'ch',
    literals: ["'\\\\'", "'\"'", "'\\n'", "'\\r'", "'\\t'", "'\\b'", "'\\f'"],
});

assertLiteralMatch({
    file: 'stdlib/nm/html_gen.nepl',
    name: 'html_escape_kind',
    scrutinee: 'ch',
    literals: ["'&'", "'<'", "'>'", "'\"'", "'\\''"],
});

assertLiteralMatch({
    file: 'stdlib/nm/html_gen.nepl',
    name: 'html_heading_kind',
    scrutinee: 'level',
    literals: [1, 2, 3, 4, 5],
});

assertLiteralMatch({
    file: 'stdlib/alloc/string.nepl',
    name: 'str_is_space',
    scrutinee: 'b',
    literals: ["' '", "'\\t'", "'\\n'", "'\\r'"],
});

console.log('stdlib match decision tree regression passed');

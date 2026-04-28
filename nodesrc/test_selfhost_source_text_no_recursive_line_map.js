#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const file = 'stdlib/neplg2/core/infra/text.nepl';
const src = fs.readFileSync(path.join(repoRoot, file), 'utf8');
const lines = src.split(/\r?\n/);

function functionBlock(name) {
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

const collect = functionBlock('source_text_collect_line_starts');
const withoutSignature = collect.split(/\r?\n/).slice(1).join('\n');

assert.match(
    collect,
    /^\s+while\s+and\s+lt\s+i\s+n\s+not\s+failed:\s*$/m,
    'source_text_collect_line_starts must scan with an explicit loop'
);
assert.doesNotMatch(
    withoutSignature,
    /\bsource_text_collect_line_starts\b/,
    'source_text_collect_line_starts must not recurse per input byte'
);

console.log('selfhost source text line map recursion regression passed');

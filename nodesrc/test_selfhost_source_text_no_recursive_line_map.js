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
const newSourceText = functionBlock('source_text_new');
const pushLineStart = functionBlock('source_text_push_line_start');
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
assert.match(
    src,
    /enum\s+SourceTextLineStartPushState:[\s\S]*\bOk\b[\s\S]*\bErr\b/,
    'source text line start push state must be an enum so branch coverage is statically visible'
);
assert.match(
    src,
    /struct\s+SourceTextLineStartPush:[\s\S]*state\s+<SourceTextLineStartPushState>[\s\S]*starts\s+<Vec<i32>>/,
    'source text line start push outcome must carry the returned Vec owner explicitly'
);
assert.match(
    pushLineStart,
    /Result::Err\s+e:[\s\S]*SourceTextLineStartPush\s+SourceTextLineStartPushState::Err\s+v::vec_push_error_vec<i32>\s+e/,
    'source_text_push_line_start must return the Vec owner carried by VecPushError on push failure'
);
assert.doesNotMatch(
    pushLineStart,
    /Result::Err[\s\S]*SourceTextLineStartPush\s+SourceTextLineStartPushState::Err\s+v::vec_empty<i32>/,
    'source_text_push_line_start must not hide the failed input owner behind a fresh empty Vec'
);
assert.match(
    collect,
    /\bsource_text_push_line_start\b[\s\S]*SourceTextLineStartPushState::Err:[\s\S]*set\s+out\s+next_out[\s\S]*set\s+failed\s+true/,
    'source_text_collect_line_starts must reinitialize the loop Vec owner on push failure'
);
assert.match(
    collect,
    /failed[\s\S]*then:[\s\S]*v::free<i32>\s+out[\s\S]*Result<Vec<i32>,\s*StdErrorKind>::Err\s+StdErrorKind::OutOfMemory/,
    'source_text_collect_line_starts must close the replacement Vec owner before returning Err'
);
assert.match(
    newSourceText,
    /match\s+v::filled<i32>\s+1\s+0:/,
    'source_text_new must build the initial line-start table without a separate consuming push'
);

console.log('selfhost source text line map recursion regression passed');

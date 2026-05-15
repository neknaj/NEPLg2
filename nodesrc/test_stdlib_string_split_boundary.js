#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments, implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const splitRelPath = 'stdlib/alloc/string/split.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const splitSrc = fs.readFileSync(path.join(repoRoot, splitRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const splitCode = stripNeplComments(splitSrc);

assert.match(rootSrc, /pub #import "\.\/string\/split" as \*/, 'alloc/string facade must re-export string/split');
assert.match(splitSrc, /#import "alloc\/string\/access" as \*/, 'string/split must use string/access for byte lengths');
assert.match(splitSrc, /#import "alloc\/string\/search" as \*/, 'string/split must use string/search for separator matching');

for (const name of [
    'str_split_count',
    'str_split_done_step',
    'str_split_next',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(splitCode, new RegExp(`fn\\s+${name}\\b`), `${splitRelPath} must own ${name}`);
}

for (const name of [
    'StrSplitStepKind',
    'StrSplitStep',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(splitCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `${splitRelPath} must own ${name}`);
}

assert.match(
    splitCode,
    /enum\s+StrSplitStepKind:[\s\S]*Part[\s\S]*Done/,
    'split scanner state must stay an enum so callers get match exhaustiveness checks',
);
assert.match(
    splitCode,
    /fn\s+str_split_done_step[\s\S]*StrSplitStepKind::Done/,
    'split scanner completion must be represented by the Done enum state',
);
assert.match(
    splitCode,
    /fn\s+str_split_next[\s\S]*StrSplitStepKind::Part[\s\S]*str_split_done_step/,
    'str_split_next must return explicit Part/Done states instead of external numeric sentinels',
);
assert.doesNotMatch(
    splitCode,
    /Result<Vec<str>|Result<Vec<i32>/,
    'string/split must not reintroduce allocation-bearing split result vectors',
);
assert.ok(implementationLineCount(splitSrc) <= 180, `${splitRelPath} should stay narrowly scoped`);

console.log('alloc/string split boundary regression passed');

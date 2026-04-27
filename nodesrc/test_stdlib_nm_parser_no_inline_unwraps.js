#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/nm/parser.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const start = code.indexOf('fn parse_inlines <(str)->Vec<Inline>> (s):');
const end = code.indexOf('pub fn parse_markdown <(str)->Document> (input):', start);
assert.notEqual(start, -1, 'parse_inlines must exist');
assert.notEqual(end, -1, 'parse_inlines section boundary must exist');
const parseInlines = code.slice(start, end);

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(parseInlines, pattern, 'parse_inlines must not use unsafe unwrap helpers');
}

assert.match(code, /struct\s+InlinePushRes:[\s\S]*items\s+<Vec<Inline>>[\s\S]*ok\s+<bool>/, 'Inline push result must carry Vec and status');
assert.match(code, /struct\s+StrPushRes:[\s\S]*items\s+<Vec<str>>[\s\S]*ok\s+<bool>/, 'String push result must carry Vec and status');
assert.match(code, /fn\s+nm_inline_empty_vec\s+<\(\)->Vec<Inline>>\s+\(\):\s+v::Vec<Inline>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'inline allocation failure must use an empty Vec sentinel');
assert.match(code, /fn\s+nm_push_inline\s+<\(Vec<Inline>, Inline\)->InlinePushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<Inline>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*InlinePushRes\s+nm_inline_empty_vec\s+false/, 'inline pushes must convert grow failure to ok=false');
assert.match(code, /fn\s+nm_push_str\s+<\(Vec<str>, str\)->StrPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*StrPushRes\s+nm_str_empty_vec\s+false/, 'string pushes must convert grow failure to ok=false');
assert.match(parseInlines, /match\s+v::new<Inline>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'parse_inlines must handle Vec<Inline> allocation failure');
assert.match(parseInlines, /while\s+and\s+lt\s+i\s+n\s+not\s+failed:/, 'parse_inlines must stop scanning after allocation failure');
assert.match(parseInlines, /let\s+pushed_tail\s+<InlinePushRes>\s+nm_push_inline\s+out\s+Inline::Text\s+tail/, 'tail text flush must go through checked push helper');

console.log('stdlib nm parser inline unsafe unwrap regression passed');

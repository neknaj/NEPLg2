#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/diag/error.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code, /struct\s+Diag:[\s\S]*notes\s+<str>[\s\S]*help\s+<str>/, 'Diag notes/help must use string fields in the current compact diagnostic layout');
assert.doesNotMatch(code, /struct\s+Diag:[\s\S]*notes\s+<Vec<str>>/, 'Diag notes must not silently reintroduce Vec<str> storage without a checked fallback policy');
assert.doesNotMatch(code, /struct\s+Diag:[\s\S]*help\s+<Vec<str>>/, 'Diag help must not silently reintroduce Vec<str> storage without a checked fallback policy');
assert.match(code, /fn\s+diag_empty_diag_vec\s+<\(\)->Vec<Diag>>\s+\(\):\s+v::Vec<Diag>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'Diags allocation fallback must use an empty sentinel');
assert.match(code, /fn\s+diag_new\s+<\(DiagKind,str\)\*>Diag>\s+\(kind,\s*message\):\s+Diag\s+kind\s+message\s+none<Span>\s+""\s+""\s+none<str>/, 'diag_new must initialize note/help text without allocating Vec<str>');
assert.match(code, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*note\):[\s\S]*let\s+notes1\s+<str>\s+concat\s+notes0\s+note;[\s\S]*let\s+notes2\s+<str>\s+concat\s+notes1\s+"\\n";[\s\S]*Diag[\s\S]*notes2/, 'diag_add_note must append to note text without Vec<str> push unwraps');
assert.match(code, /fn\s+diag_add_help\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*help_item\):[\s\S]*let\s+help1\s+<str>\s+concat\s+help0\s+"help: ";[\s\S]*let\s+help3\s+<str>\s+concat\s+help2\s+"\\n";[\s\S]*Diag[\s\S]*help3/, 'diag_add_help must append to help text without Vec<str> push unwraps');
assert.match(code, /fn\s+diags_one\s+<\(Diag\)\*>Diags>\s+\(d\):[\s\S]*match\s+v::push<Diag>\s+items0\s+d:[\s\S]*Result::Err\s+_e:[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_one must convert push failure to an empty Diags sentinel');
assert.match(code, /fn\s+diags_push\s+<\(Diags,Diag\)\*>Diags>\s+\(ds,\s*d\):[\s\S]*match\s+v::push<Diag>\s+field::get\s+ds\s+"items"\s+d:[\s\S]*Result::Err\s+_e:[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_push must convert grow failure to an empty Diags sentinel');

console.log('stdlib diag error unsafe unwrap regression passed');

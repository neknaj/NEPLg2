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
assert.match(code, /fn\s+diag_empty_diag_vec\s+<\(\)->Vec<Diag>>\s+\(\):\s+v::vec_empty<Diag>/, 'Diags allocation fallback must use typed empty Vec storage');
assert.match(code, /fn\s+diag_new\s+<\(DiagKind,str\)\*>Diag>\s+\(kind,\s*message\):\s+Diag\s+kind\s+message\s+none<Span>\s+""\s+""\s+none<str>/, 'diag_new must initialize note/help text without allocating Vec<str>');
assert.match(code, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*note\):\s+Diag[\s\S]*\snote\s+/, 'diag_add_note must store an owner-neutral note fragment directly');
assert.match(code, /fn\s+diag_add_help\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*help_item\):\s+Diag[\s\S]*\shelp_item\s+/, 'diag_add_help must store an owner-neutral help fragment directly');
assert.doesNotMatch(code, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>[\s\S]*?\n\s*fn\s+diag_add_help[\s\S]*?concat/, 'Diag note/help mutation must not build owned concatenated text blocks');
assert.match(code, /fn\s+diag_out_of_memory\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::OutOfMemory\s+"allocation failed"/, 'diag_out_of_memory must be zero-argument and static-message based');
assert.match(code, /fn\s+diag_empty_collection\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::EmptyCollection\s+"collection is empty"/, 'diag_empty_collection must be zero-argument and static-message based');
assert.match(code, /fn\s+diag_capacity_exceeded\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::CapacityExceeded\s+"capacity exceeded"/, 'diag_capacity_exceeded must be zero-argument and static-message based');
assert.match(code, /fn\s+diag_key_not_found\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::KeyNotFound\s+"key not found"/, 'diag_key_not_found must be zero-argument and static-message based');
assert.match(code, /fn\s+diags_free\s+<\(Diags\)->\(\)>\s+\(ds\):\s+v::free<Diag>\s+field::get\s+ds\s+"items"/, 'Diags must provide an explicit by-value consumption helper');
assert.match(code, /fn\s+diags_len\s+<\(Diags\)->i32>\s+\(ds\):[\s\S]*let\s+n\s+<i32>\s+diags_len\s+&ds[\s\S]*diags_free\s+ds[\s\S]*n/, 'by-value diags_len must close the Diags owner after observation');
assert.match(code, /fn\s+diags_has_errors\s+<\(Diags\)->bool>\s+\(ds\):[\s\S]*let\s+ok\s+<bool>\s+diags_has_errors\s+&ds[\s\S]*diags_free\s+ds[\s\S]*ok/, 'by-value diags_has_errors must close the Diags owner after observation');
assert.match(code, /fn\s+diags_one\s+<\(Diag\)\*>Diags>\s+\(d\):[\s\S]*match\s+v::push<Diag>\s+items0\s+d:[\s\S]*Result::Err\s+_e:[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_one must convert push failure to an empty Diags sentinel');
assert.match(code, /fn\s+diags_push\s+<\(Diags,Diag\)\*>Diags>\s+\(ds,\s*d\):[\s\S]*match\s+v::push<Diag>\s+field::get\s+ds\s+"items"\s+d:[\s\S]*Result::Err\s+_e:[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_push must convert grow failure to an empty Diags sentinel');

console.log('stdlib diag error unsafe unwrap regression passed');

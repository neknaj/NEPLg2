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

assert.match(code, /fn\s+diag_empty_str_vec\s+<\(\)->Vec<str>>\s+\(\):\s+v::Vec<str>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'Diag string Vec allocation fallback must use an empty sentinel');
assert.match(code, /fn\s+diag_empty_diag_vec\s+<\(\)->Vec<Diag>>\s+\(\):\s+v::Vec<Diag>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'Diags allocation fallback must use an empty sentinel');
assert.match(code, /fn\s+diag_push_str_vec\s+<\(Vec<str>,str\)->StrVecPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*StrVecPushRes\s+diag_empty_str_vec\s+false/, 'Diag note/help push must convert grow failure to ok=false');
assert.match(code, /fn\s+diag_push_diag_vec\s+<\(Vec<Diag>,Diag\)->DiagVecPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<Diag>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*DiagVecPushRes\s+diag_empty_diag_vec\s+false/, 'Diags push must convert grow failure to ok=false');
assert.match(code, /fn\s+diag_new\s+<\(DiagKind,str\)\*>Diag>\s+\(kind,\s*message\):[\s\S]*match\s+v::new<str>:[\s\S]*Result::Err\s+_e:[\s\S]*\(\)[\s\S]*Diag\s+kind\s+message\s+none<Span>\s+notes\s+help\s+none<str>/, 'diag_new must handle notes/help allocation failure without trapping');
assert.match(code, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*note\):[\s\S]*diag_push_str_vec\s+get\s+load<Diag>\s+d_mem\s+"notes"\s+note/, 'diag_add_note must use checked note push');
assert.match(code, /fn\s+diag_add_help\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*help_item\):[\s\S]*diag_push_str_vec\s+get\s+load<Diag>\s+d_mem\s+"help"\s+help_item/, 'diag_add_help must use checked help push');
assert.match(code, /fn\s+diags_push\s+<\(Diags,Diag\)\*>Diags>\s+\(ds,\s*d\):[\s\S]*diag_push_diag_vec\s+get\s+ds\s+"items"\s+d/, 'diags_push must use checked Diag push');

console.log('stdlib diag error unsafe unwrap regression passed');

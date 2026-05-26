#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = {
    root: 'stdlib/alloc/diag/error.nepl',
    types: 'stdlib/alloc/diag/error/types.nepl',
    diag: 'stdlib/alloc/diag/error/diag.nepl',
    diags: 'stdlib/alloc/diag/error/diags.nepl',
    outcome: 'stdlib/alloc/diag/error/outcome.nepl',
    renderer: 'stdlib/alloc/diag/diag.nepl',
};

const src = Object.fromEntries(
    Object.entries(relPaths).map(([name, relPath]) => [
        name,
        fs.readFileSync(path.join(repoRoot, relPath), 'utf8'),
    ]),
);

const code = Object.fromEntries(
    Object.entries(src).map(([name, text]) => [
        name,
        legacyTypeSyntaxView(text),
    ]),
);

const implementationCode = [code.types, code.diag, code.diags, code.outcome, code.renderer].join('\n');
const allCode = [code.root, implementationCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(allCode, pattern, `alloc/diag/error modules must not use unsafe unwrap helpers in implementation code`);
}

assert.match(code.root, /pub\s+#import\s+"\.\/*error\/types"\s+as\s+\*/, 'root facade must re-export diagnostic value types');
assert.match(code.root, /pub\s+#import\s+"\.\/*error\/diag"\s+as\s+\*/, 'root facade must re-export single-diagnostic helpers');
assert.match(code.root, /pub\s+#import\s+"\.\/*error\/diags"\s+as\s+\*/, 'root facade must re-export Diags owner helpers');
assert.match(code.root, /pub\s+#import\s+"\.\/*error\/outcome"\s+as\s+\*/, 'root facade must re-export Outcome helpers');
assert.doesNotMatch(code.root, /^\s*(fn|struct|enum|impl)\b/m, 'alloc/diag/error.nepl must stay a public facade without implementation bodies');
assert.doesNotMatch(code.diags, /#import\s+"core\/mem(?:\/(?:internal|raw))?"\s+as\b/, 'Diags owner helpers must not import raw memory modules for read-only observers');
assert.doesNotMatch(code.diags, /\b(?:mem_ptr_addr|data_mem_(?:ptr|view)|load<Diag>|size_of<Diag>)\b/, 'Diags read-only observers must not scan Vec storage through raw memory');
assert.match(code.diags, /fn\s+diags_has_errors\s+<\(&Diags\)->bool>[\s\S]*v::len\s+items[\s\S]*diags_has_errors_loop\s+items\s+items_len\s+0/, 'diags_has_errors must observe Diags through the borrowed Vec boundary');
assert.match(code.diags, /fn\s+diags_has_errors_loop\s+<\(&Vec<Diag>,i32,i32\)->bool>[\s\S]*match\s+v::get\s+items\s+i:/, 'diags_has_errors traversal must use Vec.get rather than raw loads');
assert.doesNotMatch(code.diag, /\b(?:mem_ptr_addr|load<Diag>)\b/, 'single diagnostic helpers must not carry direct raw memory evidence');
assert.doesNotMatch(code.renderer, /#import\s+"core\/mem(?:\/(?:internal|raw))?"\s+as\b/, 'diagnostic renderer must not import raw memory modules');
assert.doesNotMatch(code.renderer, /\b(?:mem_ptr_addr|data_mem_(?:ptr|view)|load<Diag>|size_of<Diag>)\b/, 'diagnostic renderer must not scan Diags through raw Vec storage');
assert.match(code.renderer, /fn\s+diags_to_string\s+<\(&Diags\)->str>[\s\S]*v::len\s+items[\s\S]*diags_to_string_loop\s+items\s+items_len\s+0\s+""/, 'diagnostic renderer must observe Diags through the borrowed Vec boundary');
assert.match(code.renderer, /fn\s+diags_to_string_loop\s+<\(&Vec<Diag>,i32,i32,str\)->str>[\s\S]*match\s+v::get\s+items\s+i:/, 'diagnostic renderer traversal must use Vec.get rather than raw loads');

const lineLimits = {
    root: 80,
    types: 230,
    diag: 220,
    diags: 190,
    outcome: 340,
    renderer: 190,
};

for (const [name, limit] of Object.entries(lineLimits)) {
    const lines = implementationLineCount(src[name]);
    assert.ok(lines <= limit, `${relPaths[name]} has ${lines} lines; split modules must stay below ${limit}`);
}

assert.match(code.types, /enum\s+DiagLevel:[\s\S]*DiagLevel::Error:[\s\S]*"error"/, 'DiagLevel must remain enum-backed and stringified through exhaustive match arms');
assert.match(code.types, /struct\s+Diag:[\s\S]*notes\s+<str>[\s\S]*help\s+<str>/, 'Diag notes/help must use string fields in the current compact diagnostic layout');
assert.doesNotMatch(code.types, /struct\s+Diag:[\s\S]*notes\s+<Vec<str>>/, 'Diag notes must not silently reintroduce Vec<str> storage without a checked fallback policy');
assert.doesNotMatch(code.types, /struct\s+Diag:[\s\S]*help\s+<Vec<str>>/, 'Diag help must not silently reintroduce Vec<str> storage without a checked fallback policy');
assert.match(code.types, /struct\s+Diags:[\s\S]*items\s+<Vec<Diag>>/, 'Diags must remain the sole Vec-backed diagnostic group owner');
assert.doesNotMatch(code.types, /struct\s+Outcome/, 'Outcome must live in the outcome module, not in the shared type/core diagnostic group module');

assert.match(code.diag, /fn\s+diag_new\s+<\(DiagKind,str\)\*>Diag>\s+\(kind,\s*message\):\s+Diag\s+kind\s+message\s+none\s+""\s+""\s+none/, 'diag_new must initialize note/help text without allocating Vec<str>');
assert.match(code.diag, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*note\):\s+Diag[\s\S]*\snote\s+/, 'diag_add_note must store an owner-neutral note fragment directly');
assert.match(code.diag, /fn\s+diag_add_help\s+<\(Diag,str\)\*>Diag>\s+\(d,\s*help_item\):\s+Diag[\s\S]*\shelp_item\s+/, 'diag_add_help must store an owner-neutral help fragment directly');
assert.doesNotMatch(code.diag, /fn\s+diag_add_note\s+<\(Diag,str\)\*>Diag>[\s\S]*?\n\s*fn\s+diag_add_help[\s\S]*?concat/, 'Diag note/help mutation must not build owned concatenated text blocks');
assert.match(code.diag, /fn\s+diag_out_of_memory\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::OutOfMemory\s+"allocation failed"/, 'diag_out_of_memory must be zero-argument and static-message based');
assert.match(code.diag, /fn\s+diag_empty_collection\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::EmptyCollection\s+"collection is empty"/, 'diag_empty_collection must be zero-argument and static-message based');
assert.match(code.diag, /fn\s+diag_capacity_exceeded\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::CapacityExceeded\s+"capacity exceeded"/, 'diag_capacity_exceeded must be zero-argument and static-message based');
assert.match(code.diag, /fn\s+diag_key_not_found\s+<\(\)\*>Diag>\s+\(\):\s+diag_error\s+StdErrorKind::KeyNotFound\s+"key not found"/, 'diag_key_not_found must be zero-argument and static-message based');

assert.match(code.diags, /fn\s+diag_empty_diag_vec\s+<\(\)->Vec<Diag>>\s+\(\):\s+v::vec_empty\b/, 'Diags allocation fallback must use typed empty Vec storage');
assert.match(code.diags, /fn\s+diags_free\s+<\(Diags\)->unit>\s+\(ds\):\s+v::free\s+field::get\s+ds\s+"items"/, 'Diags must provide an explicit by-value consumption helper');
assert.match(code.diags, /fn\s+diags_len\s+<\(Diags\)->i32>\s+\(ds\):[\s\S]*let\s+n\s+<i32>\s+diags_len\s+&ds[\s\S]*diags_free\s+ds[\s\S]*n/, 'by-value diags_len must close the Diags owner after observation');
assert.match(code.diags, /fn\s+diags_has_errors\s+<\(Diags\)->bool>\s+\(ds\):[\s\S]*let\s+ok\s+<bool>\s+diags_has_errors\s+&ds[\s\S]*diags_free\s+ds[\s\S]*ok/, 'by-value diags_has_errors must close the Diags owner after observation');
assert.match(code.diags, /match\s+level:[\s\S]*DiagLevel::Error:[\s\S]*DiagLevel::Log:[\s\S]*DiagLevel::Info:[\s\S]*DiagLevel::Warn:/, 'diags_has_errors_loop must branch by exhaustive DiagLevel match arms');
assert.match(code.diags, /fn\s+diags_one\s+<\(Diag\)\*>Diags>\s+\(d\):[\s\S]*match\s+v::push\s+items0\s+d:[\s\S]*Result::Err\s+e:[\s\S]*v::free\s+v::vec_push_error_vec\s+e[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_one must close the recovered Vec owner before converting push failure to an empty Diags sentinel');
assert.match(code.diags, /fn\s+diags_push\s+<\(Diags,Diag\)\*>Diags>\s+\(ds,\s*d\):[\s\S]*let\s+items\s+<Vec<Diag>>\s+field::get\s+ds\s+"items"[\s\S]*match\s+v::push\s+items\s+d:[\s\S]*Result::Err\s+e:[\s\S]*v::free\s+v::vec_push_error_vec\s+e[\s\S]*Diags\s+diag_empty_diag_vec/, 'diags_push must close the recovered Vec owner before converting grow failure to an empty Diags sentinel');

assert.match(code.outcome, /struct\s+Outcome<\.T,\s*\.E>:[\s\S]*result\s+<Result<\.T,\s*\.E>>[\s\S]*diags\s+<Option<Diags>>/, 'Outcome must keep result and Diags as separate axes');
assert.match(code.outcome, /fn\s+outcome_with_diags[\s\S]*Option::Some\s+old_ds:[\s\S]*diags_free\s+old_ds/, 'outcome_with_diags must close any replaced Diags owner');
assert.match(code.outcome, /fn\s+outcome_result[\s\S]*field::get\s+o\s+"diags":[\s\S]*Option::Some\s+ds:[\s\S]*diags_free\s+ds/, 'by-value outcome_result must close the Diags axis before returning Result');
assert.doesNotMatch(code.outcome, /fn\s+diag_level_str/, 'DiagLevel stringification belongs to the enum/types module');

console.log('stdlib diag error module boundary regression passed');

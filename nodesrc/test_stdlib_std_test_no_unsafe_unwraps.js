#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/test.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'std/test must qualify implementation Vec allocation calls to avoid caller import ambiguity');
assert.match(code, /fn\s+checks_empty_vec\s+<\(\)->Vec<Result<\(\),str>>>\s+\(\):\s+v::Vec<Result<\(\),str>>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'checks allocation fallback must use an empty Vec sentinel');
assert.match(code, /fn\s+checks_single_error\s+<\(str\)\*>Vec<Result<\(\),str>>>\s+\(msg\):[\s\S]*match\s+v::new<Result<\(\),str>>:[\s\S]*match\s+v::push<Result<\(\),str>>\s+items0\s+marker:[\s\S]*Result::Err\s+_e:[\s\S]*checks_empty_vec/, 'checks_push failure marker helper must avoid unsafe unwraps');
assert.match(code, /fn\s+checks_new\s+<\(\)\*>Vec<Result<\(\),str>>>\s+\(\):[\s\S]*match\s+v::new<Result<\(\),str>>:[\s\S]*Result::Err\s+_e:[\s\S]*checks_empty_vec/, 'checks_new must handle allocation failure without trapping');
assert.match(code, /fn\s+checks_push\s+<\(Vec<Result<\(\),str>>,Result<\(\),str>\)\*>Vec<Result<\(\),str>>>\s+\(checks,\s*r\):[\s\S]*match\s+v::push<Result<\(\),str>>\s+checks\s+r:[\s\S]*Result::Err\s+_e:[\s\S]*checks_single_error\s+"std\/test checks_push allocation failed"/, 'checks_push must convert grow failure to a test failure marker');

console.log('stdlib std/test unsafe unwrap regression passed');

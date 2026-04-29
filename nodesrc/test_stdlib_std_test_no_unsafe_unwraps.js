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

const obsoleteVecAccumulator = [
    /#import\s+"alloc\/collections\/vec"\s+as\s+v/,
    /\bVec<Result<\(\),str>>/,
    /\bchecks_empty_vec\b/,
    /\bchecks_has_err_loop\b/,
    /\bchecks_summary_loop\b/,
    /\bchecks_print_human_loop\b/,
    /\bload<Result<\(\),str>>/,
];

for (const pattern of obsoleteVecAccumulator) {
    assert.doesNotMatch(code, pattern, 'std/test must not reintroduce the raw Vec<Result<(),str>> accumulator');
}

assert.match(code, /struct\s+Checks:[\s\S]*count\s+<i32>[\s\S]*failed\s+<bool>[\s\S]*summary\s+<str>[\s\S]*human\s+<str>/, 'std/test must keep the value-based Checks accumulator fields');
assert.match(code, /impl\s+Copy\s+for\s+Checks:[\s\S]*fn\s+copy_mark\s+<\(Checks\)->Checks>\s+\(checks\):[\s\S]*checks/, 'Checks must remain Copy so aggregation does not require raw backing storage');
assert.match(code, /fn\s+checks_empty\s+<\(\)->Checks>\s+\(\):\s+Checks\s+0\s+false\s+"\["\s+""/, 'checks_empty must create the allocation-free accumulator sentinel');
assert.match(code, /fn\s+checks_single_error\s+<\(str\)\*>Checks>\s+\(msg\):\s+checks_push\s+checks_empty\s+Result<\(\),str>::Err\s+msg/, 'checks_single_error must build a single failure without Vec allocation');
assert.match(code, /fn\s+checks_push\s+<\(Checks,Result<\(\),str>\)\*>Checks>\s+\(checks,\s*r\):[\s\S]*let\s+count\s+<i32>\s+checks\.count[\s\S]*let\s+failed1\s+<bool>\s+match\s+r:[\s\S]*Result::Err\s+e:[\s\S]*true[\s\S]*Checks\s+add\s+count\s+1\s+failed1\s+summary1\s+human1/, 'checks_push must update the value accumulator directly');
assert.match(code, /fn\s+finish_checks\s+<\(Checks\)\*>Result<\(\),str>>\s+\(checks\):[\s\S]*let\s+summary\s+<str>\s+checks_summary_text\s+checks[\s\S]*let\s+failed\s+<bool>\s+checks\.failed[\s\S]*Result<\(\),str>::Err\s+summary[\s\S]*Result<\(\),str>::Ok\s+\(\)/, 'finish_checks must decide from the accumulated value state');

console.log('stdlib std/test unsafe unwrap regression passed');

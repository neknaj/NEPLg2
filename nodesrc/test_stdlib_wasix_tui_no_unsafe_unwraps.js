#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/platforms/wasix/tui.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'wasix tui must qualify implementation Vec allocation calls');
assert.match(code, /fn\s+tui_empty_str_vec\s+<\(\)->Vec<str>>\s+\(\):\s+v::Vec<str>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'text_wrap_lines allocation fallback must use an empty Vec sentinel');
assert.match(code, /fn\s+tui_push_str\s+<\(Vec<str>,str\)->TuiStrPushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<str>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*TuiStrPushRes\s+tui_empty_str_vec\s+false/, 'text_wrap_lines push must convert grow failure to ok=false');
assert.match(code, /fn\s+text_wrap_lines\s+<\(str,i32\)\*>Vec<str>>\s+\(text,\s*cols\):[\s\S]*match\s+v::new<str>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'text_wrap_lines must handle Vec allocation failure');
assert.match(code, /while\s+and\s+lt\s+i\s+n\s+not\s+failed:/, 'text_wrap_lines must stop scanning after line accumulation failure');
assert.match(code, /let\s+pushed_tail\s+<TuiStrPushRes>\s+tui_push_str\s+out\s+tail/, 'text_wrap_lines tail accumulation must go through checked push');

console.log('stdlib wasix tui unsafe unwrap regression passed');

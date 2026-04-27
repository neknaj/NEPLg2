#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/kp/kpgraph.nepl';
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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'kpgraph must qualify implementation Vec allocation calls');
assert.match(code, /fn\s+kp_i32_empty_vec\s+<\(\)->Vec<i32>>\s+\(\):\s+v::Vec<i32>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'BFS result allocation fallback must use an empty Vec sentinel');
assert.match(code, /fn\s+kp_push_i32\s+<\(Vec<i32>,i32\)->KpI32PushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<i32>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*KpI32PushRes\s+kp_i32_empty_vec\s+false/, 'BFS result push must convert grow failure to ok=false');
assert.match(code, /fn\s+dense_graph_bfs_dist_raw\s+<\(i32,i32,i32\)\*>Vec<i32>>\s+\(n,\s*mat,\s*start\):[\s\S]*match\s+v::new<i32>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'BFS result Vec creation must handle allocation failure');
assert.match(code, /while\s+and\s+lt\s+i1\s+n\s+not\s+failed:[\s\S]*kp_push_i32\s+out\s+load_i32\s+add\s+dist\s+mul\s+i1\s+4/, 'BFS result accumulation must stop after push failure');

console.log('stdlib kpgraph unsafe unwrap regression passed');

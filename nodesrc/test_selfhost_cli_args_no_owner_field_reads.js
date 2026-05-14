#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/neplg2/cli/args/parse.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.doesNotMatch(
    code,
    /\b(?:VecDataLen|data_len<|v::data_len)\b/,
    'self-host CLI args parser must not recreate a raw VecDataLen storage carrier',
);

assert.doesNotMatch(
    code,
    /#import\s+"core\/mem(?:\/(?:internal|raw))?"\s+as\b/,
    'self-host CLI args parser must not import raw memory modules for argv observation',
);

assert.doesNotMatch(
    code,
    /\b(?:mem_ptr_addr|data_mem_ptr|load<str>|size_of<str>)\b/,
    'self-host CLI args parser must not read Vec<str> through raw storage',
);

assert.match(
    code,
    /fn\s+selfhost_cli_arg_at\s+<\(&Vec<str>,i32\)->Option<str>>[\s\S]*v::get<str>\s+args\s+idx/,
    'selfhost_cli_arg_at must use Vec.get for checked borrowed argv reads',
);

assert.match(
    code,
    /selfhost_cli_parse_args[\s\S]*let\s+count\s+<i32>\s+v::len<str>\s+args[\s\S]*selfhost_cli_parse_loop\s+args\s+count\s+0/,
    'selfhost_cli_parse_args must pass the borrowed Vec and count without extracting raw storage',
);

assert.match(
    code,
    /selfhost_cli_parse_argv[\s\S]*let\s+count\s+<i32>\s+v::len<str>\s+argv[\s\S]*selfhost_cli_parse_loop\s+argv\s+count\s+1/,
    'selfhost_cli_parse_argv must pass the borrowed Vec and count without extracting raw storage',
);

console.log('selfhost CLI args owner field read regression passed');

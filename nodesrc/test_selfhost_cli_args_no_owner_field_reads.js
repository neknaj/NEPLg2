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

assert.match(
    code,
    /selfhost_cli_parse_args[\s\S]*mem_ptr_addr\s+v::data_mem_ptr<str>\s+args[\s\S]*v::len<str>\s+args/,
    'selfhost_cli_parse_args must read Vec data pointer and length through separate borrowed observers',
);

assert.match(
    code,
    /selfhost_cli_parse_argv[\s\S]*mem_ptr_addr\s+v::data_mem_ptr<str>\s+argv[\s\S]*v::len<str>\s+argv/,
    'selfhost_cli_parse_argv must read Vec data pointer and length through separate borrowed observers',
);

console.log('selfhost CLI args owner field read regression passed');

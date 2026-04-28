#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/neplg2/cli/args.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.doesNotMatch(
    code,
    /\bget\s+span\s+"(?:data|len)"/,
    'self-host CLI args parser must read VecDataLen span fields through get_ref so the span owner is not moved',
);

assert.match(
    code,
    /selfhost_cli_parse_args[\s\S]*mem_ptr_addr\s+\*get_ref\s+&span\s+"data"[\s\S]*\*get_ref\s+&span\s+"len"/,
    'selfhost_cli_parse_args must project VecDataLen data/len by reference',
);

assert.match(
    code,
    /selfhost_cli_parse_argv[\s\S]*mem_ptr_addr\s+\*get_ref\s+&span\s+"data"[\s\S]*\*get_ref\s+&span\s+"len"/,
    'selfhost_cli_parse_argv must project VecDataLen data/len by reference',
);

console.log('selfhost CLI args owner field read regression passed');

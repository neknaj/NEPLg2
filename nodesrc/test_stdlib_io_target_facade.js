#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const ioRelPath = 'stdlib/std/io.nepl';
const targetRelPath = 'stdlib/std/iotarget.nepl';
const ioSrc = fs.readFileSync(path.join(repoRoot, ioRelPath), 'utf8');
const targetSrc = fs.readFileSync(path.join(repoRoot, targetRelPath), 'utf8');

const codeOnly = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const ioCode = codeOnly(ioSrc);
const targetCode = codeOnly(targetSrc);

assert.match(
    ioCode,
    /pub\s+#import\s+"std\/iotarget"\s+as\s+\*/,
    'std/io root must re-export the target enum surface used by read/write signatures',
);

assert.match(
    targetCode,
    /\bpub\s+enum\s+ReadStream:[\s\S]*?\bStdio\b[\s\S]*?\bFs\s+<str>[\s\S]*?\bText\s+<str>[\s\S]*?\bBytes\s+<ByteBuf>/,
    'std/iotarget must own the ReadStream enum variants',
);

assert.match(
    targetCode,
    /\bpub\s+enum\s+WriteStream:[\s\S]*?\bStdio\b[\s\S]*?\bStderr\b[\s\S]*?\bFs\b/,
    'std/iotarget must own the WriteStream enum variants',
);

assert.doesNotMatch(
    ioCode,
    /\bpub\s+enum\s+(?:ReadStream|WriteStream)\b/,
    'std/io must not duplicate target enum definitions',
);

assert.doesNotMatch(
    targetCode,
    /\b(?:fn|trait|impl)\s+/,
    'std/iotarget must remain a target vocabulary module, not an execution facade',
);

assert.doesNotMatch(
    targetCode,
    /\b(?:core\/mem\/raw|alloc_ptr|dealloc_ptr|mem_ptr_addr|load_u8|store_u8|mem_copy)\b/,
    'std/iotarget must not carry raw memory or I/O implementation authority',
);

console.log('std/io target facade regression passed');

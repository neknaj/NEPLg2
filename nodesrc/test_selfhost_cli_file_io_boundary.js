#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const fileIoRel = 'stdlib/neplg2/cli/file_io.nepl';
const driverRel = 'stdlib/neplg2/cli/driver.nepl';
const fileIoSrc = fs.readFileSync(path.join(repoRoot, fileIoRel), 'utf8');
const driverSrc = fs.readFileSync(path.join(repoRoot, driverRel), 'utf8');

assert.match(
    fileIoSrc,
    /#import\s+"std\/fs"\s+as\s+\*/,
    'cli/file_io.nepl must be the CLI filesystem bridge',
);

assert.match(
    fileIoSrc,
    /#import\s+"neplg2\/core\/module\/loader"\s+as\s+\*/,
    'file_io must build the core VFS model rather than parsing files directly',
);

for (const symbol of [
    'fs_read_to_string_checked',
    'fs_write_to_string',
    'fs_write_to_bytes',
    'selfhost_vfs_add',
]) {
    assert.match(fileIoSrc, new RegExp(`\\b${symbol}\\b`), `file_io must use ${symbol}`);
}

assert.doesNotMatch(
    fileIoSrc,
    /#import\s+"std\/(?:stdio|streamio|io)"/,
    'file_io must not write stdout/stderr directly; reporter owns stdio',
);

assert.doesNotMatch(
    driverSrc,
    /#import\s+"std\/fs"/,
    'driver must remain VFS-facing and must not import std/fs directly',
);

function collectNeplFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...collectNeplFiles(full));
        } else if (entry.isFile() && entry.name.endsWith('.nepl')) {
            files.push(full);
        }
    }
    return files;
}

const neplg2Root = path.join(repoRoot, 'stdlib/neplg2');
for (const fullPath of collectNeplFiles(neplg2Root)) {
    const rel = path.relative(repoRoot, fullPath).replaceAll(path.sep, '/');
    if (rel === fileIoRel) {
        continue;
    }
    const src = fs.readFileSync(fullPath, 'utf8');
    assert.doesNotMatch(
        src,
        /#import\s+"std\/fs"/,
        `std/fs imports must stay confined to ${fileIoRel}; found in ${rel}`,
    );
}

console.log('selfhost CLI file_io boundary regression passed');

#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/hash/sha256/api.nepl',
    'stdlib/alloc/hash/sha256/compress.nepl',
    'stdlib/alloc/hash/sha256/digest.nepl',
    'stdlib/alloc/hash/sha256/padding.nepl',
    'stdlib/alloc/hash/sha256/schedule.nepl',
];

const violations = [];

for (const relPath of relPaths) {
    const filePath = path.join(repoRoot, relPath);
    const text = fs.readFileSync(filePath, 'utf8');
    const lines = text.split(/\r?\n/);
    lines.forEach((line, index) => {
        if (/\b(?:new|with_capacity|push|get|len|free)<i32>/.test(line)) {
            violations.push(`${relPath}:${index + 1}: sha256 Vec i32 helper postfix: ${line.trim()}`);
        }
        if (/Vec<i32>/.test(line)) {
            violations.push(`${relPath}:${index + 1}: old sha256 Vec i32 prose type notation: ${line.trim()}`);
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 sha256 modules must not reintroduce selected Vec i32 generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 sha256 postfix cleanup regression passed');

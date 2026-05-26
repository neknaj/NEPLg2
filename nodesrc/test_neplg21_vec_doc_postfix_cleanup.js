#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/vec.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const forbidden = [
    ['old Vec generic prose type notation', /Vec<\.T>/],
    ['Vec doctest helper postfix', /\b(?:new|with_capacity|push|get|len|free|clear)<i32>/],
];

const violations = [];
const lines = src.split(/\r?\n/);
lines.forEach((line, index) => {
    for (const [label, pattern] of forbidden) {
        if (pattern.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${label}: ${line.trim()}`);
        }
    }
});

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 Vec facade docs must not reintroduce selected generic postfixes:\n${violations.join('\n')}`,
);

console.log('NEPLg2.1 Vec facade doc postfix cleanup regression passed');

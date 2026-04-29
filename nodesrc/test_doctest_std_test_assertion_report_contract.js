#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const roots = ['tests', 'tutorials', 'stdlib', 'examples'];

function walk(dir, out = []) {
    if (!fs.existsSync(dir)) {
        return out;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walk(full, out);
        } else if (entry.name.endsWith('.n.md') || entry.name.endsWith('.nepl')) {
            out.push(full);
        }
    }
    return out;
}

function stripLineComment(line) {
    const idx = line.indexOf('//');
    return idx >= 0 ? line.slice(0, idx) : line;
}

function importsStdTest(code) {
    return /#import\s+"std\/test"\s+as\b/.test(code);
}

function isDiscardedStdTestAssertion(line) {
    const trimmed = stripLineComment(line).trim();
    if (!trimmed.endsWith(';')) {
        return false;
    }
    const expr = trimmed.slice(0, -1).trim();
    return /^(assert[A-Za-z0-9_]*|check[A-Za-z0-9_]*)\b/.test(expr);
}

function findDiscardedAssertions(code) {
    if (!importsStdTest(code)) {
        return [];
    }
    return code
        .split(/\r?\n/)
        .map((line, index) => ({ line, lineNumber: index + 1 }))
        .filter(({ line }) => isDiscardedStdTestAssertion(line));
}

const goodCases = [
    `#import "std/test" as *
fn helper <(i32,i32)->Result<(),str>> (a, b):
    assert_eq_i32 a b
`,
    `#import "std/test" as *
fn main <()*>i32> ():
    let checks checks_new
    let shown checks_print_report checks_push checks assert_eq_i32 1 1
    checks_exit_code shown
`,
];

const badCase = `#import "std/test" as *
fn main <()*>i32> ():
    assert_eq_i32 1 1;
    0
`;

for (const sample of goodCases) {
    assert.deepEqual(findDiscardedAssertions(sample), []);
}
assert.deepEqual(findDiscardedAssertions(badCase).map((hit) => hit.lineNumber), [3]);

const violations = [];
for (const root of roots) {
    for (const file of walk(path.join(repoRoot, root))) {
        const parsed = parseFile(file);
        parsed.doctests.forEach((doctest, index) => {
            for (const hit of findDiscardedAssertions(doctest.code)) {
                violations.push(
                    `${path.relative(repoRoot, file)}::doctest#${index + 1}:` +
                    `${hit.lineNumber}: ${hit.line.trim()}`,
                );
            }
        });
    }
}

assert.deepEqual(
    violations,
    [],
    `std/test assertions must be aggregated into a report instead of discarded:\n${violations.join('\n')}`,
);

console.log('doctest std/test assertion report contract passed');

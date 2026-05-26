#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const scanRoots = ['stdlib', 'tests', 'examples', 'tutorials'];
const sourceExtensions = ['.nepl', '.n.md'];
const helperPostfixPattern = /\b(?:uwok|uwerr|unwrap_err|unwrap_or|diag_err|some|none|ok|err|is_some|is_none|is_ok|is_err)</g;
const enumConstructorPostfixPattern = /\b(?:Option::Some|Option::None|Result::Ok|Result::Err)</g;

const violations = [];

for (const root of scanRoots) {
    const rootPath = path.join(repoRoot, root);
    if (!fs.existsSync(rootPath)) {
        continue;
    }
    for (const filePath of walkFiles(rootPath)) {
        if (!sourceExtensions.some((extension) => filePath.endsWith(extension))) {
            continue;
        }
        const relPath = path.relative(repoRoot, filePath).replace(/\\/g, '/');
        const text = fs.readFileSync(filePath, 'utf8');
        const lines = text.split(/\r?\n/);
        lines.forEach((line, index) => {
            if (helperPostfixPattern.test(line)) {
                violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
            }
            if (enumConstructorPostfixPattern.test(line)) {
                violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
            }
            helperPostfixPattern.lastIndex = 0;
            enumConstructorPostfixPattern.lastIndex = 0;
        });
    }
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 executable corpus must not reintroduce helper or enum constructor generic postfixes:\n${violations.join('\n')}`,
);

function* walkFiles(dir) {
    const entries = fs.readdirSync(dir, { withFileTypes: true })
        .sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            yield* walkFiles(fullPath);
        } else if (entry.isFile()) {
            yield fullPath;
        }
    }
}

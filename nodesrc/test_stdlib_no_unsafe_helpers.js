#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const stdlibRoot = path.join(repoRoot, 'stdlib');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

const allowances = [
    {
        id: 'core-option-unsafe-helper-definition',
        relPath: 'stdlib/core/option.nepl',
        line: /\bfn\s+unwrap\b|#intrinsic\s+"unreachable"/,
    },
    {
        id: 'core-result-unsafe-helper-definitions',
        relPath: 'stdlib/core/result.nepl',
        line: /\bfn\s+(unwrap_ok|unwrap_err|uwok|uwerr)\b|#intrinsic\s+"unreachable"/,
    },
    {
        id: 'core-test-intentional-unreachable',
        relPath: 'stdlib/core/test.nepl',
        line: /#intrinsic\s+"unreachable"/,
    },
];

function relFromRoot(filePath) {
    return path.relative(repoRoot, filePath).replace(/\\/g, '/');
}

function collectNeplFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            collectNeplFiles(fullPath, out);
        } else if (entry.isFile() && entry.name.endsWith('.nepl')) {
            out.push(fullPath);
        }
    }
    return out;
}

function matchingAllowance(relPath, line) {
    return allowances.find((allowance) => allowance.relPath === relPath && allowance.line.test(line));
}

const unexpected = [];
const seenAllowances = new Set();

for (const filePath of collectNeplFiles(stdlibRoot).sort()) {
    const relPath = relFromRoot(filePath);
    const lines = fs.readFileSync(filePath, 'utf8').split(/\r?\n/);
    for (let i = 0; i < lines.length; i += 1) {
        const rawLine = lines[i];
        const line = rawLine.trim();
        if (line.startsWith('//')) continue;
        if (!forbidden.some((pattern) => pattern.test(line))) continue;

        const allowance = matchingAllowance(relPath, line);
        if (allowance) {
            seenAllowances.add(allowance.id);
            continue;
        }
        unexpected.push(`${relPath}:${i + 1}: ${line}`);
    }
}

const staleAllowances = allowances
    .filter((allowance) => !seenAllowances.has(allowance.id))
    .map((allowance) => `${allowance.id}: ${allowance.relPath}`);

assert.deepEqual(unexpected, [], `unexpected unsafe helpers in stdlib implementation code:\n${unexpected.join('\n')}`);
assert.deepEqual(staleAllowances, [], `stale unsafe-helper allowlist entries:\n${staleAllowances.join('\n')}`);

console.log('stdlib unsafe helper source policy passed');

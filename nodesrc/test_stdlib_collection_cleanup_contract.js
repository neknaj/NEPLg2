#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const collectionsRoot = path.join(repoRoot, 'stdlib/alloc/collections');

const inspected = [];
const violations = [];

for (const relPath of walkNeplFiles(collectionsRoot)) {
    const source = readImplementation(relPath);
    const lines = source.split(/\r?\n/);
    for (let index = 0; index < lines.length; index += 1) {
        const signature = parseFunctionSignature(lines[index]);
        if (!signature || !isCleanupFunction(signature.name)) {
            continue;
        }

        const generics = parseGenericParameters(signature.afterName);
        if (generics.length === 0) {
            continue;
        }

        inspected.push(`${relPath}:${signature.name}`);
        const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
        if (missingCopy.length > 0) {
            const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
            violations.push(`${relPath}:${index + 1}: ${signature.name} cleanup generic(s) ${names} must carry Copy until collection drop traversal exists`);
        }
    }
}

assert.ok(
    inspected.length >= 15,
    `collection cleanup policy must inspect the current generic cleanup surface, inspected only ${inspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:clear',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:free',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl:vec_free_storage',
    'stdlib/alloc/collections/hashmap/api.nepl:free',
    'stdlib/alloc/collections/hashset/api.nepl:free',
    'stdlib/alloc/collections/queue/api.nepl:clear',
    'stdlib/alloc/collections/queue/api.nepl:free',
    'stdlib/alloc/collections/deque/api.nepl:clear',
    'stdlib/alloc/collections/deque/api.nepl:free',
]) {
    assert.ok(
        inspected.some((entry) => entry.includes(expected)),
        `collection cleanup policy did not inspect expected cleanup signature: ${expected}`,
    );
}

assert.deepEqual(violations, [], `generic collection cleanup APIs must remain Copy-only:\n${violations.join('\n')}`);

console.log('stdlib collection cleanup contract regression passed');

function walkNeplFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walkNeplFiles(fullPath, out);
        } else if (entry.isFile() && entry.name.endsWith('.nepl')) {
            out.push(path.relative(repoRoot, fullPath).replace(/\\/g, '/'));
        }
    }
    return out.sort();
}

function readImplementation(relPath) {
    return fs
        .readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function parseFunctionSignature(line) {
    const match = line.match(/^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(.*)$/);
    if (!match) {
        return null;
    }
    return { name: match[1], afterName: match[2].trimStart() };
}

function isCleanupFunction(name) {
    return /(?:^|_)(?:clear|free)(?:_|$)/.test(name) || /\bcleanup\b/.test(name);
}

function parseGenericParameters(afterName) {
    if (!afterName.startsWith('<') || afterName.startsWith('<(')) {
        return [];
    }

    let depth = 0;
    let end = -1;
    for (let i = 0; i < afterName.length; i += 1) {
        const ch = afterName[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>') {
            depth -= 1;
            if (depth === 0) {
                end = i;
                break;
            }
        }
    }

    if (end === -1) {
        return [];
    }

    const genericText = afterName.slice(1, end);
    const typeSignature = afterName.slice(end + 1).trimStart();
    if (!typeSignature.startsWith('<')) {
        return [];
    }

    return splitTopLevel(genericText, ',')
        .map((entry) => {
            const match = entry.trim().match(/^\.(\w+)(?:\s*:\s*(.+))?$/);
            if (!match) {
                return null;
            }
            return { name: match[1], bound: match[2]?.trim() };
        })
        .filter((entry) => entry !== null);
}

function splitTopLevel(text, delimiter) {
    const parts = [];
    let depth = 0;
    let start = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>') {
            depth -= 1;
        } else if (ch === delimiter && depth === 0) {
            parts.push(text.slice(start, i));
            start = i + 1;
        }
    }
    parts.push(text.slice(start));
    return parts;
}

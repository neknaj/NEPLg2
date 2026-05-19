#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const collectionsRoot = path.join(repoRoot, 'stdlib/alloc/collections');

const inspected = [];
const ownerAccessorInspected = [];
const popOwnerAccessorInspected = [];
const fallibleOwnerConsumerInspected = [];
const violations = [];

for (const relPath of walkNeplFiles(collectionsRoot)) {
    const source = readImplementation(relPath);
    const lines = source.split(/\r?\n/);
    for (let index = 0; index < lines.length; index += 1) {
        const signature = parseFunctionSignature(lines[index]);
        if (!signature || !isCleanupFunction(signature.name)) {
            const generics = parseGenericParameters(signature?.afterName ?? '');
            const typeSignature = parseTypeSignature(signature?.afterName ?? '');
            if (signature && generics.length > 0 && isOwnerReturningErrorAccessor(signature.name, typeSignature)) {
                ownerAccessorInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} owner-returning error accessor generic(s) ${names} must carry Copy until collection drop traversal exists`);
                }
            }
            if (signature && generics.length > 0 && isOwnerReturningPopAccessor(signature.name, typeSignature)) {
                popOwnerAccessorInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} pop-result owner accessor generic(s) ${names} must carry Copy until collection drop traversal exists`);
                }
            }
            const fallibleOwnerConsumer = signature && generics.length > 0
                ? classifyFallibleOwnerConsumer(typeSignature)
                : null;
            if (fallibleOwnerConsumer) {
                fallibleOwnerConsumerInspected.push(`${relPath}:${signature.name}`);
                if (fallibleOwnerConsumer.errorKind === 'bare') {
                    violations.push(`${relPath}:${index + 1}: ${signature.name} consumes a generic collection owner but returns bare Diag/StdErrorKind on failure; use an owner-bearing error payload`);
                }
            }
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

assert.ok(
    ownerAccessorInspected.length >= 14,
    `collection owner accessor policy must inspect the current generic error owner surface, inspected only ${ownerAccessorInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_error_vec',
    'stdlib/alloc/collections/vec/mutation/push.nepl:vec_realloc_region_error_region',
    'stdlib/alloc/collections/vec/types.nepl:vec_transform_error_vec',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl:vec_sort_merge_error_vec',
    'stdlib/alloc/collections/stack/types.nepl:stack_push_error_stack',
    'stdlib/alloc/collections/queue/types.nepl:queue_push_error_queue',
    'stdlib/alloc/collections/deque/types.nepl:deque_push_error_deque',
    'stdlib/alloc/collections/ringbuffer/types.nepl:ringbuffer_push_error_buffer',
    'stdlib/alloc/collections/binary_heap/types.nepl:binary_heap_push_error_heap',
    'stdlib/alloc/collections/list/types.nepl:list_push_error_list',
    'stdlib/alloc/collections/list/types.nepl:list_transform_error_list',
    'stdlib/alloc/collections/btreemap/types.nepl:btreemap_insert_error_owner',
    'stdlib/alloc/collections/btreeset/types.nepl:btreeset_insert_error_owner',
    'stdlib/alloc/collections/hashmap/types.nepl:hashmap_update_error_owner',
    'stdlib/alloc/collections/hashset/types.nepl:hashset_update_error_owner',
]) {
    assert.ok(
        ownerAccessorInspected.some((entry) => entry.includes(expected)),
        `collection owner accessor policy did not inspect expected error owner accessor: ${expected}`,
    );
}

assert.ok(
    popOwnerAccessorInspected.length >= 6,
    `collection pop owner accessor policy must inspect the current generic pop owner surface, inspected only ${popOwnerAccessorInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/types.nepl:vec_pop_vec',
    'stdlib/alloc/collections/stack/api.nepl:stack_pop_stack',
    'stdlib/alloc/collections/queue/types.nepl:queue_pop_queue',
    'stdlib/alloc/collections/deque/api.nepl:deque_pop_deque',
    'stdlib/alloc/collections/ringbuffer/types.nepl:ringbuffer_pop_buffer',
    'stdlib/alloc/collections/binary_heap/api/pop.nepl:binary_heap_pop_heap',
]) {
    assert.ok(
        popOwnerAccessorInspected.some((entry) => entry.includes(expected)),
        `collection pop owner accessor policy did not inspect expected pop owner accessor: ${expected}`,
    );
}

for (const expected of [
    'stdlib/alloc/collections/list/transform.nepl:map',
    'stdlib/alloc/collections/list/transform.nepl:filter',
]) {
    assert.ok(
        fallibleOwnerConsumerInspected.some((entry) => entry.includes(expected)),
        `collection fallible owner consumer policy did not inspect expected owner-consuming Result API: ${expected}`,
    );
}

assert.deepEqual(violations, [], `generic collection cleanup and owner recovery APIs must remain Copy-only:\n${violations.join('\n')}`);

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

function isOwnerReturningErrorAccessor(name, typeSignature) {
    if (!typeSignature || !/(?:^|_)error_/.test(name)) {
        return false;
    }

    const functionType = parseUnaryFunctionType(typeSignature);
    if (!functionType) {
        return false;
    }

    return !functionType.parameter.trimStart().startsWith('&')
        && /\b[A-Za-z_][A-Za-z0-9_]*Error</.test(functionType.parameter)
        && /<\./.test(functionType.returnType);
}

function isOwnerReturningPopAccessor(name, typeSignature) {
    if (!typeSignature) {
        return false;
    }

    const functionType = parseUnaryFunctionType(typeSignature);
    if (!functionType) {
        return false;
    }

    return !functionType.parameter.trimStart().startsWith('&')
        && /\b[A-Za-z_][A-Za-z0-9_]*Pop</.test(functionType.parameter)
        && /<\./.test(functionType.returnType);
}

function classifyFallibleOwnerConsumer(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length === 0) {
        return null;
    }

    const firstParameter = functionType.parameters[0].trim();
    if (firstParameter.startsWith('&') || !/\b[A-Z][A-Za-z0-9_]*<\./.test(firstParameter)) {
        return null;
    }

    const errorType = resultErrorType(functionType.returnType);
    if (!errorType) {
        return null;
    }

    return {
        errorKind: /^(?:Diag|StdErrorKind)$/.test(errorType.trim()) ? 'bare' : 'owner-bearing',
    };
}

function parseGenericParameters(afterName) {
    if (!afterName.startsWith('<') || afterName.startsWith('<(')) {
        return [];
    }

    const end = topLevelAngleEnd(afterName);
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

function parseTypeSignature(afterName) {
    let rest = afterName.trimStart();
    if (!rest.startsWith('<')) {
        return null;
    }
    if (!rest.startsWith('<(')) {
        const genericEnd = topLevelAngleEnd(rest);
        if (genericEnd === -1) {
            return null;
        }
        rest = rest.slice(genericEnd + 1).trimStart();
    }
    if (!rest.startsWith('<')) {
        return null;
    }
    const typeEnd = topLevelAngleEnd(rest);
    if (typeEnd === -1) {
        return null;
    }
    return rest.slice(0, typeEnd + 1);
}

function parseUnaryFunctionType(typeSignature) {
    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 1) {
        return null;
    }
    return { parameter: functionType.parameters[0], returnType: functionType.returnType };
}

function parseFunctionType(typeSignature) {
    if (!typeSignature.startsWith('<(') || !typeSignature.endsWith('>')) {
        return null;
    }

    const body = typeSignature.slice(1, -1);
    let angleDepth = 0;
    let parenDepth = 0;
    for (let i = 0; i < body.length; i += 1) {
        const ch = body[i];
        if (ch === '<') {
            angleDepth += 1;
        } else if (ch === '>' && body[i - 1] !== '-' && body[i - 1] !== '*') {
            angleDepth -= 1;
        } else if (ch === '(' && angleDepth === 0) {
            parenDepth += 1;
        } else if (ch === ')' && angleDepth === 0) {
            parenDepth -= 1;
            const separator = body.slice(i + 1, i + 3);
            if (parenDepth === 0 && (separator === '->' || separator === '*>')) {
                const parameterText = body.slice(1, i).trim();
                const returnType = body.slice(i + 3).trim();
                return { parameters: splitTopLevelTuple(parameterText, ','), returnType };
            }
        }
    }
    return null;
}

function topLevelAngleEnd(text) {
    let depth = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            depth -= 1;
            if (depth === 0) {
                return i;
            }
        }
    }
    return -1;
}

function splitTopLevel(text, delimiter) {
    const parts = [];
    let depth = 0;
    let start = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            depth -= 1;
        } else if (ch === delimiter && depth === 0) {
            parts.push(text.slice(start, i));
            start = i + 1;
        }
    }
    parts.push(text.slice(start));
    return parts;
}

function splitTopLevelTuple(text, delimiter) {
    if (text === '') {
        return [];
    }

    const parts = [];
    let angleDepth = 0;
    let parenDepth = 0;
    let start = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            angleDepth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            angleDepth -= 1;
        } else if (ch === '(' && angleDepth === 0) {
            parenDepth += 1;
        } else if (ch === ')' && angleDepth === 0) {
            parenDepth -= 1;
        } else if (ch === delimiter && angleDepth === 0 && parenDepth === 0) {
            parts.push(text.slice(start, i).trim());
            start = i + 1;
        }
    }
    parts.push(text.slice(start).trim());
    return parts;
}

function resultErrorType(returnType) {
    const trimmed = returnType.trim();
    if (!trimmed.startsWith('Result<') || !trimmed.endsWith('>')) {
        return null;
    }

    const inner = trimmed.slice('Result<'.length, -1);
    const parts = splitTopLevel(inner, ',');
    if (parts.length !== 2) {
        return null;
    }
    return parts[1].trim();
}

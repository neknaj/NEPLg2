#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const stdlibMapSrc = fs.readFileSync(path.join(repoRoot, 'stdlib/neplg2/core/module/stdlib_map.nepl'), 'utf8');
const loaderSrc = fs.readFileSync(path.join(repoRoot, 'nepl-core/src/loader.rs'), 'utf8');
const selfhostStringQualifiedFiles = [
    'stdlib/neplg2/core/infra/text.nepl',
    'stdlib/neplg2/core/resolve/name_resolver.nepl',
    'stdlib/neplg2/core/syntax/lexer.nepl',
    'stdlib/neplg2/core/module/import_spec.nepl',
    'stdlib/neplg2/core/module/stdlib_map.nepl',
    'stdlib/neplg2/core/module/loader.nepl',
    'stdlib/neplg2/core/module/graph.nepl',
];

for (const moduleName of [
    'utf8',
    'storage',
    'access',
    'builder',
    'search',
    'slice',
    'split',
    'integer',
    'float',
    'concat',
    'builder_ext',
    'find',
]) {
    assert.match(
        rootSrc,
        new RegExp(`pub #import "\\./string/${moduleName}" as \\*`),
        `alloc/string facade must re-export ${moduleName}`,
    );
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'alloc/string facade must not own function bodies');
assert.doesNotMatch(rootCode, /\b(?:struct|enum)\s+/, 'alloc/string facade must not own data definitions');
assert.ok(rootSrc.split(/\r?\n/).length <= 60, `${rootRelPath} should stay as a small facade`);

assert.match(
    stdlibMapSrc,
    /#import\s+"alloc\/string\/concat"\s+as\s+string_concat/,
    'self-host stdlib_map must import concat module directly',
);
assert.doesNotMatch(
    stdlibMapSrc,
    /\bstring::concat(?:_result)?\b/,
    'self-host stdlib_map must not rely on qualified concat through the broad alloc/string facade',
);

for (const modulePath of [
    '&["alloc", "string", "builder", "append.nepl"]',
    '&["alloc", "string", "builder", "build.nepl"]',
    '&["alloc", "string", "builder", "reserve.nepl"]',
    '&["alloc", "string", "builder", "types.nepl"]',
    '&["alloc", "string", "concat.nepl"]',
    '&["alloc", "string", "builder_ext.nepl"]',
    '&["alloc", "string", "integer", "format.nepl"]',
    '&["alloc", "string", "integer.nepl"]',
    '&["alloc", "string", "float", "format.nepl"]',
]) {
    assert.equal(loaderSrc.includes(modulePath), true, `loader raw-memory boundary must include ${modulePath}`);
}

assert.equal(
    loaderSrc.includes('&["alloc", "string", "builder.nepl"]'),
    false,
    'loader raw-memory boundary must not grant capability to the builder facade',
);

for (const relPath of selfhostStringQualifiedFiles) {
    const source = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    const code = stripNeplComments(source);
    assert.doesNotMatch(
        code,
        /#import\s+"alloc\/string"\s+as\s+string\b/,
        `${relPath} must import concrete alloc/string submodules instead of the root facade as string`,
    );
    assert.doesNotMatch(
        code,
        /\bstring::/,
        `${relPath} must not call through the broad alloc/string facade namespace`,
    );
}

console.log('alloc/string facade boundary regression passed');

#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
    stripNeplComments,
    implementationLineCount,
    fnSignaturePattern,
} = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/std/stdio.nepl';
const debugRelPath = 'stdlib/std/stdio/debug.nepl';
const enabledRelPath = 'stdlib/std/stdio/debug/enabled.nepl';
const disabledRelPath = 'stdlib/std/stdio/debug/disabled.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const debugSrc = fs.readFileSync(path.join(repoRoot, debugRelPath), 'utf8');
const enabledSrc = fs.readFileSync(path.join(repoRoot, enabledRelPath), 'utf8');
const disabledSrc = fs.readFileSync(path.join(repoRoot, disabledRelPath), 'utf8');

const rootCode = stripNeplComments(rootSrc);
const debugCode = stripNeplComments(debugSrc);
const enabledCode = stripNeplComments(enabledSrc);
const disabledCode = stripNeplComments(disabledSrc);

assert.match(
    rootCode,
    /pub\s+#import\s+"\.\/stdio\/debug"\s+as\s+\*/,
    'std/stdio facade must re-export stdio debug submodule',
);

for (const helper of [
    'debug',
    'debug_color',
    'debugln',
    'debugln_color',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in stdio/debug`);
    assert.doesNotMatch(debugCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must not stay in stdio/debug facade`);
    assert.match(enabledCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} debug implementation must exist in stdio/debug/enabled`);
    assert.match(disabledCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} release implementation must exist in stdio/debug/disabled`);
}

assert.match(
    debugCode,
    /pub\s+#import\s+"\.\/debug\/enabled"\s+as\s+@merge/,
    'std/stdio/debug facade must re-export debug profile implementation',
);
assert.match(
    debugCode,
    /pub\s+#import\s+"\.\/debug\/disabled"\s+as\s+@merge/,
    'std/stdio/debug facade must re-export release profile implementation',
);

assert.match(
    enabledCode,
    new RegExp(`#if\\[profile=debug\\][\\s\\S]*${fnSignaturePattern('debug', ['str'], 'unit', { effect: 'impure' })}\\s+\\\\s:[\\s\\S]*\\bprint\\s+s\\b`),
    'debug profile debug must delegate to print',
);
assert.match(
    enabledCode,
    new RegExp(fnSignaturePattern('debug_color', ['AnsiColor', 'str'], 'unit', { effect: 'impure' })),
    'debug_color must use typed AnsiColor instead of raw str color',
);
assert.match(
    enabledCode,
    new RegExp(fnSignaturePattern('debugln_color', ['AnsiColor', 'str'], 'unit', { effect: 'impure' })),
    'debugln_color must use typed AnsiColor instead of raw str color',
);
assert.match(
    disabledCode,
    new RegExp(`#if\\[profile=release\\][\\s\\S]*${fnSignaturePattern('debug', ['str'], 'unit', { effect: 'impure' })}\\s+\\\\_s:[\\s\\S]*\\bunit\\b`),
    'release profile debug must stay no-op',
);

assert.ok(implementationLineCount(debugSrc) <= 60, `${debugRelPath} must stay within the facade boundary`);
assert.ok(implementationLineCount(enabledSrc) <= 190, `${enabledRelPath} must stay within the debug profile boundary`);
assert.ok(implementationLineCount(disabledSrc) <= 210, `${disabledRelPath} must stay within the release profile boundary`);

console.log('stdlib stdio debug boundary regression passed');

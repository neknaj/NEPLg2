#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/std/stdio.nepl';
const debugRelPath = 'stdlib/std/stdio/debug.nepl';
const enabledRelPath = 'stdlib/std/stdio/debug/enabled.nepl';
const disabledRelPath = 'stdlib/std/stdio/debug/disabled.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const debugSrc = fs.readFileSync(path.join(repoRoot, debugRelPath), 'utf8');
const enabledSrc = fs.readFileSync(path.join(repoRoot, enabledRelPath), 'utf8');
const disabledSrc = fs.readFileSync(path.join(repoRoot, disabledRelPath), 'utf8');

const stripComments = (src) => src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const rootCode = stripComments(rootSrc);
const debugCode = stripComments(debugSrc);
const enabledCode = stripComments(enabledSrc);
const disabledCode = stripComments(disabledSrc);

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
    /#if\[profile=debug\][\s\S]*fn\s+debug\s+<\(str\)\*>\(\)>\s+\(s\):[\s\S]*\bprint\s+s\b/,
    'debug profile debug must delegate to print',
);
assert.match(
    enabledCode,
    /fn\s+debug_color\s+<\(AnsiColor,str\)\*>\(\)>/,
    'debug_color must use typed AnsiColor instead of raw str color',
);
assert.match(
    enabledCode,
    /fn\s+debugln_color\s+<\(AnsiColor,str\)\*>\(\)>/,
    'debugln_color must use typed AnsiColor instead of raw str color',
);
assert.match(
    disabledCode,
    /#if\[profile=release\][\s\S]*fn\s+debug\s+<\(str\)\*>\(\)>\s+\(_s\):[\s\S]*\(\)/,
    'release profile debug must stay no-op',
);

assert.ok(debugSrc.split(/\r?\n/).length <= 60, `${debugRelPath} must stay within the facade boundary`);
assert.ok(enabledSrc.split(/\r?\n/).length <= 190, `${enabledRelPath} must stay within the debug profile boundary`);
assert.ok(disabledSrc.split(/\r?\n/).length <= 210, `${disabledRelPath} must stay within the release profile boundary`);

console.log('stdlib stdio debug boundary regression passed');

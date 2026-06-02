#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const floatRelPath = 'stdlib/alloc/string/float.nepl';
const formatRelPath = 'stdlib/alloc/string/float/format.nepl';
const parseRelPath = 'stdlib/alloc/string/float/parse.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const floatSrc = fs.readFileSync(path.join(repoRoot, floatRelPath), 'utf8');
const formatSrc = fs.readFileSync(path.join(repoRoot, formatRelPath), 'utf8');
const parseSrc = fs.readFileSync(path.join(repoRoot, parseRelPath), 'utf8');
const rootCode = legacyTypeSyntaxView(rootSrc);
const floatCode = legacyTypeSyntaxView(floatSrc);
const formatCode = legacyTypeSyntaxView(formatSrc);
const parseCode = legacyTypeSyntaxView(parseSrc);

assert.match(rootSrc, /pub #import "\.\/string\/float" as \*/, 'alloc/string facade must re-export string/float');
assert.match(floatSrc, /pub #import "\.\/float\/format" as \*/, 'alloc/string/float facade must re-export float/format');
assert.match(floatSrc, /pub #import "\.\/float\/parse" as \*/, 'alloc/string/float facade must re-export float/parse');

for (const importPath of [
    'core/mem',
    'alloc/string/access',
    'alloc/string/integer/format',
    'alloc/string/storage',
]) {
    assert.match(formatSrc, new RegExp(`#import "${importPath}" as \\*`), `string/float/format must import ${importPath} directly`);
}

for (const importPath of [
    'alloc/string/access',
]) {
    assert.match(parseSrc, new RegExp(`#import "${importPath}" as \\*`), `string/float/parse must import ${importPath} directly`);
}

for (const name of [
    'from_f64_fraction_trim_len',
    'from_f64_write_fraction_digits_result',
    'from_f64_build_fixed_result',
    'from_f64_result',
    'from_f64',
    'from_f32',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(floatCode, new RegExp(`fn\\s+${name}\\b`), `${floatRelPath} facade must not own ${name}`);
    assert.match(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must own ${name}`);
    assert.doesNotMatch(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must not own format API ${name}`);
}

for (const name of [
    'to_f64',
    'to_f32',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.doesNotMatch(floatCode, new RegExp(`fn\\s+${name}\\b`), `${floatRelPath} facade must not own ${name}`);
    assert.doesNotMatch(formatCode, new RegExp(`fn\\s+${name}\\b`), `${formatRelPath} must not own parse API ${name}`);
    assert.match(parseCode, new RegExp(`fn\\s+${name}\\b`), `${parseRelPath} must own ${name}`);
}

assert.match(
    formatCode,
    /fn\s+from_f64_build_fixed_result[\s\S]*string_alloc_region[\s\S]*mem_copy[\s\S]*from_f64_write_fraction_digits_result[\s\S]*string_finish/,
    'f64 formatting must build owned output through one fixed-size string storage allocation',
);
assert.doesNotMatch(
    formatCode,
    /\bstring_builder_with_capacity_result\b|\bsb_append_result\b|\bsb_build_result\b/,
    'f64 formatting must not reintroduce growable StringBuilder owner chains',
);
assert.doesNotMatch(
    formatCode,
    /\b(?:scratch_raw|alloc_ptr<u8>\s+6|string_from_mem_unchecked_result)\b/,
    'string/float must not reintroduce raw scratch formatting',
);
assert.match(
    parseCode,
    /fn\s+to_f64[\s\S]*let\s+mut\s+has_digit\s+<i32>\s+0[\s\S]*eq\s+has_digit\s+0[\s\S]*set\s+parse_ok\s+0/,
    'to_f64 must reject strings without any digit',
);
assert.ok(implementationLineCount(floatSrc) <= 80, `${floatRelPath} should stay within the float facade boundary`);
assert.ok(implementationLineCount(formatSrc) <= 330, `${formatRelPath} should stay within the float format boundary`);
assert.ok(implementationLineCount(parseSrc) <= 300, `${parseRelPath} should stay within the float parse boundary`);
assert.match(formatCode, /\b(?:mem_ptr_addr|store_u8|mem_copy|RegionToken)\b/, 'string/float/format must carry source-level raw memory evidence');
assert.doesNotMatch(floatCode, /\b(?:mem_ptr_addr|store_u8|load_u8|mem_copy|RegionToken)\b/, 'string/float facade must not carry direct raw memory evidence');

console.log('alloc/string float boundary regression passed');

#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { stripNeplComments } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const rootRelPath = 'stdlib/alloc/string.nepl';
const floatRelPath = 'stdlib/alloc/string/float.nepl';
const rootSrc = fs.readFileSync(path.join(repoRoot, rootRelPath), 'utf8');
const floatSrc = fs.readFileSync(path.join(repoRoot, floatRelPath), 'utf8');
const rootCode = stripNeplComments(rootSrc);
const floatCode = stripNeplComments(floatSrc);

assert.match(rootSrc, /pub #import "\.\/string\/float" as \*/, 'alloc/string facade must re-export string/float');

for (const importPath of [
    'core/mem',
    'alloc/string/access',
    'alloc/string/integer',
    'alloc/string/storage',
]) {
    assert.match(floatSrc, new RegExp(`#import "${importPath}" as \\*`), `string/float must import ${importPath} directly`);
}

for (const name of [
    'from_f64_fraction_trim_len',
    'from_f64_write_fraction_digits_result',
    'from_f64_build_fixed_result',
    'from_f64_result',
    'from_f64',
    'to_f64',
    'from_f32',
    'to_f32',
]) {
    assert.doesNotMatch(rootCode, new RegExp(`fn\\s+${name}\\b`), `${rootRelPath} must not own ${name}`);
    assert.match(floatCode, new RegExp(`fn\\s+${name}\\b`), `${floatRelPath} must own ${name}`);
}

assert.match(
    floatCode,
    /fn\s+from_f64_build_fixed_result[\s\S]*string_alloc_region[\s\S]*mem_copy[\s\S]*from_f64_write_fraction_digits_result[\s\S]*string_finish/,
    'f64 formatting must build owned output through one fixed-size string storage allocation',
);
assert.doesNotMatch(
    floatCode,
    /\bstring_builder_with_capacity_result\b|\bsb_append_result\b|\bsb_build_result\b/,
    'f64 formatting must not reintroduce growable StringBuilder owner chains',
);
assert.doesNotMatch(
    floatCode,
    /\b(?:scratch_raw|alloc_ptr<u8>\s+6|string_from_mem_unchecked_result)\b/,
    'string/float must not reintroduce raw scratch formatting',
);
assert.match(
    floatCode,
    /fn\s+to_f64[\s\S]*let\s+mut\s+has_digit\s+<i32>\s+0[\s\S]*eq\s+has_digit\s+0[\s\S]*set\s+ok\s+0/,
    'to_f64 must reject strings without any digit',
);
assert.ok(floatSrc.split(/\r?\n/).length <= 560, `${floatRelPath} should stay within the float conversion boundary`);

console.log('alloc/string float boundary regression passed');

#!/usr/bin/env node
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const typeExpectationPath = path.join(repoRoot, 'nepl-core/src/typecheck/type_expectation.rs');
const prefixCheckPath = path.join(repoRoot, 'nepl-core/src/typecheck/prefix_check.rs');
const callReductionPath = path.join(repoRoot, 'nepl-core/src/typecheck/call_reduction.rs');
const callResolutionPath = path.join(repoRoot, 'nepl-core/src/typecheck/call_resolution.rs');

const typeExpectation = fs.readFileSync(typeExpectationPath, 'utf8');
const prefixCheck = fs.readFileSync(prefixCheckPath, 'utf8');
const callReduction = fs.readFileSync(callReductionPath, 'utf8');
const callResolution = fs.readFileSync(callResolutionPath, 'utf8');

assert.match(typeExpectation, /enum\s+TypeExpectationSource\s*{[\s\S]*ExplicitAscription[\s\S]*BlockResult[\s\S]*OuterConsumerArgument[\s\S]*}/);
assert.match(typeExpectation, /struct\s+TypeExpectation\s*{[\s\S]*target:\s*TypeId[\s\S]*base_depth:\s*usize[\s\S]*span:\s*Span[\s\S]*source:\s*TypeExpectationSource[\s\S]*}/);
assert.match(typeExpectation, /fn\s+call_result_target_after_args\s*\(/);
assert.match(typeExpectation, /TypeExpectationSource::ExplicitAscription\s*=>\s*self\.span/);

for (const [name, source] of [
    ['prefix_check.rs', prefixCheck],
    ['call_reduction.rs', callReduction],
]) {
    assert.doesNotMatch(
        source,
        /Option<\s*\(\s*TypeId\s*,\s*usize\s*\)\s*>/,
        `${name} must not encode type expectations as Option<(TypeId, usize)>`,
    );
}

assert.doesNotMatch(
    prefixCheck,
    /pending_ascription\s*=\s*Some\s*\(\s*\(\s*ty\s*,\s*stack\.len\(\)\s*\)\s*\)/,
    'explicit type annotations must construct TypeExpectation instead of a tuple',
);
assert.match(prefixCheck, /TypeExpectation::explicit_ascription/);
assert.match(prefixCheck, /TypeExpectation::block_result/);
assert.match(callResolution, /TypeExpectation::outer_consumer_argument/);

console.log('type expectation model source policy passed');

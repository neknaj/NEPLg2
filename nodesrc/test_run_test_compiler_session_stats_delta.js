#!/usr/bin/env node
const assert = require('node:assert/strict');

const { compilerSessionStatsDelta } = require('./run_test');

const before = {
    nepl_meta_materialized_compile_attempts: 2,
    nepl_meta_materialized_compile_attempted_surfaces: 5,
    nepl_meta_materialized_compile_accepts: 0,
    nepl_meta_materialized_compile_source_fallbacks: 1,
    nepl_meta_materialized_compile_source_fallback_successes: 1,
    nepl_meta_materialized_compile_source_fallback_failures: 0,
    nepl_meta_materialized_compile_body_missing_fallbacks: 1,
    nepl_obj_candidate_body_missing_surfaces: 2,
};
const after = {
    nepl_meta_materialized_compile_attempts: 3,
    nepl_meta_materialized_compile_attempted_surfaces: 9,
    nepl_meta_materialized_compile_accepts: 0,
    nepl_meta_materialized_compile_source_fallbacks: 2,
    nepl_meta_materialized_compile_source_fallback_successes: 2,
    nepl_meta_materialized_compile_source_fallback_failures: 0,
    nepl_meta_materialized_compile_body_missing_fallbacks: 2,
    nepl_obj_candidate_body_missing_surfaces: 7,
};

const delta = compilerSessionStatsDelta(before, after);
assert.equal(delta.available, true);
assert.equal(delta.reason, 'ok');
assert.equal(delta.materialized_compile.attempts.delta, 1);
assert.equal(delta.materialized_compile.attempted_surfaces.delta, 4);
assert.equal(delta.materialized_compile.source_fallbacks.delta, 1);
assert.equal(delta.materialized_compile.source_fallback_successes.delta, 1);
assert.equal(delta.materialized_compile.source_fallback_failures.delta, 0);
assert.equal(delta.materialized_compile.body_missing_fallbacks.delta, 1);
assert.equal(delta.materialized_compile.body_missing_candidate_surfaces.delta, 5);

const missing = compilerSessionStatsDelta(before, {
    ...after,
    nepl_meta_materialized_compile_attempts: undefined,
});
assert.equal(missing.available, false);
assert.equal(missing.reason, 'invalid_counter');
assert.equal(missing.counter, 'nepl_meta_materialized_compile_attempts');

const parseError = compilerSessionStatsDelta({ parse_error: 'bad json' }, after);
assert.equal(parseError.available, false);
assert.equal(parseError.reason, 'stats_parse_error');

const decreased = compilerSessionStatsDelta(after, before);
assert.equal(decreased.available, false);
assert.equal(decreased.reason, 'counter_decreased');
assert.equal(decreased.counter, 'nepl_meta_materialized_compile_attempts');

console.log('run_test compiler session stats delta regression passed');

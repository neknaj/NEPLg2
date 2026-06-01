#!/usr/bin/env node
const assert = require('node:assert/strict');

const {
    SCHEMA,
    fallbackDiagnosticCodeCounts,
    publicRunShape,
    sourceForValue,
    summarizeBenchmarkRuns,
} = require('./bench_materialized_compile_fallbacks');

assert.equal(SCHEMA, 'neplg2-materialized-compile-fallback-benchmark/v1');
assert.match(sourceForValue(7), /fn main %fn unit i32 \\unit:/);
assert.match(sourceForValue(7), /\n#import "core\/char" as \*\n/);
assert.match(sourceForValue(7), /\n    char_utf8_cont_byte 7\n/);

const shaped = publicRunShape('candidate', {
    ok: true,
    status: 'pass',
    phase: 'run',
    timing: {
        compile_ms: 120,
        run_ms: 5,
        compiler_session_stats: {
            available: true,
            materialized_compile: {
                attempts: { delta: 1 },
                attempted_surfaces: { delta: 3 },
                source_fallbacks: { delta: 1 },
                source_fallback_successes: { delta: 1 },
                source_fallback_failures: { delta: 0 },
                body_missing_fallbacks: { delta: 1 },
                body_missing_candidate_surfaces: { delta: 3 },
                body_missing_skip_hits: { delta: 0 },
                body_missing_skip_stores: { delta: 3 },
                body_missing_skip_stale_entries: { delta: 0 },
            },
        },
        compiler_session_cache_after: {
            nepl_meta_materialized_compile_last_fallback_reason_code: 1,
            nepl_meta_materialized_compile_last_fallback_diagnostic_code: 'backend.codegen.materialized_function_body_missing',
            nepl_meta_materialized_compile_last_attempted_surfaces: 3,
            nepl_obj_candidate_last_body_missing_surfaces: 3,
            nepl_meta_body_missing_skip_last_hits: 0,
            nepl_meta_body_missing_skip_last_stores: 3,
            compile_stage_timings: [
                { stage: 'resource_typecheck', elapsed_ms: 20.5 },
                { stage: 'resource_static_check', elapsed_ms: 70 },
                { stage: 'wasm_codegen', elapsed_ms: 10 },
            ],
        },
    },
});

assert.equal(shaped.name, 'candidate');
assert.equal(shaped.compile_ms, 120);
assert.equal(shaped.materialized_compile.attempts_delta, 1);
assert.equal(shaped.materialized_compile.body_missing_candidate_surfaces_delta, 3);
assert.equal(shaped.materialized_compile.body_missing_skip_stores_delta, 3);
assert.equal(shaped.materialized_compile.last_body_missing_skip_stores, 3);
assert.equal(shaped.materialized_compile.last_fallback_reason_code, 1);
assert.equal(shaped.materialized_compile.last_fallback_diagnostic_code, 'backend.codegen.materialized_function_body_missing');
assert.equal(shaped.materialized_compile.last_body_missing_candidate_surfaces, 3);
assert.equal(shaped.stages_ms.resource_static_check, 70);

const summary = summarizeBenchmarkRuns([
    shaped,
    {
        ...shaped,
        compile_ms: 80,
        materialized_compile: {
            ...shaped.materialized_compile,
            attempts_delta: 0,
            source_fallbacks_delta: 0,
            body_missing_fallbacks_delta: 0,
            body_missing_candidate_surfaces_delta: 0,
            body_missing_skip_hits_delta: 3,
            body_missing_skip_stores_delta: 0,
            last_fallback_diagnostic_code: '',
        },
    },
]);

assert.equal(summary.runs, 2);
assert.equal(summary.total_compile_ms, 200);
assert.equal(summary.max_compile_ms, 120);
assert.equal(summary.materialized_attempts_delta_sum, 1);
assert.equal(summary.materialized_body_missing_fallbacks_delta_sum, 1);
assert.equal(summary.materialized_non_body_missing_fallbacks_delta_sum, 0);
assert.equal(summary.neplobj_candidate_body_missing_surfaces_delta_sum, 3);
assert.equal(summary.body_missing_skip_hits_delta_sum, 3);
assert.equal(summary.body_missing_skip_stores_delta_sum, 3);
assert.deepEqual(summary.materialized_fallback_diagnostic_code_counts, {
    'backend.codegen.materialized_function_body_missing': 1,
});
assert.deepEqual(fallbackDiagnosticCodeCounts([
    shaped,
    {
        ...shaped,
        materialized_compile: {
            ...shaped.materialized_compile,
            source_fallbacks_delta: 2,
            last_fallback_diagnostic_code: 'type.public_surface.materializer_rejected',
        },
    },
]), {
    'backend.codegen.materialized_function_body_missing': 1,
    'type.public_surface.materializer_rejected': 2,
});

console.log('materialized compile fallback benchmark helper regression passed');

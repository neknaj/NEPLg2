#!/usr/bin/env node
const assert = require('node:assert/strict');

const {
    SCHEMA,
    buildDelta,
    renderMarkdown,
    statNumbers,
    summarizeMetricsJson,
    summarizeTestsJson,
} = require('./compare_git_versions');

assert.equal(SCHEMA, 'neplg2-git-version-comparison/v1');

const stats = statNumbers([10, null, 30, 20, Number.NaN]);
assert.deepEqual(stats, {
    count: 3,
    sum: 60,
    avg: 20,
    min: 10,
    p50: 20,
    p95: 30,
    max: 30,
});

const tests = summarizeTestsJson({
    summary: { total: 3, passed: 2, failed: 1, errored: 0 },
    top_issues: [{ id: 'case#3' }],
    results: [
        { status: 'pass', timing: { compile_ms: 100, run_ms: 5 }, duration_ms: 120 },
        { status: 'pass', timing: { compile_ms: 80, run_ms: 4 }, duration_ms: 100 },
        { status: 'fail', timing: { compile_ms: 40, run_ms: null }, duration_ms: 70 },
    ],
});
assert.equal(tests.total, 3);
assert.equal(tests.passed, 2);
assert.equal(tests.failed, 1);
assert.equal(tests.pass_rate, 2 / 3);
assert.equal(tests.timing.compile_ms.sum, 220);
assert.equal(tests.timing.run_ms.count, 2);
assert.equal(tests.timing.duration_ms.max, 120);

const metrics = summarizeMetricsJson({
    byArea: [
        { name: 'source_tree', files: 2, lines: 100, chars: 1000, bytes: 1100, blank: 5, source: 80, doc_comment: 10, document: 0, test: 5, comment: 0, other: 0, testCases: 1 },
        { name: 'top_level_docs_tests', files: 1, lines: 50, chars: 500, bytes: 550, blank: 4, source: 0, doc_comment: 0, document: 30, test: 16, comment: 0, other: 0, testCases: 2 },
    ],
    byContentKind: [
        { name: 'source', files: 2, lines: 80, chars: 800, bytes: 900, testCases: 0 },
    ],
    byExtension: [
        { name: '.rs', files: 1, lines: 40, chars: 400, bytes: 450, testCases: 0 },
    ],
    skipped: [{ path: 'web/dist/x.wasm', reason: 'binary' }],
});
assert.equal(metrics.totals.files, 3);
assert.equal(metrics.totals.lines, 150);
assert.equal(metrics.totals.doc_comment, 10);
assert.equal(metrics.totals.testCases, 3);
assert.equal(metrics.by_area.source_tree.files, 2);
assert.equal(metrics.skipped.length, 1);

const base = {
    ref: 'base',
    commit: 'aaaaaaaaaaaa',
    tests,
    metrics,
};
const next = {
    ref: 'next',
    commit: 'bbbbbbbbbbbb',
    tests: {
        ...tests,
        passed: 3,
        failed: 0,
        pass_rate: 1,
        timing: {
            compile_ms: { ...tests.timing.compile_ms, sum: 180, avg: 60 },
            run_ms: { ...tests.timing.run_ms, sum: 7, avg: 3.5 },
            duration_ms: { ...tests.timing.duration_ms, sum: 250, avg: 83.3333333333 },
        },
    },
    metrics: {
        ...metrics,
        totals: { ...metrics.totals, lines: 175, source: 95 },
    },
};
const delta = buildDelta(base, next);
assert.equal(delta.tests.passed, 1);
assert.equal(delta.tests.failed, -1);
assert.equal(delta.tests.compile_ms_sum, -40);
assert.equal(delta.metrics.lines, 25);
assert.equal(delta.metrics.source, 15);

const markdown = renderMarkdown({
    generated_at: '2026-05-20T00:00:00.000Z',
    revisions: [base, next],
    deltas: [delta],
});
assert.match(markdown, /NEPLg2 version comparison/);
assert.match(markdown, /base -> next/);
assert.match(markdown, /compile_ms_sum/);

console.log('compare_git_versions summary regression passed');

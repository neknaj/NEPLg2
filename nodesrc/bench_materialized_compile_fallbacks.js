#!/usr/bin/env node
// nodesrc/bench_materialized_compile_fallbacks.js
// 目的:
// - 同一 CompilerSession で `.neplmeta` store を温めながら複数回 compile し、
//   materialized compile fallback と `.neplobj` candidate surface 数を JSON で観測する。
// - 通常 test runner の worker 分散や fixture 順序に依存せず、base / warm edit の compile_ms と
//   compiler-internal counter delta を同じ report として保存する。

const fs = require('node:fs');
const path = require('node:path');

const { createRunner, runSingle } = require('./run_test');

const SCHEMA = 'neplg2-materialized-compile-fallback-benchmark/v1';

function parseArgs(argv) {
    const args = {
        out: '',
        distHint: '',
        preseedNeplMeta: false,
    };
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if ((arg === '-o' || arg === '--out') && i + 1 < argv.length) {
            args.out = argv[++i];
        } else if (arg === '--dist-hint' && i + 1 < argv.length) {
            args.distHint = argv[++i];
        } else if (arg === '--preseed-neplmeta') {
            args.preseedNeplMeta = true;
        } else if (arg === '-h' || arg === '--help') {
            return { ...args, help: true };
        } else {
            throw new Error(`unknown argument: ${arg}`);
        }
    }
    return args;
}

function usage() {
    return [
        'Usage: node nodesrc/bench_materialized_compile_fallbacks.js [--out tmp/report.json] [--dist-hint web/dist] [--preseed-neplmeta]',
        '',
        'Runs a fixed same-session compile sequence and reports materialized compile fallback deltas.',
    ].join('\n');
}

function sourceForValue(value) {
    return [
        '#entry main',
        '#import "core/char" as *',
        'fn main %fn void i32 \\void:',
        `    char_utf8_cont_byte ${Number(value) | 0}`,
        '',
    ].join('\n');
}

function numeric(value) {
    if (value === null || value === undefined || value === '') return null;
    const n = Number(value);
    return Number.isFinite(n) ? n : null;
}

function deltaOf(result, name) {
    const stats = result?.timing?.compiler_session_stats;
    if (!stats || stats.available !== true) return null;
    return numeric(stats?.materialized_compile?.[name]?.delta);
}

function sum(values) {
    return values.reduce((acc, value) => acc + (Number.isFinite(value) ? value : 0), 0);
}

function fallbackDiagnosticCodeCounts(runs) {
    const counts = {};
    for (const run of runs) {
        const code = String(run.materialized_compile?.last_fallback_diagnostic_code || '');
        const fallbacks = numeric(run.materialized_compile?.source_fallbacks_delta);
        if (code === '' || !Number.isFinite(fallbacks) || fallbacks <= 0) {
            continue;
        }
        counts[code] = (counts[code] || 0) + fallbacks;
    }
    return counts;
}

function stageTiming(result, stage) {
    const rows = result?.timing?.compiler_session_cache_after?.compile_stage_timings;
    if (!Array.isArray(rows)) return null;
    const found = rows.find((row) => row && row.stage === stage);
    return numeric(found?.elapsed_ms);
}

function summarizeBenchmarkRuns(runs) {
    const compileMsValues = runs.map((run) => numeric(run.compile_ms)).filter(Number.isFinite);
    const sourceFallbacks = sum(runs.map((run) => numeric(run.materialized_compile?.source_fallbacks_delta)));
    const bodyMissingFallbacks = sum(runs.map((run) => numeric(run.materialized_compile?.body_missing_fallbacks_delta)));
    return {
        runs: runs.length,
        total_compile_ms: sum(compileMsValues),
        max_compile_ms: compileMsValues.length > 0 ? Math.max(...compileMsValues) : null,
        materialized_attempts_delta_sum: sum(runs.map((run) => numeric(run.materialized_compile?.attempts_delta))),
        materialized_source_fallbacks_delta_sum: sourceFallbacks,
        materialized_body_missing_fallbacks_delta_sum: bodyMissingFallbacks,
        materialized_non_body_missing_fallbacks_delta_sum: sourceFallbacks - bodyMissingFallbacks,
        neplobj_candidate_body_missing_surfaces_delta_sum: sum(runs.map((run) => numeric(run.materialized_compile?.body_missing_candidate_surfaces_delta))),
        body_missing_skip_hits_delta_sum: sum(runs.map((run) => numeric(run.materialized_compile?.body_missing_skip_hits_delta))),
        body_missing_skip_stores_delta_sum: sum(runs.map((run) => numeric(run.materialized_compile?.body_missing_skip_stores_delta))),
        materialized_fallback_diagnostic_code_counts: fallbackDiagnosticCodeCounts(runs),
    };
}

function publicRunShape(name, result) {
    return {
        name,
        ok: Boolean(result?.ok),
        status: result?.status || null,
        phase: result?.phase || null,
        compile_ms: numeric(result?.timing?.compile_ms),
        run_ms: numeric(result?.timing?.run_ms),
        compiler_session_stats_available: result?.timing?.compiler_session_stats?.available === true,
        materialized_compile: {
            attempts_delta: deltaOf(result, 'attempts'),
            attempted_surfaces_delta: deltaOf(result, 'attempted_surfaces'),
            source_fallbacks_delta: deltaOf(result, 'source_fallbacks'),
            source_fallback_successes_delta: deltaOf(result, 'source_fallback_successes'),
            source_fallback_failures_delta: deltaOf(result, 'source_fallback_failures'),
            body_missing_fallbacks_delta: deltaOf(result, 'body_missing_fallbacks'),
            body_missing_candidate_surfaces_delta: deltaOf(result, 'body_missing_candidate_surfaces'),
            body_missing_skip_hits_delta: deltaOf(result, 'body_missing_skip_hits'),
            body_missing_skip_stores_delta: deltaOf(result, 'body_missing_skip_stores'),
            body_missing_skip_stale_entries_delta: deltaOf(result, 'body_missing_skip_stale_entries'),
            last_fallback_reason_code: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_reason_code),
            last_fallback_diagnostic_code: result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_diagnostic_code || '',
            last_fallback_diagnostic_message: result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_diagnostic_message || '',
            last_fallback_diagnostic_path: result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_diagnostic_path || '',
            last_fallback_diagnostic_start: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_diagnostic_start),
            last_fallback_diagnostic_end: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_fallback_diagnostic_end),
            last_attempted_surfaces: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_materialized_compile_last_attempted_surfaces),
            last_body_missing_candidate_surfaces: numeric(result?.timing?.compiler_session_cache_after?.nepl_obj_candidate_last_body_missing_surfaces),
            last_body_missing_skip_hits: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_body_missing_skip_last_hits),
            last_body_missing_skip_stores: numeric(result?.timing?.compiler_session_cache_after?.nepl_meta_body_missing_skip_last_stores),
        },
        stages_ms: {
            resource_typecheck: stageTiming(result, 'resource_typecheck'),
            resource_static_check: stageTiming(result, 'resource_static_check'),
            wasm_codegen: stageTiming(result, 'wasm_codegen'),
        },
    };
}

async function runBenchmark(args) {
    const loaded = await createRunner(args.distHint || '');
    const session = loaded?.meta?.compilerSession;
    const preseedAvailable =
        Boolean(session)
        && typeof session.preseed_nepl_meta_artifacts_for_source === 'function';
    const preseed = {
        enabled: Boolean(args.preseedNeplMeta),
        available: preseedAvailable,
        artifact_count: null,
        elapsed_ms: null,
        error: null,
    };
    if (args.preseedNeplMeta) {
        if (preseedAvailable) {
            const start = Date.now();
            try {
                preseed.artifact_count = session.preseed_nepl_meta_artifacts_for_source(
                    '/virtual/entry.nepl',
                    sourceForValue(1),
                );
            } catch (error) {
                preseed.error = String(error && error.message ? error.message : error);
            } finally {
                preseed.elapsed_ms = Date.now() - start;
            }
        } else {
            preseed.error = 'missing CompilerSession.preseed_nepl_meta_artifacts_for_source';
        }
    }
    const steps = [
        ['cold_base', 1],
        ['warm_store_probe', 2],
        ['body_edit_candidate', 3],
        ['body_edit_repeat', 4],
    ];
    const runs = [];
    for (const [name, value] of steps) {
        const result = await runSingle({
            id: name,
            source: sourceForValue(value),
            tags: [],
            stdin: '',
            distHint: args.distHint || '',
        }, loaded);
        runs.push(publicRunShape(name, result));
    }
    return {
        schema: SCHEMA,
        generated_at: new Date().toISOString(),
        preseed,
        summary: summarizeBenchmarkRuns(runs),
        runs,
    };
}

async function main() {
    const args = parseArgs(process.argv.slice(2));
    if (args.help) {
        console.log(usage());
        return 0;
    }
    const report = await runBenchmark(args);
    const json = JSON.stringify(report, null, 2);
    if (args.out) {
        fs.mkdirSync(path.dirname(path.resolve(args.out)), { recursive: true });
        fs.writeFileSync(args.out, `${json}\n`, 'utf8');
    }
    console.log(json);
    return 0;
}

if (require.main === module) {
    main().then((code) => {
        process.exitCode = code;
    }).catch((error) => {
        console.error(String(error?.stack || error?.message || error));
        process.exitCode = 1;
    });
}

module.exports = {
    SCHEMA,
    fallbackDiagnosticCodeCounts,
    publicRunShape,
    runBenchmark,
    sourceForValue,
    summarizeBenchmarkRuns,
};

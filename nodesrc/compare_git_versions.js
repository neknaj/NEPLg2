#!/usr/bin/env node
// nodesrc/compare_git_versions.js
// 目的:
// - 指定した git commit / ref ごとに一時 worktree を作り、同一入力でテスト結果と repo metrics を比較する。
// - 現在の作業ツリーを移動せず、長期開発中の性能・規模・通過率の変化を commit 単位で確認できるようにする。

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const SCHEMA = 'neplg2-git-version-comparison/v1';

function parseArgs(argv) {
    const args = {
        refs: [],
        inputs: [],
        out: '',
        markdown: '',
        jobs: 1,
        runner: 'wasm',
        assertIo: false,
        withTree: false,
        metricsOnly: false,
        distCurrent: '',
        distRel: 'web/dist',
        buildCmd: '',
        keepWorktrees: false,
        workRoot: path.join('tmp', 'version_compare'),
        commandTimeoutMs: 15 * 60 * 1000,
    };

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '--rev' && i + 1 < argv.length) {
            args.refs.push(argv[++i]);
        } else if (a === '-i' && i + 1 < argv.length) {
            args.inputs.push(argv[++i]);
        } else if ((a === '-o' || a === '--out') && i + 1 < argv.length) {
            args.out = argv[++i];
        } else if (a === '--markdown' && i + 1 < argv.length) {
            args.markdown = argv[++i];
        } else if ((a === '-j' || a === '--jobs') && i + 1 < argv.length) {
            args.jobs = Number(argv[++i]);
        } else if (a === '--runner' && i + 1 < argv.length) {
            args.runner = argv[++i];
        } else if (a === '--assert-io') {
            args.assertIo = true;
        } else if (a === '--with-tree') {
            args.withTree = true;
        } else if (a === '--no-tree') {
            args.withTree = false;
        } else if (a === '--metrics-only') {
            args.metricsOnly = true;
        } else if (a === '--dist-current' && i + 1 < argv.length) {
            args.distCurrent = argv[++i];
        } else if (a === '--dist-rel' && i + 1 < argv.length) {
            args.distRel = argv[++i];
        } else if (a === '--build-cmd' && i + 1 < argv.length) {
            args.buildCmd = argv[++i];
        } else if (a === '--keep-worktrees') {
            args.keepWorktrees = true;
        } else if (a === '--work-root' && i + 1 < argv.length) {
            args.workRoot = argv[++i];
        } else if (a === '--command-timeout-ms' && i + 1 < argv.length) {
            args.commandTimeoutMs = Number(argv[++i]);
        } else if (a === '-h' || a === '--help') {
            return { ...args, help: true };
        } else {
            throw new Error(`unknown argument: ${a}`);
        }
    }

    if (!Number.isFinite(args.jobs) || args.jobs <= 0) {
        throw new Error('--jobs must be a positive number');
    }
    if (!['wasm', 'llvm', 'all'].includes(args.runner)) {
        throw new Error('--runner must be wasm|llvm|all');
    }
    if (!Number.isFinite(args.commandTimeoutMs) || args.commandTimeoutMs <= 0) {
        throw new Error('--command-timeout-ms must be a positive number');
    }
    return args;
}

function printUsage() {
    console.log('Usage: node nodesrc/compare_git_versions.js --rev <ref> --rev <ref> [options]');
    console.log('');
    console.log('Options:');
    console.log('  -i <path>                 Doctest input path, repeated. Relative paths are resolved inside each worktree.');
    console.log('  -o, --out <json>          Write structured comparison JSON.');
    console.log('  --markdown <md>           Write Markdown summary table.');
    console.log('  -j, --jobs <n>            Jobs passed to nodesrc/tests.js (default: 1).');
    console.log('  --runner <wasm|llvm|all>  Test runner passed to nodesrc/tests.js.');
    console.log('  --assert-io               Pass --assert-io to nodesrc/tests.js.');
    console.log('  --with-tree               Include tree tests. By default this tool passes --no-tree.');
    console.log('  --no-tree                 Keep tree tests disabled explicitly.');
    console.log('  --metrics-only            Skip tests and collect repo_metrics.ts only.');
    console.log('  --dist-current <path>     Use one dist directory from the current checkout for all refs.');
    console.log('  --dist-rel <path>         Dist directory relative to each worktree (default: web/dist).');
    console.log('  --build-cmd <cmd>         Shell command run inside each worktree before tests.');
    console.log('  --keep-worktrees          Keep temporary worktrees for inspection.');
    console.log('  --work-root <path>        Temporary worktree/output root (default: tmp/version_compare).');
}

function run(cmd, opts = {}) {
    const proc = spawnSync(cmd[0], cmd.slice(1), {
        cwd: opts.cwd || process.cwd(),
        encoding: 'utf8',
        timeout: opts.timeoutMs || 15 * 60 * 1000,
        maxBuffer: 64 * 1024 * 1024,
        env: { ...process.env, ...(opts.env || {}) },
        shell: !!opts.shell,
    });
    return {
        command: cmd.join(' '),
        cwd: opts.cwd || process.cwd(),
        status: proc.status,
        signal: proc.signal || null,
        stdout: proc.stdout || '',
        stderr: proc.stderr || '',
        error: proc.error ? String(proc.error.message || proc.error) : null,
    };
}

function requireOk(result, label) {
    if (result.status === 0 && !result.signal && !result.error) return result;
    const detail = [result.stderr, result.stdout, result.error].filter(Boolean).join('\n').trim();
    throw new Error(`${label} failed: ${result.command}\n${detail}`);
}

function repoRoot() {
    const r = requireOk(run(['git', 'rev-parse', '--show-toplevel']), 'git rev-parse');
    return path.resolve(r.stdout.trim());
}

function resolveCommit(ref) {
    const r = requireOk(run(['git', 'rev-parse', '--verify', `${ref}^{commit}`]), `resolve ref ${ref}`);
    return r.stdout.trim();
}

function safeName(text) {
    return String(text).replace(/[^A-Za-z0-9_.-]+/g, '_').replace(/^_+|_+$/g, '').slice(0, 80) || 'ref';
}

function ensureDir(dir) {
    fs.mkdirSync(dir, { recursive: true });
}

function readJson(file) {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function writeJson(file, value) {
    ensureDir(path.dirname(path.resolve(file)));
    fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function statNumbers(values) {
    const nums = values.filter((v) => Number.isFinite(v)).sort((a, b) => a - b);
    if (nums.length === 0) {
        return { count: 0, sum: 0, avg: null, min: null, p50: null, p95: null, max: null };
    }
    const sum = nums.reduce((a, b) => a + b, 0);
    const pick = (q) => nums[Math.min(nums.length - 1, Math.max(0, Math.ceil(nums.length * q) - 1))];
    return {
        count: nums.length,
        sum,
        avg: sum / nums.length,
        min: nums[0],
        p50: pick(0.50),
        p95: pick(0.95),
        max: nums[nums.length - 1],
    };
}

function numeric(value) {
    if (value === null || value === undefined || value === '') return null;
    const n = Number(value);
    return Number.isFinite(n) ? n : null;
}

const MATERIALIZED_COMPILE_DELTA_COUNTERS = [
    ['attempts_delta', 'attempts'],
    ['attempted_surfaces_delta', 'attempted_surfaces'],
    ['accepts_delta', 'accepts'],
    ['source_fallbacks_delta', 'source_fallbacks'],
    ['source_fallback_successes_delta', 'source_fallback_successes'],
    ['source_fallback_failures_delta', 'source_fallback_failures'],
    ['body_missing_fallbacks_delta', 'body_missing_fallbacks'],
];

function materializedCompileDelta(result, counterName) {
    const stats = result?.timing?.compiler_session_stats;
    if (!stats || stats.available !== true) return null;
    return numeric(stats?.materialized_compile?.[counterName]?.delta);
}

function summarizeMaterializedCompile(results) {
    const reasonCounts = {};
    let available = 0;
    for (const result of results) {
        const stats = result?.timing?.compiler_session_stats;
        if (stats && stats.available === true) {
            available += 1;
            continue;
        }
        const reason = stats && stats.reason ? String(stats.reason) : 'missing_stats';
        reasonCounts[reason] = (reasonCounts[reason] || 0) + 1;
    }
    const out = {
        available_results: available,
        unavailable_results: results.length - available,
        unavailable_reasons: reasonCounts,
    };
    for (const [summaryName, counterName] of MATERIALIZED_COMPILE_DELTA_COUNTERS) {
        out[summaryName] = statNumbers(results.map((r) => materializedCompileDelta(r, counterName)));
    }
    return out;
}

function summarizeTestsJson(testJson) {
    if (!testJson) return null;
    const results = Array.isArray(testJson.results) ? testJson.results : [];
    const summary = testJson.summary || {};
    const total = Number(summary.total ?? results.length) || 0;
    const passed = Number(summary.passed ?? results.filter((r) => r.status === 'pass').length) || 0;
    const failed = Number(summary.failed ?? results.filter((r) => r.status === 'fail').length) || 0;
    const errored = Number(summary.errored ?? results.filter((r) => r.status === 'error').length) || 0;
    return {
        total,
        passed,
        failed,
        errored,
        pass_rate: total > 0 ? passed / total : null,
        timing: {
            compile_ms: statNumbers(results.map((r) => numeric(r?.timing?.compile_ms))),
            run_ms: statNumbers(results.map((r) => numeric(r?.timing?.run_ms))),
            duration_ms: statNumbers(results.map((r) => numeric(r?.duration_ms))),
            materialized_compile: summarizeMaterializedCompile(results),
        },
        top_issues: Array.isArray(testJson.top_issues) ? testJson.top_issues : [],
    };
}

function sumRows(rows) {
    const totals = {
        files: 0,
        lines: 0,
        chars: 0,
        bytes: 0,
        blank: 0,
        source: 0,
        doc_comment: 0,
        document: 0,
        test: 0,
        comment: 0,
        other: 0,
        testCases: 0,
    };
    for (const row of rows || []) {
        for (const key of Object.keys(totals)) {
            const value = row[key] ?? (key === 'testCases' ? row.testCases : undefined);
            totals[key] += Number(value) || 0;
        }
    }
    return totals;
}

function summarizeMetricsJson(metricsJson) {
    if (!metricsJson) return null;
    const byArea = Array.isArray(metricsJson.byArea) ? metricsJson.byArea : [];
    const byContentKind = Array.isArray(metricsJson.byContentKind) ? metricsJson.byContentKind : [];
    const byExtension = Array.isArray(metricsJson.byExtension) ? metricsJson.byExtension : [];
    return {
        totals: sumRows(byArea),
        by_area: Object.fromEntries(byArea.map((r) => [r.name, r])),
        by_content_kind: Object.fromEntries(byContentKind.map((r) => [r.name, r])),
        by_extension: Object.fromEntries(byExtension.map((r) => [r.name, r])),
        skipped: Array.isArray(metricsJson.skipped) ? metricsJson.skipped : [],
    };
}

function deltaNumber(base, next) {
    if (!Number.isFinite(base) || !Number.isFinite(next)) return null;
    return next - base;
}

function buildDelta(base, next) {
    const bt = base.tests;
    const nt = next.tests;
    const bm = base.metrics?.totals || {};
    const nm = next.metrics?.totals || {};
    return {
        ref: next.ref,
        commit: next.commit,
        base_ref: base.ref,
        base_commit: base.commit,
        tests: bt && nt ? {
            total: deltaNumber(bt.total, nt.total),
            passed: deltaNumber(bt.passed, nt.passed),
            failed: deltaNumber(bt.failed, nt.failed),
            errored: deltaNumber(bt.errored, nt.errored),
            pass_rate: deltaNumber(bt.pass_rate, nt.pass_rate),
            compile_ms_sum: deltaNumber(bt.timing.compile_ms.sum, nt.timing.compile_ms.sum),
            compile_ms_avg: deltaNumber(bt.timing.compile_ms.avg, nt.timing.compile_ms.avg),
            run_ms_sum: deltaNumber(bt.timing.run_ms.sum, nt.timing.run_ms.sum),
            run_ms_avg: deltaNumber(bt.timing.run_ms.avg, nt.timing.run_ms.avg),
            duration_ms_sum: deltaNumber(bt.timing.duration_ms.sum, nt.timing.duration_ms.sum),
            duration_ms_avg: deltaNumber(bt.timing.duration_ms.avg, nt.timing.duration_ms.avg),
            materialized_compile_available_results: deltaNumber(
                bt.timing.materialized_compile.available_results,
                nt.timing.materialized_compile.available_results,
            ),
            materialized_compile_attempts_delta_sum: deltaNumber(
                bt.timing.materialized_compile.attempts_delta.sum,
                nt.timing.materialized_compile.attempts_delta.sum,
            ),
            materialized_compile_source_fallbacks_delta_sum: deltaNumber(
                bt.timing.materialized_compile.source_fallbacks_delta.sum,
                nt.timing.materialized_compile.source_fallbacks_delta.sum,
            ),
            materialized_compile_source_fallback_successes_delta_sum: deltaNumber(
                bt.timing.materialized_compile.source_fallback_successes_delta.sum,
                nt.timing.materialized_compile.source_fallback_successes_delta.sum,
            ),
            materialized_compile_source_fallback_failures_delta_sum: deltaNumber(
                bt.timing.materialized_compile.source_fallback_failures_delta.sum,
                nt.timing.materialized_compile.source_fallback_failures_delta.sum,
            ),
            materialized_compile_body_missing_fallbacks_delta_sum: deltaNumber(
                bt.timing.materialized_compile.body_missing_fallbacks_delta.sum,
                nt.timing.materialized_compile.body_missing_fallbacks_delta.sum,
            ),
        } : null,
        metrics: {
            files: deltaNumber(bm.files, nm.files),
            lines: deltaNumber(bm.lines, nm.lines),
            bytes: deltaNumber(bm.bytes, nm.bytes),
            source: deltaNumber(bm.source, nm.source),
            doc_comment: deltaNumber(bm.doc_comment, nm.doc_comment),
            document: deltaNumber(bm.document, nm.document),
            test: deltaNumber(bm.test, nm.test),
            testCases: deltaNumber(bm.testCases, nm.testCases),
        },
    };
}

function fmt(value, digits = 1) {
    if (value === null || value === undefined || Number.isNaN(value)) return '-';
    if (typeof value === 'number') {
        return Number.isInteger(value) ? String(value) : value.toFixed(digits);
    }
    return String(value);
}

function pct(value) {
    return value === null || value === undefined ? '-' : `${(value * 100).toFixed(2)}%`;
}

function markdownTable(headers, rows) {
    const lines = [];
    lines.push(`| ${headers.join(' | ')} |`);
    lines.push(`| ${headers.map(() => '---').join(' | ')} |`);
    for (const row of rows) {
        lines.push(`| ${row.map((v) => String(v).replace(/\|/g, '\\|')).join(' | ')} |`);
    }
    return lines.join('\n');
}

function renderMarkdown(report) {
    const rows = report.revisions.map((r) => {
        const t = r.tests;
        const m = r.metrics?.totals || {};
        return [
            r.ref,
            r.commit.slice(0, 12),
            t ? fmt(t.total, 0) : '-',
            t ? fmt(t.passed, 0) : '-',
            t ? fmt(t.failed, 0) : '-',
            t ? fmt(t.errored, 0) : '-',
            t ? pct(t.pass_rate) : '-',
            t ? fmt(t.timing.compile_ms.sum, 0) : '-',
            t ? fmt(t.timing.run_ms.sum, 0) : '-',
            t ? fmt(t.timing.duration_ms.sum, 0) : '-',
            fmt(m.files, 0),
            fmt(m.lines, 0),
            fmt(m.source, 0),
            fmt(m.doc_comment, 0),
            fmt(m.test, 0),
            fmt(m.testCases, 0),
        ];
    });
    const parts = [
        `# NEPLg2 version comparison`,
        '',
        `Generated: ${report.generated_at}`,
        '',
        markdownTable(
            ['ref', 'commit', 'total', 'passed', 'failed', 'errored', 'pass_rate', 'compile_ms_sum', 'run_ms_sum', 'duration_ms_sum', 'files', 'lines', 'source', 'doc_comment', 'test_lines', 'test_cases'],
            rows,
        ),
    ];
    const materializedRows = report.revisions.map((r) => {
        const m = r.tests?.timing?.materialized_compile;
        return [
            r.ref,
            r.commit.slice(0, 12),
            m ? fmt(m.available_results, 0) : '-',
            m ? fmt(m.unavailable_results, 0) : '-',
            m ? fmt(m.attempts_delta.sum, 0) : '-',
            m ? fmt(m.source_fallbacks_delta.sum, 0) : '-',
            m ? fmt(m.source_fallback_successes_delta.sum, 0) : '-',
            m ? fmt(m.source_fallback_failures_delta.sum, 0) : '-',
            m ? fmt(m.body_missing_fallbacks_delta.sum, 0) : '-',
        ];
    });
    parts.push('', '## Materialized Compile', '', markdownTable(
        [
            'ref',
            'commit',
            'available_results',
            'unavailable_results',
            'attempts_delta_sum',
            'source_fallbacks_delta_sum',
            'source_fallback_successes_delta_sum',
            'source_fallback_failures_delta_sum',
            'body_missing_fallbacks_delta_sum',
        ],
        materializedRows,
    ));
    if (report.deltas.length > 0) {
        const deltaRows = report.deltas.map((d) => [
            `${d.base_ref} -> ${d.ref}`,
            d.commit.slice(0, 12),
            d.tests ? fmt(d.tests.passed, 0) : '-',
            d.tests ? fmt(d.tests.failed, 0) : '-',
            d.tests ? fmt(d.tests.pass_rate, 4) : '-',
            d.tests ? fmt(d.tests.compile_ms_sum, 0) : '-',
            d.tests ? fmt(d.tests.run_ms_sum, 0) : '-',
            fmt(d.metrics.lines, 0),
            fmt(d.metrics.source, 0),
            fmt(d.metrics.doc_comment, 0),
            fmt(d.metrics.test, 0),
        ]);
        parts.push('', '## Delta from first ref', '', markdownTable(
            ['range', 'commit', 'passed', 'failed', 'pass_rate', 'compile_ms_sum', 'run_ms_sum', 'lines', 'source', 'doc_comment', 'test_lines'],
            deltaRows,
        ));
        const materializedDeltaRows = report.deltas.map((d) => [
            `${d.base_ref} -> ${d.ref}`,
            d.commit.slice(0, 12),
            d.tests ? fmt(d.tests.materialized_compile_available_results, 0) : '-',
            d.tests ? fmt(d.tests.materialized_compile_attempts_delta_sum, 0) : '-',
            d.tests ? fmt(d.tests.materialized_compile_source_fallbacks_delta_sum, 0) : '-',
            d.tests ? fmt(d.tests.materialized_compile_source_fallback_successes_delta_sum, 0) : '-',
            d.tests ? fmt(d.tests.materialized_compile_source_fallback_failures_delta_sum, 0) : '-',
            d.tests ? fmt(d.tests.materialized_compile_body_missing_fallbacks_delta_sum, 0) : '-',
        ]);
        parts.push('', '## Materialized Compile Delta from first ref', '', markdownTable(
            [
                'range',
                'commit',
                'available_results',
                'attempts_delta_sum',
                'source_fallbacks_delta_sum',
                'source_fallback_successes_delta_sum',
                'source_fallback_failures_delta_sum',
                'body_missing_fallbacks_delta_sum',
            ],
            materializedDeltaRows,
        ));
    }
    return `${parts.join('\n')}\n`;
}

function removeWorktree(repo, worktreePath) {
    const resolved = path.resolve(worktreePath);
    run(['git', 'worktree', 'remove', '--force', resolved], { cwd: repo, timeoutMs: 120000 });
    if (fs.existsSync(resolved)) {
        fs.rmSync(resolved, { recursive: true, force: true });
    }
}

function compareOne(repo, scriptRoot, args, ref, index, runId) {
    const commit = resolveCommit(ref);
    const workRootAbs = path.resolve(repo, args.workRoot);
    const worktreeRoot = path.join(workRootAbs, 'worktrees');
    const artifactsRoot = path.join(workRootAbs, 'artifacts', `${String(index + 1).padStart(2, '0')}-${safeName(ref)}-${commit.slice(0, 12)}`);
    ensureDir(worktreeRoot);
    ensureDir(artifactsRoot);
    const worktreePath = path.join(worktreeRoot, `${runId}-${String(index + 1).padStart(2, '0')}-${safeName(ref)}-${commit.slice(0, 12)}`);
    const commands = [];
    let testJson = null;
    let metricsJson = null;
    let testError = null;
    let metricsError = null;
    let buildResult = null;

    try {
        const add = run(['git', 'worktree', 'add', '--detach', worktreePath, commit], { cwd: repo, timeoutMs: 120000 });
        commands.push({ kind: 'git_worktree_add', ...add });
        requireOk(add, `git worktree add ${ref}`);

        if (args.buildCmd) {
            buildResult = run([args.buildCmd], {
                cwd: worktreePath,
                shell: true,
                timeoutMs: args.commandTimeoutMs,
            });
            commands.push({ kind: 'build', ...buildResult });
            requireOk(buildResult, `build ${ref}`);
        }

        const metricsOut = path.join(artifactsRoot, 'repo_metrics.json');
        const metricsRun = run([
            process.execPath,
            '--experimental-strip-types',
            path.join(scriptRoot, 'repo_metrics.ts'),
            '--root',
            worktreePath,
            '--mode',
            'git',
            '--json',
            metricsOut,
        ], { cwd: repo, timeoutMs: args.commandTimeoutMs });
        commands.push({ kind: 'repo_metrics', ...metricsRun });
        if (metricsRun.status === 0 && fs.existsSync(metricsOut)) {
            metricsJson = readJson(metricsOut);
        } else {
            metricsError = [metricsRun.stderr, metricsRun.stdout, metricsRun.error].filter(Boolean).join('\n').trim();
        }

        if (!args.metricsOnly && args.inputs.length > 0) {
            const testsOut = path.join(artifactsRoot, 'tests.json');
            const distPath = args.distCurrent
                ? path.resolve(repo, args.distCurrent)
                : path.resolve(worktreePath, args.distRel);
            const testArgs = [
                process.execPath,
                path.join(scriptRoot, 'nodesrc', 'tests.js'),
            ];
            for (const input of args.inputs) {
                testArgs.push('-i', input);
            }
            testArgs.push('-o', testsOut, '--dist', distPath, '-j', String(args.jobs), '--runner', args.runner);
            if (!args.withTree) testArgs.push('--no-tree');
            if (args.assertIo) testArgs.push('--assert-io');
            const testsRun = run(testArgs, {
                cwd: worktreePath,
                timeoutMs: args.commandTimeoutMs,
            });
            commands.push({ kind: 'tests', ...testsRun });
            if (fs.existsSync(testsOut)) {
                testJson = readJson(testsOut);
            }
            if (testsRun.status !== 0 && !testJson) {
                testError = [testsRun.stderr, testsRun.stdout, testsRun.error].filter(Boolean).join('\n').trim();
            }
        }
    } finally {
        if (!args.keepWorktrees) {
            removeWorktree(repo, worktreePath);
        }
    }

    return {
        ref,
        commit,
        worktree: args.keepWorktrees ? worktreePath : null,
        artifacts: artifactsRoot,
        build: buildResult ? {
            status: buildResult.status,
            signal: buildResult.signal,
        } : null,
        tests: summarizeTestsJson(testJson),
        test_error: testError,
        metrics: summarizeMetricsJson(metricsJson),
        metrics_error: metricsError,
        commands: commands.map((c) => ({
            kind: c.kind,
            command: c.command,
            cwd: c.cwd,
            status: c.status,
            signal: c.signal,
            error: c.error,
            stderr_tail: String(c.stderr || '').slice(-4000),
            stdout_tail: String(c.stdout || '').slice(-4000),
        })),
    };
}

function main() {
    const args = parseArgs(process.argv.slice(2));
    if (args.help) {
        printUsage();
        return 0;
    }
    if (args.refs.length === 0) {
        throw new Error('at least one --rev is required');
    }
    if (!args.out && !args.markdown) {
        throw new Error('one of --out or --markdown is required');
    }
    if (!args.metricsOnly && args.inputs.length === 0) {
        throw new Error('test comparison needs at least one -i input; use --metrics-only to skip tests');
    }

    const repo = repoRoot();
    const runId = `${Date.now()}-${process.pid}-${Math.random().toString(16).slice(2)}`;
    const revisions = args.refs.map((ref, index) => compareOne(repo, repo, args, ref, index, runId));
    const base = revisions[0];
    const deltas = revisions.slice(1).map((r) => buildDelta(base, r));
    const report = {
        schema: SCHEMA,
        generated_at: new Date().toISOString(),
        options: {
            refs: args.refs,
            inputs: args.inputs,
            jobs: args.jobs,
            runner: args.runner,
            assert_io: args.assertIo,
            with_tree: args.withTree,
            metrics_only: args.metricsOnly,
            dist_current: args.distCurrent || null,
            dist_rel: args.distRel,
            build_cmd: args.buildCmd || null,
            keep_worktrees: args.keepWorktrees,
        },
        revisions,
        deltas,
    };

    if (args.out) writeJson(args.out, report);
    if (args.markdown) {
        ensureDir(path.dirname(path.resolve(args.markdown)));
        fs.writeFileSync(args.markdown, renderMarkdown(report), 'utf8');
    }
    console.log(JSON.stringify({
        schema: report.schema,
        generated_at: report.generated_at,
        revisions: report.revisions.map((r) => ({
            ref: r.ref,
            commit: r.commit,
            tests: r.tests ? {
                total: r.tests.total,
                passed: r.tests.passed,
                failed: r.tests.failed,
                errored: r.tests.errored,
                pass_rate: r.tests.pass_rate,
            } : null,
            metrics: r.metrics ? r.metrics.totals : null,
            test_error: r.test_error,
            metrics_error: r.metrics_error,
        })),
        out: args.out || null,
        markdown: args.markdown || null,
    }, null, 2));
    return 0;
}

if (require.main === module) {
    try {
        process.exitCode = main();
    } catch (error) {
        console.error(String(error?.stack || error?.message || error));
        process.exitCode = 1;
    }
}

module.exports = {
    SCHEMA,
    buildDelta,
    renderMarkdown,
    statNumbers,
    summarizeMetricsJson,
    summarizeTestsJson,
};

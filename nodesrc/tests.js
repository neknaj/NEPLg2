#!/usr/bin/env node
// nodesrc/tests.js
// 目的:
// - /tests/compiler/*.n.md, /tests/stdlib/*.n.md, /tutorials/**/*.n.md, /stdlib/**/*.nepl などに埋め込まれた doctest を走査して実行し、結果を JSON にまとめる。
//
// 使い方例:
//   node nodesrc/tests.js -i tests -i tutorials -i stdlib -o dist/tests.json
//
// 注意:
// - dist の場所は自動探索（dist / web/dist）だが、NEPL_DIST 環境変数で強制できる。

const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn, spawnSync } = require('node:child_process');
const { Worker } = require('node:worker_threads');
const { parseFile } = require('./parser');
const { createRunner, runSingle } = require('./run_test');
const { runTreeSuite } = require('../tests/compiler/tree/run');

const DEFAULT_TEST_CASE_TIMEOUT_MS = 60000;

// doctest 集計の標準出力は要約重視にする。
process.removeAllListeners('warning');
process.on('warning', () => {});

function parseArgs(argv) {
    const inputs = [];
    let outPath = '';
    let distHint = '';
    let jobs = 0;
    let includeStdlib = false;
    let includeStdlibTouched = false;
    let includeTree = true;
    let includeTreeTouched = false;
    let runner = 'wasm';
    let llvmAll = false;
    let assertIo = false;
    let strictDual = false;
    let llvmCompileOnly = false;
    let changedOnly = false;
    let changedBase = 'HEAD';

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '-i' && i + 1 < argv.length) {
            inputs.push(argv[++i]);
            continue;
        }
        if (a === '-o' && i + 1 < argv.length) {
            outPath = argv[++i];
            continue;
        }
        if ((a === '--dist' || a === '--dist-hint') && i + 1 < argv.length) {
            distHint = argv[++i];
            continue;
        }
        if ((a === '-j' || a === '--jobs') && i + 1 < argv.length) {
            jobs = parseInt(argv[++i], 10);
            continue;
        }
        if (a === '--no-stdlib') {
            includeStdlib = false;
            includeStdlibTouched = true;
            continue;
        }
        if (a === '--with-stdlib') {
            includeStdlib = true;
            includeStdlibTouched = true;
            continue;
        }
        if (a === '--no-tree') {
            includeTree = false;
            includeTreeTouched = true;
            continue;
        }
        if (a === '--with-tree') {
            includeTree = true;
            includeTreeTouched = true;
            continue;
        }
        if (a === '--changed') {
            changedOnly = true;
            continue;
        }
        if (a === '--changed-base' && i + 1 < argv.length) {
            changedBase = argv[++i];
            continue;
        }
        if ((a === '--runner' || a === '--mode') && i + 1 < argv.length) {
            runner = String(argv[++i] || '').trim();
            continue;
        }
        if (a === '--llvm-all') {
            llvmAll = true;
            continue;
        }
        if (a === '--assert-io') {
            assertIo = true;
            continue;
        }
        if (a === '--strict-dual') {
            strictDual = true;
            continue;
        }
        if (a === '--llvm-compile-only') {
            llvmCompileOnly = true;
            continue;
        }
        if (a === '-h' || a === '--help') {
            return {
                help: true,
                inputs,
                outPath,
                distHint,
                jobs,
                includeStdlib,
                includeTree,
                runner,
                llvmAll,
                assertIo,
                strictDual,
                llvmCompileOnly,
                changedOnly,
                changedBase,
            };
        }
    }

    if (jobs <= 0) {
        // ローカル実行では CPU を積極利用し、CI では明示 -j を優先する。
        const cpuCount = os.cpus()?.length || 4;
        jobs = Math.max(1, Math.min(32, cpuCount - 1));
    }

    if (!['wasm', 'llvm', 'all'].includes(runner)) {
        throw new Error(`--runner must be one of wasm|llvm|all, got: ${runner}`);
    }

    return {
        help: false,
        inputs,
        outPath,
        distHint,
        jobs,
        includeStdlib,
        includeStdlibTouched,
        includeTree,
        includeTreeTouched,
        runner,
        llvmAll,
        assertIo,
        strictDual,
        llvmCompileOnly,
        changedOnly,
        changedBase,
    };
}

function isDir(p) {
    try { return fs.statSync(p).isDirectory(); } catch { return false; }
}
function isFile(p) {
    try { return fs.statSync(p).isFile(); } catch { return false; }
}

function walkFiles(root) {
    const out = [];
    function rec(cur) {
        const ents = fs.readdirSync(cur, { withFileTypes: true });
        for (const e of ents) {
            const p = path.join(cur, e.name);
            if (e.isDirectory()) rec(p);
            else if (e.isFile()) out.push(p);
        }
    }
    rec(root);
    return out;
}

function collectTestsFromPath(inputPath) {
    const abs = path.resolve(inputPath);
    const cases = [];

    const files = [];
    if (isFile(abs)) {
        files.push(abs);
    } else if (isDir(abs)) {
        for (const f of walkFiles(abs)) {
            if (f.endsWith('.n.md') || f.endsWith('.nepl')) files.push(f);
        }
    } else {
        return cases;
    }

    for (const f of files) {
        const p = parseFile(f);
        for (let i = 0; i < p.doctests.length; i++) {
            const dt = p.doctests[i];
            cases.push({
                id: `${path.relative(process.cwd(), f)}::doctest#${i + 1}`,
                file: path.relative(process.cwd(), f),
                index: i + 1,
                tags: dt.tags,
                source: dt.code,
                stdin: dt.stdin ?? '',
                argv: Array.isArray(dt.argv) ? dt.argv.map((v) => String(v)) : [],
                expected_stdout: dt.stdout ?? null,
                expected_stderr: dt.stderr ?? null,
                expected_ret: Object.prototype.hasOwnProperty.call(dt, 'ret') ? dt.ret : null,
                expected_exit_code: Object.prototype.hasOwnProperty.call(dt, 'exit_code') ? dt.exit_code : null,
                expected_diag_codes: Array.isArray(dt.diag_codes) ? dt.diag_codes : [],
                expected_diag_spans: Array.isArray(dt.diag_spans) ? dt.diag_spans : [],
            });
        }
    }

    return cases;
}

function runGitNameOnly(args) {
    const p = spawnSync('git', args, {
        encoding: 'utf8',
        maxBuffer: 8 * 1024 * 1024,
    });
    if (p.status !== 0) return [];
    return String(p.stdout || '')
        .split(/\r?\n/)
        .map((v) => v.trim())
        .filter((v) => v.length > 0);
}

function collectChangedTestInputs(baseRef) {
    const files = new Set();
    const add = (arr) => {
        for (const f of arr) files.add(path.resolve(f));
    };
    add(runGitNameOnly(['diff', '--name-only', '--diff-filter=ACMR', `${baseRef}...HEAD`]));
    add(runGitNameOnly(['diff', '--name-only', '--diff-filter=ACMR']));
    add(runGitNameOnly(['diff', '--cached', '--name-only', '--diff-filter=ACMR']));
    add(runGitNameOnly(['ls-files', '--others', '--exclude-standard']));

    return Array.from(files)
        .filter((abs) => isFile(abs))
        .filter((abs) => abs.endsWith('.n.md') || abs.endsWith('.nepl'))
        .filter((abs) => {
            const rel = path.relative(process.cwd(), abs).replace(/\\/g, '/');
            return rel.startsWith('tests/')
                || rel.startsWith('stdlib/')
                || rel.startsWith('tutorials/')
                || rel.startsWith('examples/');
        });
}

async function runAllLegacy(cases, jobs, distHint) {
    const results = [];
    let idx = 0;

    async function worker(workerId) {
        let loaded = null;
        try {
            loaded = await createRunner(distHint);
        } catch (e) {
            const err = String(e?.stack || e?.message || e);
            while (true) {
                const i = idx;
                idx++;
                if (i >= cases.length) break;
                const c = cases[i];
                results.push({
                    ok: false,
                    id: c.id,
                    file: c.file,
                    index: c.index,
                    tags: c.tags,
                    status: 'error',
                    error: err,
                    worker: workerId,
                });
            }
            return;
        }

        while (true) {
            const i = idx;
            idx++;
            if (i >= cases.length) break;
            const c = cases[i];
            const req = {
                id: c.id,
                file: c.file,
                source: c.source,
                tags: c.tags,
                stdin: c.stdin || '',
                argv: Array.isArray(c.argv) ? c.argv : [],
                expected_ret: Object.prototype.hasOwnProperty.call(c, 'expected_ret') ? c.expected_ret : null,
                distHint,
            };
            const r = await runSingle(req, loaded);
            const wrapped = {
                ...r,
                id: c.id,
                file: c.file,
                index: c.index,
                tags: c.tags,
                worker: workerId,
            };
            results.push(applyDoctestExpectations(wrapped, c));
        }
    }

    const ws = [];
    for (let i = 0; i < jobs; i++) ws.push(worker(i + 1));
    await Promise.all(ws);

    // 入力順に並べたいのでソート
    results.sort((a, b) => {
        if (a.file < b.file) return -1;
        if (a.file > b.file) return 1;
        return (a.index || 0) - (b.index || 0);
    });
    return results;
}

async function runAllThreadPool(cases, jobs, distHint) {
    if (!Array.isArray(cases) || cases.length === 0) return [];

    const workerScript = path.resolve(__dirname, 'tests_wasm_worker.js');
    const workerCount = Math.max(1, Math.min(jobs, cases.length));
    const caseTimeoutMs = (() => {
        const raw = parseInt(process.env.NEPL_TEST_CASE_TIMEOUT_MS || String(DEFAULT_TEST_CASE_TIMEOUT_MS), 10);
        return Number.isFinite(raw) && raw > 0 ? raw : DEFAULT_TEST_CASE_TIMEOUT_MS;
    })();
    const results = new Array(cases.length);
    let nextIndex = 0;
    let finished = 0;
    let aborted = false;

    return await new Promise((resolve, reject) => {
        const workers = [];
        const timers = new Map();
        const running = new Map();

        function shutdownAll() {
            for (const timer of timers.values()) {
                clearTimeout(timer);
            }
            timers.clear();
            running.clear();
            for (const w of workers) {
                try { w.terminate(); } catch {}
            }
        }

        function clearWorkerTimer(w) {
            const timer = timers.get(w);
            if (timer) clearTimeout(timer);
            timers.delete(w);
        }

        function spawnWorker(slot) {
            const w = new Worker(workerScript, {
                workerData: { distHint },
            });
            workers[slot] = w;

            w.on('message', (msg) => {
                if (aborted) return;
                if (!msg || msg.kind !== 'result') return;
                clearWorkerTimer(w);
                const idx = Number(msg.index);
                running.delete(w);
                const c = cases[idx];
                const r = msg.result || {
                    ok: false,
                    status: 'error',
                    error: 'worker returned empty result',
                };
                results[idx] = {
                    ...r,
                    id: c.id,
                    file: c.file,
                    index: c.index,
                    tags: c.tags,
                    worker: slot + 1,
                };
                finished++;
                if (finished >= cases.length) {
                    shutdownAll();
                    resolve(results.filter(Boolean));
                    return;
                }
                schedule(w);
            });

            w.on('error', (err) => {
                if (aborted) return;
                aborted = true;
                clearWorkerTimer(w);
                shutdownAll();
                reject(err);
            });

            w.on('exit', (code) => {
                if (aborted) return;
                clearWorkerTimer(w);
                const active = running.get(w);
                running.delete(w);
                if (active && active.index < cases.length) {
                    const c = cases[active.index];
                    results[active.index] = {
                        ok: false,
                        id: c.id,
                        file: c.file,
                        index: c.index,
                        tags: c.tags,
                        status: 'error',
                        error: `wasm test worker exited unexpectedly: code=${code}`,
                        worker: slot + 1,
                    };
                    finished++;
                    if (finished >= cases.length) {
                        shutdownAll();
                        resolve(results.filter(Boolean));
                        return;
                    }
                    const next = spawnWorker(slot);
                    schedule(next);
                    return;
                }
                if (code !== 0 && finished < cases.length) {
                    aborted = true;
                    shutdownAll();
                    reject(new Error(`wasm test worker exited unexpectedly: code=${code}`));
                }
            });

            return w;
        }

        function schedule(w) {
            if (aborted) return;
            if (nextIndex >= cases.length) return;
            const i = nextIndex++;
            const c = cases[i];
            running.set(w, { index: i });
            w.postMessage({
                kind: 'run',
                index: i,
                req: {
                    id: c.id,
                    file: c.file,
                    source: c.source,
                    tags: c.tags,
                    stdin: c.stdin || '',
                    argv: Array.isArray(c.argv) ? c.argv : [],
                    expected_ret: Object.prototype.hasOwnProperty.call(c, 'expected_ret') ? c.expected_ret : null,
                    distHint,
                },
            });
            clearWorkerTimer(w);
            const timer = setTimeout(() => {
                if (aborted) return;
                const active = running.get(w);
                if (!active) return;
                const idx = active.index;
                const timedCase = cases[idx];
                running.delete(w);
                try { w.terminate(); } catch {}
                results[idx] = {
                    ok: false,
                    id: timedCase.id,
                    file: timedCase.file,
                    index: timedCase.index,
                    tags: timedCase.tags,
                    status: 'error',
                    error: `wasm test case timeout after ${caseTimeoutMs}ms`,
                    worker: (workers.indexOf(w) + 1) || 0,
                };
                finished++;
                if (finished >= cases.length) {
                    shutdownAll();
                    resolve(results.filter(Boolean));
                    return;
                }
                const slot = workers.indexOf(w);
                if (slot >= 0) {
                    const next = spawnWorker(slot);
                    schedule(next);
                }
            }, caseTimeoutMs);
            timers.set(w, timer);
        }

        for (let i = 0; i < workerCount; i++) {
            spawnWorker(i);
        }
        for (const w of workers) schedule(w);
    });
}

async function runAll(cases, jobs, distHint) {
    const useThreadPool = (process.env.NEPL_WASM_THREAD_POOL || '1') !== '0';
    // compiler wasm の JS stack 条件を jobs 数に依存させないため、既定では 1 job でも worker runner を使う。
    if (!useThreadPool) {
        return runAllLegacy(cases, jobs, distHint);
    }
    try {
        const results = await runAllThreadPool(cases, jobs, distHint);
        results.sort((a, b) => {
            if (a.file < b.file) return -1;
            if (a.file > b.file) return 1;
            return (a.index || 0) - (b.index || 0);
        });
        return results;
    } catch {
        return runAllLegacy(cases, jobs, distHint);
    }
}

function hasTag(tags, name) {
    return Array.isArray(tags) && tags.includes(name);
}

function normalizeOutputByTags(text, tags) {
    let out = String(text || '');
    if (hasTag(tags, 'normalize_newlines')) {
        out = out.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    }
    if (hasTag(tags, 'strip_ansi')) {
        out = stripAnsi(out);
    }
    return out;
}

function normalizePathLike(p) {
    return String(p || '').replace(/\\/g, '/');
}

function lineColFromUtf8Offset(text, offset) {
    const limit = Math.max(0, Number(offset) || 0);
    let pos = 0;
    let line = 1;
    let col = 1;
    const src = String(text || '');
    let prevWasCr = false;
    for (const ch of src) {
        if (pos >= limit) break;
        const width = Buffer.byteLength(ch, 'utf8');
        if (ch === '\n') {
            if (!prevWasCr) {
                line += 1;
                col = 1;
            }
            prevWasCr = false;
        } else if (ch === '\r') {
            line += 1;
            col = 1;
            prevWasCr = true;
        } else {
            col += 1;
            prevWasCr = false;
        }
        pos += width;
    }
    return { line, col };
}

function extractDiagSpansFromCompileError(text, testCase = null) {
    const src = stripAnsi(String(text || '')).replace(/\r\n/g, '\n');
    const out = [];
    const lines = src.split('\n');
    for (const line of lines) {
        const m = line.match(/^\s*-->\s+(.+)\s*$/);
        if (!m) continue;
        const loc = String(m[1] || '').trim();
        const lm = loc.match(/^(.*):(\d+):(\d+)$/);
        if (!lm) continue;
        out.push({
            file: String(lm[1] || '').trim(),
            line: Number(lm[2]),
            col: Number(lm[3]),
        });
    }
    const source = typeof testCase?.source === 'string' ? testCase.source : '';
    if (source) {
        const compact = src.replace(/\n/g, ' ');
        const re = /\(file=(\d+),\s*start=(\d+),\s*end=(\d+)\)/g;
        let m;
        while ((m = re.exec(compact)) !== null) {
            const fileId = Number(m[1]);
            const start = Number(m[2]);
            if (fileId !== 0 || !Number.isFinite(start)) continue;
            const loc = lineColFromUtf8Offset(source, start);
            out.push({
                file: '/virtual/entry.nepl',
                line: loc.line,
                col: loc.col,
            });
        }
    }
    return out;
}

function diagSpanMatches(expected, actual) {
    const expLine = Number(expected?.line);
    const expCol = Number(expected?.col);
    if (!Number.isFinite(expLine) || !Number.isFinite(expCol)) return false;
    if (Number(actual?.line) !== expLine || Number(actual?.col) !== expCol) return false;
    const expFile = String(expected?.file || '').trim();
    if (!expFile) return true;
    const want = normalizePathLike(expFile);
    const got = normalizePathLike(actual?.file || '');
    return got === want || got.endsWith(`/${want}`) || got.endsWith(want);
}

function applyDoctestExpectations(result, testCase, options = {}) {
    const r = { ...result };
    const tags = testCase?.tags || r.tags || [];
    const expectedDiagCodes = Array.isArray(testCase?.expected_diag_codes)
        ? testCase.expected_diag_codes.map((v) => String(v).trim()).filter(Boolean)
        : [];
    const expectedDiagSpans = Array.isArray(testCase?.expected_diag_spans)
        ? testCase.expected_diag_spans.filter((v) => v && Number.isFinite(Number(v.line)) && Number.isFinite(Number(v.col)))
        : [];
    const wantsRet = Object.prototype.hasOwnProperty.call(testCase || {}, 'expected_ret')
        && testCase.expected_ret !== null
        && testCase.expected_ret !== undefined;
    const wantsExitCode = Object.prototype.hasOwnProperty.call(testCase || {}, 'expected_exit_code')
        && testCase.expected_exit_code !== null
        && testCase.expected_exit_code !== undefined;
    const wantsStdout = testCase?.expected_stdout !== null && testCase?.expected_stdout !== undefined;
    const wantsStderr = testCase?.expected_stderr !== null && testCase?.expected_stderr !== undefined;
    const importsStdTest = /#import\s+"std\/test"\s+as\s+\*/.test(String(testCase?.source || ''));

    if (r.status !== 'pass') return r;
    if (r.phase === 'skip') return r;
    if (hasTag(tags, 'compile_fail')) {
        const compileError = String(r.compile_error || '');
        if (expectedDiagCodes.length > 0) {
            const missing = expectedDiagCodes.filter((code) => !compileError.includes(`[${code}]`));
            if (missing.length > 0) {
                r.ok = false;
                r.status = 'fail';
                r.phase = r.phase || 'compile';
                r.error = [
                    'compile_fail diagnostic code mismatch',
                    `expected codes: ${JSON.stringify(expectedDiagCodes)}`,
                    `missing codes: ${JSON.stringify(missing)}`,
                    `compile_error: ${JSON.stringify(compileError)}`,
                ].join('\n');
                return r;
            }
        }
        if (expectedDiagSpans.length > 0) {
            const actualSpans = extractDiagSpansFromCompileError(compileError, testCase);
            const missingSpans = expectedDiagSpans.filter(
                (exp) => !actualSpans.some((act) => diagSpanMatches(exp, act))
            );
            if (missingSpans.length > 0) {
                r.ok = false;
                r.status = 'fail';
                r.phase = r.phase || 'compile';
                r.error = [
                    'compile_fail diagnostic span mismatch',
                    `expected spans: ${JSON.stringify(expectedDiagSpans)}`,
                    `missing spans: ${JSON.stringify(missingSpans)}`,
                    `actual spans: ${JSON.stringify(actualSpans)}`,
                    `compile_error: ${JSON.stringify(compileError)}`,
                ].join('\n');
                return r;
            }
        }
        return r;
    }
    if (options.llvmCompileOnly && r.phase === 'compile_llvm_cli') {
        return r;
    }
    if (wantsExitCode) {
        const expected = testCase.expected_exit_code;
        if (!Object.prototype.hasOwnProperty.call(r, 'exit_code')
            || r.exit_code === null
            || r.exit_code === undefined) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = [
                'exit code result missing',
                `expected: ${JSON.stringify(expected)}`,
            ].join('\n');
            return r;
        }
        const actual = r.exit_code;
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = [
                'exit code mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }
    if (wantsRet) {
        const expected = testCase.expected_ret;
        const actual = Object.prototype.hasOwnProperty.call(r, 'return_value') ? r.return_value : null;
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = [
                'return value mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }
    if (!wantsRet && !wantsExitCode && !wantsStdout && !wantsStderr) return r;
    const strictIo = wantsStdout
        || wantsStderr
        || !!options.assertIo
        || process.env.NEPL_ASSERT_IO === '1'
        || hasTag(tags, 'assert_io');
    if (!strictIo) return r;
    if (hasTag(tags, 'should_panic')) return r;

    if (wantsStdout) {
        const expected = normalizeOutputByTags(String(testCase.expected_stdout), tags);
        const actual = normalizeOutputByTags(String(r.stdout || ''), tags);
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = [
                'stdout mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }

    if (wantsStderr) {
        const expected = normalizeOutputByTags(String(testCase.expected_stderr), tags);
        const actual = normalizeOutputByTags(String(r.stderr || ''), tags);
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = [
                'stderr mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }

    if (importsStdTest && !wantsStdout) {
        const actual = normalizeOutputByTags(String(r.stdout || ''), tags);
        if (/^FAIL:/m.test(actual)) {
            r.ok = false;
            r.status = 'fail';
            r.phase = r.phase || 'run';
            r.error = 'std/test reported FAIL output';
            return r;
        }
    }

    return r;
}

function isLlvmCase(c) {
    if (hasTag(c.tags, 'llvm_cli')) return true;
    return /^\s*#target\s+llvm\s*$/m.test(String(c.source || ''));
}

function skipOnWasmRunner(c) {
    return hasTag(c.tags, 'skip_wasm') || hasTag(c.tags, 'llvm_only');
}

function skipOnLlvmRunner(c) {
    return hasTag(c.tags, 'skip_llvm') || hasTag(c.tags, 'wasm_only') || hasTag(c.tags, 'wasi_only');
}

function llvmProgramTerminatedAbnormally(runResult) {
    if (!runResult) return true;
    if (runResult.signal) return true;
    if (runResult.code === null || runResult.code === undefined) return true;
    return Number(runResult.code) < 0;
}

function llvmReturnValueFromProcessResult(runResult) {
    if (llvmProgramTerminatedAbnormally(runResult)) return null;
    return Number(runResult.code);
}

function runCommand(cmd, args, options = {}) {
    return new Promise((resolve) => {
        const stdinText = Object.prototype.hasOwnProperty.call(options, 'stdinText')
            ? options.stdinText
            : null;
        const timeoutMs = Object.prototype.hasOwnProperty.call(options, 'timeoutMs')
            ? Number(options.timeoutMs)
            : (() => {
                const raw = parseInt(process.env.NEPL_TEST_CASE_TIMEOUT_MS || String(DEFAULT_TEST_CASE_TIMEOUT_MS), 10);
                return Number.isFinite(raw) && raw > 0 ? raw : DEFAULT_TEST_CASE_TIMEOUT_MS;
            })();
        const spawnOptions = { ...options };
        delete spawnOptions.stdinText;
        delete spawnOptions.timeoutMs;
        const child = spawn(cmd, args, {
            ...spawnOptions,
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        let stdout = '';
        let stderr = '';
        let settled = false;
        const finish = (result) => {
            if (settled) return;
            settled = true;
            clearTimeout(timer);
            resolve(result);
        };
        child.stdin.on('error', () => {
            // 子プロセスが先に終了した場合の EPIPE は無視する。
        });
        if (stdinText !== null && stdinText !== undefined) {
            child.stdin.end(String(stdinText));
        } else {
            child.stdin.end();
        }
        child.stdout.on('data', (d) => {
            stdout += d.toString();
        });
        child.stderr.on('data', (d) => {
            stderr += d.toString();
        });
        child.on('error', (err) => {
            finish({
                code: -1,
                signal: null,
                stdout,
                stderr: `${stderr}\n${String(err?.stack || err?.message || err)}`,
            });
        });
        child.on('close', (code, signal) => {
            finish({ code, signal, stdout, stderr });
        });
        const timer = setTimeout(() => {
            try { child.kill('SIGKILL'); } catch {}
            finish({
                code: -1,
                signal: 'SIGKILL',
                stdout,
                stderr: `${stderr}\ncommand timeout after ${timeoutMs}ms`.trim(),
            });
        }, timeoutMs);
    });
}

function buildLlvmRunResult(c, workerId, llPath, exePath, runWithArgs, t0) {
    const abnormal = llvmProgramTerminatedAbnormally(runWithArgs);
    const exitCode = llvmReturnValueFromProcessResult(runWithArgs);
    const base = {
        id: `${c.id}::llvm`,
        file: c.file,
        index: c.index,
        tags: c.tags,
        phase: 'run_llvm_cli',
        stdout: String(runWithArgs.stdout || ''),
        stderr: String(runWithArgs.stderr || ''),
        runtime: { code: runWithArgs.code, signal: runWithArgs.signal },
        worker: workerId,
        compiler: { runner: 'nepl-cli-llvm', ll_path: llPath, exe_path: exePath },
        duration_ms: Date.now() - t0,
    };
    if (hasTag(c.tags, 'should_panic')) {
        const ok = abnormal || (runWithArgs.code !== 0);
        return {
            ...base,
            ok,
            status: ok ? 'pass' : 'fail',
            error: ok ? null : 'expected should_panic, but llvm program finished successfully',
        };
    }
    const ok = !abnormal;
    return {
        ...base,
        ok,
        status: ok ? 'pass' : 'fail',
        return_value: exitCode,
        exit_code: exitCode,
        error: ok ? null : `llvm program terminated by signal=${runWithArgs.signal ?? 'null'}`,
    };
}

function ensureNeplCliBuilt() {
    const build = spawnSync(
        'cargo',
        ['build', '--quiet', '-p', 'nepl-cli'],
        {
            encoding: 'utf8',
            env: { ...process.env, NO_COLOR: 'true' },
            maxBuffer: 20 * 1024 * 1024,
        },
    );
    if (build.status !== 0) {
        const err = [String(build.stderr || ''), String(build.stdout || '')]
            .filter(Boolean)
            .join('\n')
            .trim();
        throw new Error(err || `failed to build nepl-cli (status=${build.status ?? 'null'})`);
    }
    const exe = process.platform === 'win32' ? 'nepl-cli.exe' : 'nepl-cli';
    const cliPath = path.resolve('target', 'debug', exe);
    if (!isFile(cliPath)) {
        throw new Error(`nepl-cli binary not found after build: ${cliPath}`);
    }
    return cliPath;
}

function preferredLlvmClangBin() {
    const envBin = process.env.NEPL_LLVM_CLANG_BIN;
    if (envBin && isFile(envBin)) return envBin;
    const candidates = [
        '/opt/llvm-21.1.0/bin/clang',
        '/usr/local/opt/llvm/bin/clang',
    ];
    for (const c of candidates) {
        if (isFile(c)) return c;
    }
    return null;
}

function stageLocalImportsForLlvmCase(c, tmpDir) {
    const importRe = /#import\s+"([^"]+)"/g;
    const stdPrefixes = ['core/', 'alloc/', 'std/', 'nm/', 'kp/', 'platform/'];
    const caseDir = path.resolve(path.dirname(String(c?.file || '.')));
    const stack = [{ srcDir: caseDir, source: String(c?.source || '') }];
    const seen = new Set();

    while (stack.length > 0) {
        const { srcDir, source } = stack.pop();
        importRe.lastIndex = 0;
        let m;
        while ((m = importRe.exec(source)) !== null) {
            const spec = String(m[1] || '').trim();
            if (!spec || path.isAbsolute(spec)) continue;
            if (stdPrefixes.some((p) => spec.startsWith(p))) continue;

            const relBase = spec.replace(/^\.\/+/, '');
            const relCandidates = relBase.endsWith('.nepl')
                ? [relBase]
                : [`${relBase}.nepl`, relBase];

            let copied = false;
            for (const rel of relCandidates) {
                const srcPath = path.resolve(srcDir, rel);
                if (!isFile(srcPath)) continue;
                const realSrc = fs.realpathSync(srcPath);
                if (!seen.has(realSrc)) {
                    seen.add(realSrc);
                    const dstPath = path.resolve(tmpDir, rel);
                    fs.mkdirSync(path.dirname(dstPath), { recursive: true });
                    fs.copyFileSync(realSrc, dstPath);
                    const nestedSource = fs.readFileSync(realSrc, 'utf8');
                    stack.push({
                        srcDir: path.dirname(realSrc),
                        source: nestedSource,
                    });
                }
                copied = true;
                break;
            }
            if (!copied) {
                continue;
            }
        }
    }
}

async function runSingleLlvmCli(c, workerId, cliPath, options = {}) {
    const t0 = Date.now();
    if (hasTag(c.tags, 'skip')) {
        return {
            ok: true,
            id: `${c.id}::llvm`,
            file: c.file,
            index: c.index,
            tags: c.tags,
            status: 'pass',
            phase: 'skip',
            skipped: true,
            worker: workerId,
            compiler: { runner: 'nepl-cli-llvm' },
            duration_ms: Date.now() - t0,
        };
    }

    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-llvm-cli-'));
    const entryPath = path.join(tmpDir, 'entry.nepl');
    const outputBase = path.join(tmpDir, 'out');
    const llPath = `${outputBase}.ll`;
    const exePath = process.platform === 'win32' ? `${outputBase}.exe` : `${outputBase}.bin`;
    fs.writeFileSync(entryPath, c.source, 'utf8');
    stageLocalImportsForLlvmCase(c, tmpDir);

    const clangBin = preferredLlvmClangBin();
    const child = await runCommand(
        cliPath,
        [
            '--target',
            'llvm',
            '--profile',
            'debug',
            '--input',
            entryPath,
            '--output',
            outputBase,
        ],
        {
            env: {
                ...process.env,
                NO_COLOR: 'true',
                ...(clangBin ? { NEPL_LLVM_CLANG_BIN: clangBin } : {}),
            },
        },
    );

    let result;
    const stderr = String(child.stderr || '');
    const stdout = String(child.stdout || '');
    const compileError = [stderr, stdout].filter(Boolean).join('\n').trim() || null;

    const hasCompileFailTag = hasTag(c.tags, 'compile_fail');
    if (hasCompileFailTag) {
        const ok = child.code !== 0;
        result = {
            ok,
            id: `${c.id}::llvm`,
            file: c.file,
            index: c.index,
            tags: c.tags,
            status: ok ? 'pass' : 'fail',
            phase: 'compile_llvm_cli',
            compile_error: compileError,
            error: ok ? null : 'expected compile_fail, but llvm compilation succeeded',
            worker: workerId,
            compiler: { runner: 'nepl-cli-llvm' },
            duration_ms: Date.now() - t0,
        };
    } else {
        const compileOk = child.code === 0 && isFile(llPath);
        if (!compileOk) {
            result = {
                ok: false,
                id: `${c.id}::llvm`,
                file: c.file,
                index: c.index,
                tags: c.tags,
                status: 'fail',
                phase: 'compile_llvm_cli',
                error: compileError || `llvm compilation failed (status=${child.code ?? 'null'})`,
                worker: workerId,
                compiler: { runner: 'nepl-cli-llvm', ll_path: llPath },
                duration_ms: Date.now() - t0,
            };
        } else if (options.llvmCompileOnly) {
            result = {
                ok: true,
                id: `${c.id}::llvm`,
                file: c.file,
                index: c.index,
                tags: c.tags,
                status: 'pass',
                phase: 'compile_llvm_cli',
                error: null,
                worker: workerId,
                compiler: { runner: 'nepl-cli-llvm', ll_path: llPath },
                duration_ms: Date.now() - t0,
            };
        } else {
            const link = await runCommand(
                clangBin || 'clang',
                ['-O0', llPath, '-o', exePath, '-lm'],
                {
                    env: {
                        ...process.env,
                        NO_COLOR: 'true',
                    },
                },
            );
            const linkErr = [String(link.stderr || ''), String(link.stdout || '')]
                .filter(Boolean)
                .join('\n')
                .trim();
            if (link.code !== 0 || !isFile(exePath)) {
                result = {
                    ok: false,
                    id: `${c.id}::llvm`,
                    file: c.file,
                    index: c.index,
                    tags: c.tags,
                    status: 'fail',
                    phase: 'link_llvm_cli',
                    error: linkErr || `clang link failed (status=${link.code ?? 'null'})`,
                    worker: workerId,
                    compiler: { runner: 'nepl-cli-llvm', ll_path: llPath },
                    duration_ms: Date.now() - t0,
                };
            } else {
                const runArgs = Array.isArray(c.argv) ? c.argv.map((v) => String(v)) : [];
                const runWithArgs = await runCommand(exePath, runArgs, {
                    env: { ...process.env, NO_COLOR: 'true' },
                    stdinText: c.stdin || '',
                });
                result = buildLlvmRunResult(c, workerId, llPath, exePath, runWithArgs, t0);
            }
        }
    }

    try {
        fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {
        // noop
    }
    return result;
}

async function runAllLlvm(cases, jobs, options = {}) {
    const cliPath = ensureNeplCliBuilt();
    const results = [];
    let idx = 0;
    const onResult = typeof options.onResult === 'function' ? options.onResult : null;

    async function worker(workerId) {
        while (true) {
            const i = idx;
            idx++;
            if (i >= cases.length) break;
            const c = cases[i];
            const r = await runSingleLlvmCli(c, workerId, cliPath, options);
            const checked = applyDoctestExpectations(r, c, options);
            results.push(checked);
            if (onResult) onResult(checked);
        }
    }

    const cpuCount = os.cpus()?.length || 4;
    const requested = Number(process.env.NEPL_LLVM_TEST_JOBS || jobs || 2) || 2;
    const workerCount = Math.max(1, Math.min(32, cpuCount, requested));
    const ws = [];
    for (let i = 0; i < workerCount; i++) ws.push(worker(i + 1));
    await Promise.all(ws);

    results.sort((a, b) => {
        if (a.file < b.file) return -1;
        if (a.file > b.file) return 1;
        return (a.index || 0) - (b.index || 0);
    });
    return results;
}

function summarize(results) {
    let passed = 0;
    let failed = 0;
    let errored = 0;
    for (const r of results) {
        if (r.status === 'pass') passed++;
        else if (r.status === 'fail') failed++;
        else errored++;
    }
    return {
        total: results.length,
        passed,
        failed,
        errored,
    };
}

function stripLlvmSuffix(id) {
    const s = String(id || '');
    return s.endsWith('::llvm') ? s.slice(0, -6) : s;
}

function compareWasmLlvmResults(wasmResults, llvmResults, options = {}) {
    const llvmMap = new Map();
    for (const r of llvmResults) {
        llvmMap.set(stripLlvmSuffix(r.id), r);
    }
    const compared = [];
    for (const w of wasmResults) {
        const k = String(w.id || '');
        if (skipOnLlvmRunner(w)) continue;
        const l = llvmMap.get(k);
        if (!l) {
            if (options.strictDual) {
                compared.push({
                    ok: false,
                    id: `${k}::compare_wasm_llvm`,
                    file: w.file,
                    index: w.index,
                    tags: [...(w.tags || []), 'compare_wasm_llvm'],
                    status: 'fail',
                    phase: 'compare_wasm_llvm',
                    error: 'missing llvm counterpart result',
                    worker: 0,
                });
            }
            continue;
        }
        if (w.status !== 'pass' || l.status !== 'pass') continue;
        if (w.phase !== 'run' || l.phase !== 'run_llvm_cli') continue;
        if (hasTag(w.tags, 'compile_fail') || hasTag(w.tags, 'should_panic')) continue;

        const waOut = normalizeOutputByTags(String(w.stdout || ''), w.tags || []);
        const llOut = normalizeOutputByTags(String(l.stdout || ''), l.tags || []);
        const waErr = normalizeOutputByTags(String(w.stderr || ''), w.tags || []);
        const llErr = normalizeOutputByTags(String(l.stderr || ''), l.tags || []);
        const ok = waOut === llOut && waErr === llErr;

        compared.push({
            ok,
            id: `${k}::compare_wasm_llvm`,
            file: w.file,
            index: w.index,
            tags: [...(w.tags || []), 'compare_wasm_llvm'],
            status: ok ? 'pass' : 'fail',
            phase: 'compare_wasm_llvm',
            error: ok
                ? null
                : [
                    'wasm/llvm output mismatch',
                    `wasm.stdout=${JSON.stringify(waOut)}`,
                    `llvm.stdout=${JSON.stringify(llOut)}`,
                    `wasm.stderr=${JSON.stringify(waErr)}`,
                    `llvm.stderr=${JSON.stringify(llErr)}`,
                ].join('\n'),
            worker: 0,
        });
    }
    return compared;
}

function buildCaseMap(cases) {
    const map = new Map();
    for (const c of cases) map.set(c.id, c);
    return map;
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

function pickTopIssues(results, limit) {
    const failed = results.filter(r => r.status === 'fail');
    const errored = results.filter(r => r.status === 'error');
    const ordered = [...errored, ...failed];
    return ordered.slice(0, limit).map((r) => ({
        id: r.id,
        status: r.status,
        phase: r.phase || null,
        error: summarizeError(r.error),
    }));
}

function stripAnsi(s) {
    return String(s || '').replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '');
}

function summarizeError(raw) {
    if (!raw) return null;
    const lines = stripAnsi(raw)
        .replace(/\r\n/g, '\n')
        .split('\n')
        .map((l) => l.trim())
        .filter((l) => l.length > 0);
    if (lines.length === 0) return null;
    const explicit = lines.find((l) => /^Error:\s+/i.test(l));
    if (explicit) return explicit.slice(0, 240);
    const fatal = lines.find((l) => /\berror\b/i.test(l));
    if (fatal) return fatal.slice(0, 240);
    return lines.slice(0, 3).join(' | ').slice(0, 240);
}

function collectResolvedDistDirs(results) {
    const dirs = new Set();
    for (const r of results) {
        const d = r?.compiler?.distDir;
        if (typeof d === 'string' && d.length > 0) dirs.add(d);
    }
    return Array.from(dirs).sort();
}

function progressFlushEvery() {
    const raw = parseInt(process.env.NEPL_TEST_PROGRESS_FLUSH_EVERY || '10', 10);
    return Number.isFinite(raw) && raw > 0 ? raw : 10;
}

function printStdlibBuildMessage() {
    console.log('[nodesrc/tests] 固定メッセージ: stdlib のみの修正時は trunk build 不要です。');
    console.log('[nodesrc/tests] 固定メッセージ: Rust 側（nepl-core/nepl-web/nepl-cli）を修正した場合は trunk build 後に tests を実行してください。');
}

async function main() {
    const {
        help,
        inputs,
        outPath,
        distHint,
        jobs,
        includeStdlib,
        includeStdlibTouched,
        includeTree,
        includeTreeTouched,
        runner,
        llvmAll,
        assertIo,
        strictDual,
        llvmCompileOnly,
        changedOnly,
        changedBase,
    } = parseArgs(process.argv.slice(2));
    printStdlibBuildMessage();
    if (help || (!changedOnly && inputs.length === 0) || !outPath) {
        console.log('Usage: node nodesrc/tests.js -i <dir_or_file> [-i ...] -o <out.json> [--changed] [--changed-base <gitRef>] [--dist <distDirHint>] [-j N] [--runner wasm|llvm|all] [--llvm-all] [--assert-io] [--strict-dual] [--llvm-compile-only] [--no-stdlib|--with-stdlib] [--no-tree|--with-tree]');
        process.exit(help ? 0 : 2);
    }

    const effectiveInputs = inputs.slice();
    let effectiveIncludeStdlib = includeStdlib;
    let effectiveIncludeTree = includeTree;
    if (changedOnly) {
        const changedInputs = collectChangedTestInputs(changedBase);
        for (const c of changedInputs) {
            if (!effectiveInputs.includes(c)) effectiveInputs.push(c);
        }
        if (!includeStdlibTouched) effectiveIncludeStdlib = false;
        if (!includeTreeTouched) effectiveIncludeTree = false;
    }

    if (effectiveInputs.length === 0) {
        const out = {
            schema: 'neplg2-doctest/v1',
            generated_at: new Date().toISOString(),
            jobs,
            runner,
            llvm_all: llvmAll,
            assert_io: assertIo,
            strict_dual: strictDual,
            llvm_compile_only: llvmCompileOnly,
            dist_hint: distHint || null,
            resolved_dist_dirs: [],
            summary: { total: 0, passed: 0, failed: 0, errored: 0 },
            results: [],
            scan: {
                changed: changedOnly,
                changed_base: changedBase,
                inputs: [],
                include_stdlib: effectiveIncludeStdlib,
                include_tree: effectiveIncludeTree,
            },
        };
        const outAbs = path.resolve(outPath);
        ensureDir(path.dirname(outAbs));
        fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));
        console.log(JSON.stringify({
            dist: { hint: distHint || null, resolved: [] },
            summary: out.summary,
            scan: out.scan,
            top_issues: [],
        }, null, 2));
        return;
    }

    const allCases = [];
    const scanInputs = effectiveInputs.slice();
    if (effectiveIncludeStdlib && !scanInputs.some((p) => path.resolve(p) === path.resolve('stdlib'))) {
        scanInputs.push('stdlib');
    }
    for (const p of scanInputs) {
        allCases.push(...collectTestsFromPath(p));
    }

    const wasmCases = allCases.filter((c) => !isLlvmCase(c) && !skipOnWasmRunner(c));
    const llvmCasesRaw = (llvmAll ? allCases : allCases.filter((c) => isLlvmCase(c)))
        .filter((c) => !skipOnLlvmRunner(c));
    const llvmCases = llvmCasesRaw.map((c) => ({
        ...c,
        _llvmExplicit: isLlvmCase(c),
    }));

    const runnableCaseCount = runner === 'wasm'
        ? wasmCases.length
        : runner === 'llvm'
            ? llvmCases.length
            : wasmCases.length + llvmCases.length;
    if (runnableCaseCount === 0 && !changedOnly) {
        const scanInputLabels = scanInputs.map((p) => path.relative(process.cwd(), path.resolve(p)));
        const error = `no runnable doctests collected for inputs: ${scanInputLabels.join(', ') || '(none)'}`;
        const results = [{
            ok: false,
            id: 'nodesrc/tests/no-runnable-doctests',
            file: 'nodesrc/tests.js',
            index: 0,
            tags: ['scan'],
            status: 'error',
            phase: 'scan',
            error,
            detail: {
                runner,
                all_cases: allCases.length,
                wasm_cases: wasmCases.length,
                llvm_cases: llvmCases.length,
            },
            worker: 0,
        }];
        const summary = summarize(results);
        const out = {
            schema: 'neplg2-doctest/v1',
            generated_at: new Date().toISOString(),
            jobs,
            runner,
            llvm_all: llvmAll,
            assert_io: assertIo,
            strict_dual: strictDual,
            llvm_compile_only: llvmCompileOnly,
            dist_hint: distHint || null,
            resolved_dist_dirs: [],
            scan: {
                changed: changedOnly,
                changed_base: changedBase,
                inputs: scanInputLabels,
                include_stdlib: effectiveIncludeStdlib,
                include_tree: effectiveIncludeTree,
            },
            summary,
            results,
        };
        const outAbs = path.resolve(outPath);
        ensureDir(path.dirname(outAbs));
        fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));
        const topIssues = pickTopIssues(results, 5);
        console.log(JSON.stringify({
            dist: { hint: distHint || null, resolved: [] },
            summary,
            scan: out.scan,
            top_issues: topIssues,
        }, null, 2));
        process.exitCode = 1;
        return;
    }

    const outAbs = path.resolve(outPath);
    const partialResults = [];
    const expectedProgressResults = runner === 'wasm'
        ? wasmCases.length
        : runner === 'llvm'
            ? llvmCases.length
            : wasmCases.length + llvmCases.length;
    const flushEvery = progressFlushEvery();
    const scanSummary = {
        changed: changedOnly,
        changed_base: changedBase,
        inputs: scanInputs.map((p) => path.relative(process.cwd(), path.resolve(p))),
        include_stdlib: effectiveIncludeStdlib,
        include_tree: effectiveIncludeTree,
    };
    const writePartialOutput = (reason) => {
        ensureDir(path.dirname(outAbs));
        const last = partialResults.length > 0 ? partialResults[partialResults.length - 1] : null;
        const out = {
            schema: 'neplg2-doctest/v1',
            generated_at: new Date().toISOString(),
            partial: true,
            partial_reason: reason,
            jobs,
            runner,
            llvm_all: llvmAll,
            assert_io: assertIo,
            strict_dual: strictDual,
            llvm_compile_only: llvmCompileOnly,
            dist_hint: distHint || null,
            resolved_dist_dirs: collectResolvedDistDirs(partialResults),
            scan: scanSummary,
            progress: {
                completed_results: partialResults.length,
                expected_results: expectedProgressResults,
                last_result_id: last?.id || null,
                last_result_status: last?.status || null,
                last_result_phase: last?.phase || null,
            },
            summary: summarize(partialResults),
            results: partialResults,
        };
        fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));
    };
    const recordProgress = (result) => {
        partialResults.push(result);
        if (partialResults.length % flushEvery === 0) {
            writePartialOutput('progress');
        }
    };
    writePartialOutput('started');

    let results = [];
    if (runner === 'wasm') {
        results = await runAll(wasmCases, jobs, distHint);
        const wasmCaseMap = buildCaseMap(wasmCases);
        results = results.map((r) => {
            const c = wasmCaseMap.get(r.id);
            return c ? applyDoctestExpectations(r, c, { assertIo }) : r;
        });
    } else if (runner === 'llvm') {
        results = await runAllLlvm(llvmCases, jobs, { assertIo, llvmCompileOnly, onResult: recordProgress });
    } else {
        const [wasmResults, llvmResults] = await Promise.all([
            runAll(wasmCases, jobs, distHint),
            runAllLlvm(llvmCases, jobs, { assertIo, llvmCompileOnly, onResult: recordProgress }),
        ]);
        const wasmCaseMap = buildCaseMap(wasmCases);
        const checkedWasm = wasmResults.map((r) => {
            const c = wasmCaseMap.get(r.id);
            return c ? applyDoctestExpectations(r, c, { assertIo }) : r;
        });
        const compared = compareWasmLlvmResults(checkedWasm, llvmResults, { strictDual });
        results = [...checkedWasm, ...llvmResults, ...compared];
    }

    if (effectiveIncludeTree && runner !== 'llvm') {
        try {
            const tree = await runTreeSuite(distHint || '');
            const treeResults = Array.isArray(tree?.results) ? tree.results : [];
            for (const tr of treeResults) {
                const status = tr?.status === 'pass' ? 'pass' : tr?.status === 'fail' ? 'fail' : 'error';
                results.push({
                    ok: status === 'pass',
                    id: `tests/compiler/tree/${tr?.id || 'unknown'}`,
                    file: 'tests/compiler/tree',
                    index: 0,
                    tags: ['tree_api'],
                    status,
                    phase: 'analysis',
                    error: tr?.error || null,
                    detail: tr?.detail || null,
                    worker: 0,
                });
            }
        } catch (e) {
            results.push({
                ok: false,
                id: 'tests/compiler/tree/run',
                file: 'tests/compiler/tree',
                index: 0,
                tags: ['tree_api'],
                status: 'error',
                phase: 'analysis',
                error: String(e?.stack || e?.message || e),
                worker: 0,
            });
        }
    }
    const summary = summarize(results);
    const resolvedDistDirs = collectResolvedDistDirs(results);

    const out = {
        schema: 'neplg2-doctest/v1',
        generated_at: new Date().toISOString(),
        partial: false,
        jobs,
        runner,
        llvm_all: llvmAll,
        assert_io: assertIo,
        strict_dual: strictDual,
        llvm_compile_only: llvmCompileOnly,
        dist_hint: distHint || null,
        resolved_dist_dirs: resolvedDistDirs,
        scan: scanSummary,
        summary,
        results,
    };

    ensureDir(path.dirname(outAbs));
    fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));

    const topIssues = pickTopIssues(results, 5);
    console.log(JSON.stringify({
        dist: {
            hint: distHint || null,
            resolved: resolvedDistDirs,
        },
        summary,
        scan: out.scan,
        top_issues: topIssues,
    }, null, 2));

    // 失敗があれば exit code を 1 にする（gh-pages では continue-on-error で許可する想定）
    if (summary.failed > 0 || summary.errored > 0) {
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main().catch((e) => {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    });
}

module.exports = {
    applyDoctestExpectations,
    buildLlvmRunResult,
    extractDiagSpansFromCompileError,
    llvmReturnValueFromProcessResult,
};

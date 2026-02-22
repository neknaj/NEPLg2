#!/usr/bin/env node
// nodesrc/tests.js
// 目的:
// - /tests/*.n.md, /tutorials/**/*.n.md, /stdlib/**/*.nepl などに埋め込まれた doctest を走査して実行し、結果を JSON にまとめる。
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
const { parseFile } = require('./parser');
const { createRunner, runSingle } = require('./run_test');
const { runTreeSuite } = require('../tests/tree/run');

// doctest 集計の標準出力は要約重視にする。
process.removeAllListeners('warning');
process.on('warning', () => {});

function parseArgs(argv) {
    const inputs = [];
    let outPath = '';
    let distHint = '';
    let jobs = 0;
    let includeStdlib = true;
    let includeTree = true;
    let runner = 'wasm';
    let llvmAll = false;
    let assertIo = false;
    let strictDual = false;
    let llvmCompileOnly = false;

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
            continue;
        }
        if (a === '--no-tree') {
            includeTree = false;
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
            };
        }
    }

    if (jobs <= 0) {
        // GH Actions でも暴れない程度
        jobs = Math.max(1, Math.min(8, Math.floor((os.cpus()?.length || 4) / 2)));
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
        includeTree,
        runner,
        llvmAll,
        assertIo,
        strictDual,
        llvmCompileOnly,
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
                expected_stdout: dt.stdout ?? null,
                expected_stderr: dt.stderr ?? null,
            });
        }
    }

    return cases;
}

async function runAll(cases, jobs, distHint) {
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

function applyDoctestExpectations(result, testCase, options = {}) {
    const r = { ...result };
    const tags = testCase?.tags || r.tags || [];
    const strictIo = !!options.assertIo || process.env.NEPL_ASSERT_IO === '1' || hasTag(tags, 'assert_io');
    if (!strictIo) return r;
    const wantsStdout = testCase?.expected_stdout !== null && testCase?.expected_stdout !== undefined;
    const wantsStderr = testCase?.expected_stderr !== null && testCase?.expected_stderr !== undefined;

    if (r.status !== 'pass') return r;
    if (r.phase === 'skip') return r;
    if (hasTag(tags, 'compile_fail') || hasTag(tags, 'should_panic')) return r;
    if (!wantsStdout && !wantsStderr) return r;

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

    return r;
}

function isLlvmCase(c) {
    if (hasTag(c.tags, 'llvm_cli')) return true;
    return /^\s*#target\s+llvm\s*$/m.test(String(c.source || ''));
}

function runCommand(cmd, args, options = {}) {
    return new Promise((resolve) => {
        const stdinText = Object.prototype.hasOwnProperty.call(options, 'stdinText')
            ? options.stdinText
            : null;
        const spawnOptions = { ...options };
        delete spawnOptions.stdinText;
        const child = spawn(cmd, args, {
            ...spawnOptions,
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        let stdout = '';
        let stderr = '';
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
            resolve({
                code: -1,
                signal: null,
                stdout,
                stderr: `${stderr}\n${String(err?.stack || err?.message || err)}`,
            });
        });
        child.on('close', (code, signal) => {
            resolve({ code, signal, stdout, stderr });
        });
    });
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
                ['-O0', llPath, '-o', exePath],
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
                const run = await runCommand(exePath, [], {
                    env: { ...process.env, NO_COLOR: 'true' },
                    stdinText: c.stdin || '',
                });
                const signaled = !!run.signal;
                const abnormal = signaled;
                if (hasTag(c.tags, 'should_panic')) {
                    const ok = abnormal || (run.code !== 0);
                    result = {
                        ok,
                        id: `${c.id}::llvm`,
                        file: c.file,
                        index: c.index,
                        tags: c.tags,
                        status: ok ? 'pass' : 'fail',
                        phase: 'run_llvm_cli',
                        stdout: String(run.stdout || ''),
                        stderr: String(run.stderr || ''),
                        error: ok ? null : 'expected should_panic, but llvm program finished successfully',
                        runtime: { code: run.code, signal: run.signal },
                        worker: workerId,
                        compiler: { runner: 'nepl-cli-llvm', ll_path: llPath, exe_path: exePath },
                        duration_ms: Date.now() - t0,
                    };
                } else {
                    const ok = !abnormal;
                    result = {
                        ok,
                        id: `${c.id}::llvm`,
                        file: c.file,
                        index: c.index,
                        tags: c.tags,
                        status: ok ? 'pass' : 'fail',
                        phase: 'run_llvm_cli',
                        stdout: String(run.stdout || ''),
                        stderr: String(run.stderr || ''),
                        error: ok ? null : `llvm program terminated by signal=${run.signal ?? 'null'}`,
                        runtime: { code: run.code, signal: run.signal },
                        worker: workerId,
                        compiler: { runner: 'nepl-cli-llvm', ll_path: llPath, exe_path: exePath },
                        duration_ms: Date.now() - t0,
                    };
                }
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

    async function worker(workerId) {
        while (true) {
            const i = idx;
            idx++;
            if (i >= cases.length) break;
            const c = cases[i];
            const r = await runSingleLlvmCli(c, workerId, cliPath, options);
            results.push(applyDoctestExpectations(r, c, options));
        }
    }

    const workerCount = Math.max(
        1,
        Math.min(4, Number(process.env.NEPL_LLVM_TEST_JOBS || jobs || 2) || 2),
    );
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
    return s.endsWith('::llvm') ? s.slice(0, -7) : s;
}

function compareWasmLlvmResults(wasmResults, llvmResults, options = {}) {
    const llvmMap = new Map();
    for (const r of llvmResults) {
        llvmMap.set(stripLlvmSuffix(r.id), r);
    }
    const compared = [];
    for (const w of wasmResults) {
        const k = String(w.id || '');
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
    return String(s || '').replace(/\x1b\[[0-9;]*m/g, '');
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

async function main() {
    const { help, inputs, outPath, distHint, jobs, includeStdlib, includeTree, runner, llvmAll, assertIo, strictDual, llvmCompileOnly } = parseArgs(process.argv.slice(2));
    if (help || inputs.length === 0 || !outPath) {
        console.log('Usage: node nodesrc/tests.js -i <dir_or_file> [-i ...] -o <out.json> [--dist <distDirHint>] [-j N] [--runner wasm|llvm|all] [--llvm-all] [--assert-io] [--strict-dual] [--llvm-compile-only] [--no-stdlib] [--no-tree]');
        process.exit(help ? 0 : 2);
    }

    const allCases = [];
    const scanInputs = inputs.slice();
    if (includeStdlib && !scanInputs.some((p) => path.resolve(p) === path.resolve('stdlib'))) {
        scanInputs.push('stdlib');
    }
    for (const p of scanInputs) {
        allCases.push(...collectTestsFromPath(p));
    }

    const wasmCases = allCases.filter((c) => !isLlvmCase(c));
    const llvmCasesRaw = llvmAll ? allCases : allCases.filter((c) => isLlvmCase(c));
    const llvmCases = llvmCasesRaw.map((c) => ({
        ...c,
        _llvmExplicit: isLlvmCase(c),
    }));

    let results = [];
    if (runner === 'wasm') {
        results = await runAll(wasmCases, jobs, distHint);
        const wasmCaseMap = buildCaseMap(wasmCases);
        results = results.map((r) => {
            const c = wasmCaseMap.get(r.id);
            return c ? applyDoctestExpectations(r, c, { assertIo }) : r;
        });
    } else if (runner === 'llvm') {
        results = await runAllLlvm(llvmCases, jobs, { assertIo, llvmCompileOnly });
    } else {
        const wasmResults = await runAll(wasmCases, jobs, distHint);
        const wasmCaseMap = buildCaseMap(wasmCases);
        const checkedWasm = wasmResults.map((r) => {
            const c = wasmCaseMap.get(r.id);
            return c ? applyDoctestExpectations(r, c, { assertIo }) : r;
        });
        const llvmResults = await runAllLlvm(llvmCases, jobs, { assertIo, llvmCompileOnly });
        const compared = compareWasmLlvmResults(checkedWasm, llvmResults, { strictDual });
        results = [...checkedWasm, ...llvmResults, ...compared];
    }

    if (includeTree && runner !== 'llvm') {
        try {
            const tree = await runTreeSuite(distHint || '');
            const treeResults = Array.isArray(tree?.results) ? tree.results : [];
            for (const tr of treeResults) {
                const status = tr?.status === 'pass' ? 'pass' : tr?.status === 'fail' ? 'fail' : 'error';
                results.push({
                    ok: status === 'pass',
                    id: `tests/tree/${tr?.id || 'unknown'}`,
                    file: 'tests/tree',
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
                id: 'tests/tree/run',
                file: 'tests/tree',
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
        jobs,
        runner,
        llvm_all: llvmAll,
        assert_io: assertIo,
        strict_dual: strictDual,
        llvm_compile_only: llvmCompileOnly,
        dist_hint: distHint || null,
        resolved_dist_dirs: resolvedDistDirs,
        summary,
        results,
    };

    const outAbs = path.resolve(outPath);
    ensureDir(path.dirname(outAbs));
    fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));

    const topIssues = pickTopIssues(results, 5);
    console.log(JSON.stringify({
        dist: {
            hint: distHint || null,
            resolved: resolvedDistDirs,
        },
        summary,
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

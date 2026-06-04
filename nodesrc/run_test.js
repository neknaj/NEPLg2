#!/usr/bin/env node
// nodesrc/run_test.js
// 目的: doctest 1 件を「コンパイル -> WASI 実行」して結果を返す。
//
// 入力:
// - JSON (stdin)
//   { "id": "...", "source": "...", "tags": [..], "stdin": "...", "distHint": "..." }
// 出力:
// - JSON (stdout)
//   { ok, id, status, stdout, stderr, error, compiler, runtime, timing, duration_ms }

const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn } = require('node:child_process');
const { WASI } = require('node:wasi');
const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');
const { wasmerRunMountArgs } = require('./wasmer_args');
const {
    loadStdlibVfsFromFs,
    stdlibOverrideIsNewerThanArtifact,
} = require('./stdlib_vfs_cache');

function readStdinAll() {
    return new Promise((resolve) => {
        const chunks = [];
        process.stdin.on('data', (c) => chunks.push(c));
        process.stdin.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
        process.stdin.resume();
    });
}

function writeJson(obj) {
    process.stdout.write(JSON.stringify(obj, (_key, value) => {
        if (typeof value === 'bigint') return value.toString();
        return value;
    }));
}

function mkTmpPath(prefix) {
    return path.join(os.tmpdir(), `${prefix}-${process.pid}-${Math.random().toString(16).slice(2)}`);
}

function safeUnlink(p) {
    try { fs.unlinkSync(p); } catch {}
}

function ensureWasiScratchDir(preopenRoot) {
    const scratchDir = path.join(preopenRoot, 'tmp');
    fs.mkdirSync(scratchDir, { recursive: true });
    return scratchDir;
}

function formatError(e) {
    if (!e) return 'unknown error';
    const name = typeof e.name === 'string' && e.name.length > 0 ? e.name : null;
    const message = typeof e.message === 'string' && e.message.length > 0 ? e.message : String(e);
    const stack = typeof e.stack === 'string' && e.stack.length > 0 ? e.stack : null;
    if (stack) return stack;
    if (name && message) return `${name}: ${message}`;
    return message;
}

function decodeExpectedReturn(expectedRet, rawValue, memory) {
    if (expectedRet === null || expectedRet === undefined) return rawValue;
    if (typeof expectedRet === 'string') {
        if (!memory || !Number.isFinite(rawValue)) return null;
        const addr = Number(rawValue) | 0;
        const view = new DataView(memory.buffer);
        if (addr < 0 || addr + 4 > view.byteLength) return null;
        const len = view.getInt32(addr, true);
        if (len < 0 || addr + 4 + len > view.byteLength) return null;
        const bytes = new Uint8Array(memory.buffer, addr + 4, len);
        return new TextDecoder('utf-8').decode(bytes);
    }
    if (typeof rawValue === 'bigint') {
        const minSafe = BigInt(Number.MIN_SAFE_INTEGER);
        const maxSafe = BigInt(Number.MAX_SAFE_INTEGER);
        if (rawValue >= minSafe && rawValue <= maxSafe) {
            return Number(rawValue);
        }
        return rawValue.toString();
    }
    return rawValue;
}

function decodeExitCode(rawValue) {
    if (typeof rawValue === 'bigint') {
        const minSafe = BigInt(Number.MIN_SAFE_INTEGER);
        const maxSafe = BigInt(Number.MAX_SAFE_INTEGER);
        if (rawValue >= minSafe && rawValue <= maxSafe) return Number(rawValue);
        return null;
    }
    if (typeof rawValue === 'number' && Number.isFinite(rawValue)) return rawValue;
    return null;
}

function runWasiBytes(wasmBytes, stdinText, argv = []) {
    return {
        ...runWasiBytesWithImports(wasmBytes, stdinText, argv),
        runner: 'node-wasi',
    };
}

function runWasiBytesWithImports(wasmBytes, stdinText, argv = [], extraImports = {}) {
    const wasmPath = mkTmpPath('nepl-doctest') + '.wasm';
    const stdinPath = mkTmpPath('wasi-stdin');
    const stdoutPath = mkTmpPath('wasi-stdout');
    const stderrPath = mkTmpPath('wasi-stderr');
    const preopenRoot = process.cwd();
    ensureWasiScratchDir(preopenRoot);

    fs.writeFileSync(wasmPath, Buffer.from(wasmBytes));
    fs.writeFileSync(stdinPath, Buffer.from(stdinText || '', 'utf-8'));
    fs.writeFileSync(stdoutPath, Buffer.alloc(0));
    fs.writeFileSync(stderrPath, Buffer.alloc(0));

    const stdinFd = fs.openSync(stdinPath, 'r');
    const stdoutFd = fs.openSync(stdoutPath, 'w+');
    const stderrFd = fs.openSync(stderrPath, 'w+');

    const wasi = new WASI({
        version: 'preview1',
        args: [wasmPath, ...(Array.isArray(argv) ? argv.map((v) => String(v)) : [])],
        env: {},
        preopens: {
            '/': preopenRoot,
        },
        stdin: stdinFd,
        stdout: stdoutFd,
        stderr: stderrFd,
    });

    let trapped = false;
    let trapError = null;
    let returnValue = null;
    let memory = null;
    try {
        const module = new WebAssembly.Module(Buffer.from(wasmBytes));
        const instance = new WebAssembly.Instance(module, {
            wasi_snapshot_preview1: wasi.wasiImport,
            nepl_gui_web: {
                poll_action_id: () => 0,
                wait_action_id: () => 0,
                poll_event_kind: () => 0,
                wait_event_kind: () => 0,
                last_event_window_id: () => 0,
                last_event_action_id: () => 0,
                last_event_point_x_milli: () => 0,
                last_event_point_y_milli: () => 0,
                last_event_pointer_kind: () => 0,
                last_event_pointer_id: () => 0,
                last_event_pointer_button: () => 0,
                last_event_keyboard_kind: () => 0,
                last_event_key_code: () => 0,
                last_event_key_modifiers: () => 0,
                last_event_text_scalar_value: () => 0,
                last_event_window_kind: () => 0,
                last_event_window_width: () => 0,
                last_event_window_height: () => 0,
            },
            ...extraImports,
        });
        memory = instance.exports.memory || null;
        if (typeof instance.exports.main === 'function') {
            if (typeof wasi.initialize === 'function' && instance.exports.memory) {
                const initExports = { memory: instance.exports.memory };
                if (typeof instance.exports._initialize === 'function') {
                    initExports._initialize = instance.exports._initialize;
                }
                wasi.initialize({ exports: initExports });
            }
            returnValue = instance.exports.main();
        } else {
            returnValue = wasi.start(instance);
        }
    } catch (e) {
        trapped = true;
        trapError = e;
    }

    fs.closeSync(stdinFd);
    fs.closeSync(stdoutFd);
    fs.closeSync(stderrFd);

    const out = fs.readFileSync(stdoutPath).toString('utf-8');
    const err = fs.readFileSync(stderrPath).toString('utf-8');

    safeUnlink(wasmPath);
    safeUnlink(stdinPath);
    safeUnlink(stdoutPath);
    safeUnlink(stderrPath);

    return {
        trapped,
        trapError: trapError ? formatError(trapError) : null,
        stdout: out,
        stderr: err,
        returnValue,
        memory,
    };
}

function runWasixBytesWithTtyFallback(wasmBytes, stdinText, argv = [], fallbackReason = null) {
    const result = runWasiBytesWithImports(wasmBytes, stdinText, argv, {
        wasix_32v1: {
            tty_get: () => 1,
            tty_set: () => 1,
        },
    });
    const out = {
        ...result,
        runner: 'node-wasi-wasix-tty-fallback',
        fallbackReason,
    };
    if (out.trapped && fallbackReason) {
        const detail = out.trapError || 'program trapped';
        out.trapError = `${fallbackReason}; Node WASI fallback failed: ${detail}`;
    }
    return out;
}

function isWasixTtyUnknownImport(result) {
    const detail = `${result && result.trapError ? result.trapError : ''}\n${result && result.stderr ? result.stderr : ''}`;
    return /wasix_32v1/.test(detail) && /tty_(get|set)/.test(detail) && /unknown import/i.test(detail);
}

function isWasmerExecutableMissing(result) {
    if (!result) return false;
    if (result.spawnErrorCode === 'ENOENT') return true;
    const detail = `${result.trapError || ''}\n${result.stderr || ''}`;
    return /\bENOENT\b/.test(detail) && /\bspawn\b/.test(detail) && /\bwasmer\b/i.test(detail);
}

function detectTarget(source) {
    const m = String(source || '').match(/^\s*#target\s+([^\s]+)/m);
    return m ? String(m[1]).trim() : '';
}

function runWasmerWasixBytes(wasmBytes, stdinText, argv = []) {
    const wasmPath = mkTmpPath('nepl-doctest') + '.wasm';
    const vfsRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-doctest-wasix-'));
    ensureWasiScratchDir(vfsRoot);
    fs.writeFileSync(wasmPath, Buffer.from(wasmBytes));

    const wasmerBin = process.env.WASMER_BIN || 'wasmer';
    const timeoutMs = (() => {
        const raw = parseInt(process.env.NEPL_WASIX_TIMEOUT_MS || '5000', 10);
        return Number.isFinite(raw) && raw > 0 ? raw : 5000;
    })();

    return new Promise((resolve) => {
        const mountArgs = wasmerRunMountArgs(wasmerBin, vfsRoot, '/');
        const child = spawn(
            wasmerBin,
            ['run', ...mountArgs, wasmPath, ...(Array.isArray(argv) ? argv.map((v) => String(v)) : [])],
            { stdio: ['pipe', 'pipe', 'pipe'] },
        );
        let stdout = '';
        let stderr = '';
        let finished = false;

        const cleanup = () => {
            safeUnlink(wasmPath);
            try { fs.rmSync(vfsRoot, { recursive: true, force: true }); } catch {}
        };

        const finish = (result) => {
            if (finished) return;
            finished = true;
            clearTimeout(timer);
            cleanup();
            resolve(result);
        };

        child.stdout.on('data', (chunk) => {
            stdout += chunk.toString('utf8');
        });
        child.stderr.on('data', (chunk) => {
            stderr += chunk.toString('utf8');
        });
        child.on('error', (e) => {
            finish({
                trapped: true,
                trapError: formatError(e),
                spawnErrorCode: e && e.code ? String(e.code) : null,
                stdout,
                stderr,
                exitCode: null,
                returnValue: null,
                memory: null,
                runner: 'wasmer',
            });
        });
        child.on('close', (code) => {
            finish({
                trapped: code !== 0,
                trapError: code === 0 ? null : `wasmer exit code=${code}\n${stderr.trim()}`.trim(),
                spawnErrorCode: null,
                stdout,
                stderr,
                exitCode: typeof code === 'number' ? code : null,
                returnValue: null,
                memory: null,
                runner: 'wasmer',
            });
        });

        try {
            child.stdin.end(Buffer.from(stdinText || '', 'utf-8'));
        } catch {}

        const timer = setTimeout(() => {
            try { child.kill('SIGKILL'); } catch {}
            finish({
                trapped: true,
                trapError: `wasmer timeout after ${timeoutMs}ms`,
                spawnErrorCode: null,
                stdout,
                stderr,
                exitCode: null,
                returnValue: null,
                memory: null,
                runner: 'wasmer',
            });
        }, timeoutMs);
    });
}

async function runWasixBytes(wasmBytes, stdinText, argv = []) {
    const wasmerResult = await runWasmerWasixBytes(wasmBytes, stdinText, argv);
    if (isWasmerExecutableMissing(wasmerResult)) {
        const reason = `wasmer executable not found (${process.env.WASMER_BIN || 'wasmer'}); using Node WASI WASIX TTY fallback`;
        return runWasixBytesWithTtyFallback(wasmBytes, stdinText, argv, reason);
    }
    if (isWasixTtyUnknownImport(wasmerResult)) {
        const reason = 'wasmer lacks wasix_32v1 tty_get/tty_set imports; using Node WASI WASIX TTY fallback';
        return runWasixBytesWithTtyFallback(wasmBytes, stdinText, argv, reason);
    }
    return wasmerResult;
}

async function runTargetBytes(source, wasmBytes, stdinText, argv = []) {
    const target = detectTarget(source);
    if (target === 'wasix') {
        return await runWasixBytes(wasmBytes, stdinText, argv);
    }
    return runWasiBytes(wasmBytes, stdinText, argv);
}

function hasTag(tags, name) {
    return Array.isArray(tags) && tags.includes(name);
}

function extractImportSpecs(source) {
    const specs = [];
    const re = /^\s*#(?:import|include)\s+"([^"]+)"/gm;
    let m;
    while ((m = re.exec(source)) !== null) {
        specs.push(m[1]);
    }
    return specs;
}

function resolveVirtualImport(fromVirtualFile, spec) {
    const baseDir = path.posix.dirname(fromVirtualFile);
    let out = spec.startsWith('/')
        ? spec
        : path.posix.join(baseDir, spec);
    if (!path.posix.extname(out)) out += '.nepl';
    return path.posix.normalize(out);
}

function resolveRealImport(fromRealDir, spec) {
    let out = spec.startsWith('/')
        ? path.resolve(spec)
        : path.resolve(fromRealDir, spec);
    if (!path.extname(out)) out += '.nepl';
    return out;
}

function collectVfsSources(entrySource, testFile) {
    const vfs = {};
    if (!testFile) return vfs;
    const testAbs = path.resolve(testFile);
    const rootDir = path.dirname(testAbs);
    const seen = new Set();

    function visit(source, realDir, virtualFile) {
        for (const spec of extractImportSpecs(source)) {
            if (!(spec.startsWith('./') || spec.startsWith('../') || spec.startsWith('/'))) {
                continue;
            }
            const virtualPath = resolveVirtualImport(virtualFile, spec);
            if (seen.has(virtualPath)) continue;
            const realPath = resolveRealImport(realDir, spec);
            if (!fs.existsSync(realPath) || !fs.statSync(realPath).isFile()) {
                continue;
            }
            const content = fs.readFileSync(realPath, 'utf-8');
            vfs[virtualPath] = content;
            seen.add(virtualPath);
            visit(content, path.dirname(realPath), virtualPath);
        }
    }

    visit(entrySource, rootDir, '/virtual/entry.nepl');
    return vfs;
}

function selectStdlibVfsMode(meta, forceStdlibVfs = false) {
    if (forceStdlibVfs || process.env.NEPL_RUN_TEST_FORCE_STDLIB_VFS === '1') {
        return 'forced';
    }
    const artifactPath = meta && (meta.wasmPath || meta.jsPath);
    const artifactHash = meta && meta.bundledStdlibHash ? meta.bundledStdlibHash : null;
    const needsOverride = stdlibOverrideIsNewerThanArtifact(
        path.resolve(process.cwd(), 'stdlib'),
        artifactPath,
        { missing: 'empty', artifactHash },
    );
    return needsOverride ? 'fs_override' : 'bundled';
}

function shouldPassFsStdlib(meta, forceStdlibVfs = false) {
    return selectStdlibVfsMode(meta, forceStdlibVfs) !== 'bundled';
}

function vfsHasStdlibOverlay(vfs) {
    for (const rawPath of Object.keys(vfs || {})) {
        let normalized = String(rawPath).replace(/\\/g, '/');
        if (!normalized.startsWith('/')) normalized = `/${normalized}`;
        normalized = path.posix.normalize(normalized);
        if (normalized === '/stdlib' || normalized.startsWith('/stdlib/')) {
            return true;
        }
    }
    return false;
}

function loadStdlibVfsForCompile(metrics = null) {
    const start = Date.now();
    try {
        return loadStdlibVfsFromFs(path.resolve(process.cwd(), 'stdlib'), { missing: 'empty' });
    } finally {
        if (metrics) {
            metrics.stdlib_vfs_ms = (metrics.stdlib_vfs_ms || 0) + (Date.now() - start);
        }
    }
}

function compilerSessionCacheStats(compilerApi) {
    if (!compilerApi || typeof compilerApi.loader_cache_stats_json !== 'function') {
        return null;
    }
    try {
        return JSON.parse(compilerApi.loader_cache_stats_json());
    } catch (err) {
        return { parse_error: String(err && err.message ? err.message : err) };
    }
}

const COMPILER_SESSION_MATERIALIZED_COMPILE_COUNTERS = [
    ['attempts', 'nepl_meta_materialized_compile_attempts'],
    ['attempted_surfaces', 'nepl_meta_materialized_compile_attempted_surfaces'],
    ['accepts', 'nepl_meta_materialized_compile_accepts'],
    ['source_fallbacks', 'nepl_meta_materialized_compile_source_fallbacks'],
    ['source_fallback_successes', 'nepl_meta_materialized_compile_source_fallback_successes'],
    ['source_fallback_failures', 'nepl_meta_materialized_compile_source_fallback_failures'],
    ['body_missing_fallbacks', 'nepl_meta_materialized_compile_body_missing_fallbacks'],
    ['body_missing_candidate_surfaces', 'nepl_obj_candidate_body_missing_surfaces'],
    ['body_missing_skip_hits', 'nepl_meta_body_missing_skip_hits'],
    ['body_missing_skip_stores', 'nepl_meta_body_missing_skip_stores'],
    ['body_missing_skip_stale_entries', 'nepl_meta_body_missing_skip_stale_entries'],
];

function compilerSessionCounterValue(snapshot, key) {
    if (!snapshot || typeof snapshot !== 'object') {
        return { ok: false, reason: 'missing_snapshot', counter: key };
    }
    if (Object.prototype.hasOwnProperty.call(snapshot, 'parse_error')) {
        return { ok: false, reason: 'stats_parse_error', counter: key };
    }
    if (!Object.prototype.hasOwnProperty.call(snapshot, key)) {
        return { ok: false, reason: 'missing_counter', counter: key };
    }
    const value = Number(snapshot[key]);
    if (!Number.isFinite(value)) {
        return { ok: false, reason: 'invalid_counter', counter: key };
    }
    return { ok: true, value };
}

function compilerSessionCounterDeltas(before, after, counters) {
    const out = {};
    for (const [name, key] of counters) {
        const beforeValue = compilerSessionCounterValue(before, key);
        if (!beforeValue.ok) return beforeValue;
        const afterValue = compilerSessionCounterValue(after, key);
        if (!afterValue.ok) return afterValue;
        const delta = afterValue.value - beforeValue.value;
        if (delta < 0) {
            return { ok: false, reason: 'counter_decreased', counter: key };
        }
        out[name] = {
            before: beforeValue.value,
            after: afterValue.value,
            delta,
        };
    }
    return { ok: true, value: out };
}

function compilerSessionStatsDelta(before, after) {
    if (!before || !after) {
        return { available: false, reason: 'missing_snapshot' };
    }
    const materializedCompile = compilerSessionCounterDeltas(
        before,
        after,
        COMPILER_SESSION_MATERIALIZED_COMPILE_COUNTERS,
    );
    if (!materializedCompile.ok) {
        return {
            available: false,
            reason: materializedCompile.reason,
            counter: materializedCompile.counter || null,
        };
    }
    return {
        available: true,
        reason: 'ok',
        materialized_compile: materializedCompile.value,
    };
}

function callCompilerForTiming(fn, metrics = null, compilerApi = null) {
    const start = Date.now();
    if (metrics && compilerApi) {
        metrics.compiler_session_cache_before = compilerSessionCacheStats(compilerApi);
    }
    try {
        return withConsoleSuppressed(fn);
    } finally {
        if (metrics) {
            metrics.wasm_call_ms = (metrics.wasm_call_ms || 0) + (Date.now() - start);
            if (compilerApi) {
                metrics.compiler_session_cache_after = compilerSessionCacheStats(compilerApi);
                metrics.compiler_session_stats = compilerSessionStatsDelta(
                    metrics.compiler_session_cache_before,
                    metrics.compiler_session_cache_after,
                );
            }
        }
    }
}

function compileApiForRun(api, meta = null) {
    return meta && meta.compilerSession ? meta.compilerSession : api;
}

function compileWithFsStdlib(api, source, vfs, profile = 'debug', meta = null, forceStdlibVfs = false, metrics = null, testMode = false) {
    const stdlibVfsMode = selectStdlibVfsMode(meta, forceStdlibVfs);
    const mustPassStdlibVfs = stdlibVfsMode !== 'bundled';
    const compilerApi = compileApiForRun(api, meta);
    if (metrics) {
        metrics.stdlib_vfs_mode = stdlibVfsMode;
        metrics.compiler_session = compilerApi !== api;
    }
    if (compilerApi !== api) {
        prewarmCompilerSession(meta, source, stdlibVfsMode, metrics, vfs);
    }
    if (mustPassStdlibVfs && testMode && typeof compilerApi.compile_source_with_vfs_stdlib_and_profile_test_mode === 'function') {
        const stdlibVfs = loadStdlibVfsForCompile(metrics);
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs_stdlib_and_profile_test_mode(
                '/virtual/entry.nepl',
                source,
                vfs,
                stdlibVfs,
                profile,
                true,
            ),
            metrics,
            compilerApi,
        );
    }
    if (mustPassStdlibVfs && typeof compilerApi.compile_source_with_vfs_stdlib_and_profile === 'function') {
        const stdlibVfs = loadStdlibVfsForCompile(metrics);
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs_stdlib_and_profile(
                '/virtual/entry.nepl',
                source,
                vfs,
                stdlibVfs,
                profile,
            ),
            metrics,
            compilerApi,
        );
    }
    if (mustPassStdlibVfs && typeof compilerApi.compile_source_with_vfs_and_stdlib === 'function') {
        const stdlibVfs = loadStdlibVfsForCompile(metrics);
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs_and_stdlib(
                '/virtual/entry.nepl',
                source,
                vfs,
                stdlibVfs,
            ),
            metrics,
            compilerApi,
        );
    }
    const effectiveVfs = mustPassStdlibVfs
        ? {
            ...loadStdlibVfsForCompile(metrics),
            ...vfs,
        }
        : vfs;
    if (testMode && typeof compilerApi.compile_source_with_vfs_and_profile_test_mode === 'function') {
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs_and_profile_test_mode(
                '/virtual/entry.nepl',
                source,
                effectiveVfs,
                profile,
                true,
            ),
            metrics,
            compilerApi,
        );
    }
    if (typeof compilerApi.compile_source_with_vfs_and_profile === 'function') {
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs_and_profile(
                '/virtual/entry.nepl',
                source,
                effectiveVfs,
                profile,
            ),
            metrics,
            compilerApi,
        );
    }
    if (typeof compilerApi.compile_source_with_vfs === 'function') {
        return callCompilerForTiming(() =>
            compilerApi.compile_source_with_vfs('/virtual/entry.nepl', source, effectiveVfs),
            metrics,
            compilerApi,
        );
    }
    if (typeof compilerApi.compile_source_with_profile === 'function') {
        return callCompilerForTiming(
            () => compilerApi.compile_source_with_profile(source, profile),
            metrics,
            compilerApi,
        );
    }
    return callCompilerForTiming(() => compilerApi.compile_source(source), metrics, compilerApi);
}

function createCompilerSession(api) {
    if (typeof api.CompilerSession !== 'function') {
        return null;
    }
    return new api.CompilerSession();
}

function bundledStdlibHashForCompiler(api, session) {
    if (session && typeof session.bundled_stdlib_hash === 'function') {
        return session.bundled_stdlib_hash();
    }
    if (typeof api.get_bundled_stdlib_hash === 'function') {
        return api.get_bundled_stdlib_hash();
    }
    return null;
}

function warmCompiler(api, meta) {
    const source = [
        '#entry main',
        '#target wasm',
        '#indent 4',
        '',
        'fn main %fn void i32 \\void:',
        '    0',
        '',
    ].join('\n');
    const metrics = {
        stdlib_vfs_mode: null,
        stdlib_vfs_ms: 0,
        compiler_session: false,
        compiler_session_stats: { available: false, reason: 'not_started' },
        compiler_session_cache_before: null,
        compiler_session_cache_after: null,
        compiler_session_prewarm_ms: 0,
        compiler_session_prewarm_count: null,
        compiler_session_prewarm_skipped_reason: null,
        compiler_session_prewarm_error: null,
        compiler_session_prewarm_cache_before: null,
        compiler_session_prewarm_cache_after: null,
        wasm_call_ms: 0,
    };
    try {
        compileWithFsStdlib(api, source, {}, 'debug', meta, false, metrics);
    } catch {}
    return metrics;
}

function prewarmCompilerSession(meta, source, stdlibVfsMode, metrics = null, vfs = null) {
    const session = meta && meta.compilerSession;
    if (!session || typeof session.prewarm_loader_cache_for_source !== 'function') {
        if (metrics) {
            metrics.compiler_session_prewarm_skipped_reason = 'missing_session_api';
        }
        return null;
    }
    if (stdlibVfsMode !== 'bundled') {
        if (metrics) {
            metrics.compiler_session_prewarm_skipped_reason = stdlibVfsMode || 'unknown_stdlib_mode';
        }
        return null;
    }
    if (vfsHasStdlibOverlay(vfs)) {
        if (metrics) {
            metrics.compiler_session_prewarm_skipped_reason = 'stdlib_overlay';
        }
        return null;
    }

    const start = Date.now();
    if (metrics) {
        metrics.compiler_session_prewarm_cache_before = compilerSessionCacheStats(session);
    }
    try {
        const count = session.prewarm_loader_cache_for_source('/virtual/entry.nepl', source);
        if (metrics) {
            metrics.compiler_session_prewarm_count = count;
        }
        return count;
    } catch (err) {
        if (metrics) {
            metrics.compiler_session_prewarm_error = String(err && err.message ? err.message : err);
        }
        return null;
    } finally {
        if (metrics) {
            metrics.compiler_session_prewarm_ms =
                (metrics.compiler_session_prewarm_ms || 0) + (Date.now() - start);
            metrics.compiler_session_prewarm_cache_after = compilerSessionCacheStats(session);
        }
    }
}

function withConsoleSuppressed(fn) {
    const origLog = console.log;
    const origInfo = console.info;
    const origWarn = console.warn;
    const origError = console.error;
    console.log = () => {};
    console.info = () => {};
    console.warn = () => {};
    console.error = () => {};
    try {
        return fn();
    } finally {
        console.log = origLog;
        console.info = origInfo;
        console.warn = origWarn;
        console.error = origError;
    }
}

async function createRunner(distHint) {
    const candidates = candidateDistDirs(distHint || '');
    const loaded = await withConsoleSuppressed(() => loadCompilerFromCandidates(candidates));
    loaded.meta.compilerSession = createCompilerSession(loaded.api);
    loaded.meta.bundledStdlibHash = bundledStdlibHashForCompiler(
        loaded.api,
        loaded.meta.compilerSession,
    );
    loaded.meta.compilerSessionFileCount = loaded.meta.compilerSession
        && typeof loaded.meta.compilerSession.bundled_stdlib_file_count === 'function'
        ? loaded.meta.compilerSession.bundled_stdlib_file_count()
        : null;
    let warmupMs = 0;
    loaded.meta.compilerSessionPrewarmCount = null;
    loaded.meta.compilerSessionPrewarmMs = 0;
    loaded.meta.compilerSessionPrewarmSkippedReason = null;
    loaded.meta.compilerSessionPrewarmError = null;
    if (process.env.NEPL_RUN_TEST_SKIP_COMPILER_WARMUP !== '1') {
        const warmupStart = Date.now();
        const warmupMetrics = withConsoleSuppressed(() => warmCompiler(loaded.api, loaded.meta));
        loaded.meta.compilerSessionPrewarmCount =
            warmupMetrics && warmupMetrics.compiler_session_prewarm_count !== undefined
                ? warmupMetrics.compiler_session_prewarm_count
                : null;
        loaded.meta.compilerSessionPrewarmMs =
            warmupMetrics && Number.isFinite(warmupMetrics.compiler_session_prewarm_ms)
                ? warmupMetrics.compiler_session_prewarm_ms
                : 0;
        loaded.meta.compilerSessionPrewarmSkippedReason = warmupMetrics
            ? warmupMetrics.compiler_session_prewarm_skipped_reason
            : null;
        loaded.meta.compilerSessionPrewarmError = warmupMetrics
            ? warmupMetrics.compiler_session_prewarm_error
            : null;
        warmupMs = Date.now() - warmupStart;
    }
    loaded.meta.warmupMs = warmupMs;
    return loaded;
}

function notifyPhaseProgress(onProgress, payload) {
    if (typeof onProgress !== 'function') return;
    try {
        onProgress(payload);
    } catch {}
}

async function runSingle(req, preloaded, onProgress = null) {
    const t0 = Date.now();
    const timing = {
        load_ms: 0,
        warmup_ms: 0,
        collect_vfs_ms: null,
        stdlib_vfs_ms: 0,
        stdlib_vfs_mode: null,
        compiler_session: false,
        compiler_session_stats: { available: false, reason: 'not_started' },
        compiler_session_cache_before: null,
        compiler_session_cache_after: null,
        compiler_session_prewarm_ms: 0,
        compiler_session_prewarm_count: null,
        compiler_session_prewarm_skipped_reason: null,
        compiler_session_prewarm_error: null,
        compiler_session_prewarm_cache_before: null,
        compiler_session_prewarm_cache_after: null,
        wasm_call_ms: null,
        compile_ms: null,
        run_ms: null,
    };
    const finish = (result) => {
        const totalMs = Date.now() - t0;
        return {
            ...result,
            timing: {
                ...timing,
                total_ms: totalMs,
            },
            duration_ms: totalMs,
        };
    };
    try {
        const id = req.id || '';
        const source = req.source || '';
        const tags = Array.isArray(req.tags) ? req.tags : [];
        const stdinText = req.stdin || '';
        const argv = Array.isArray(req.argv) ? req.argv.map((v) => String(v)) : [];
        const expectedRet = Object.prototype.hasOwnProperty.call(req, 'expected_ret') ? req.expected_ret : null;
        notifyPhaseProgress(onProgress, { id, phase: 'load', event: 'start', elapsed_ms: Date.now() - t0 });
        const loadStart = Date.now();
        const loaded = preloaded || await createRunner(req.distHint || '');
        timing.load_ms = Date.now() - loadStart;
        timing.warmup_ms = loaded.meta && Number.isFinite(loaded.meta.warmupMs)
            ? loaded.meta.warmupMs
            : 0;
        notifyPhaseProgress(onProgress, { id, phase: 'load', event: 'end', elapsed_ms: Date.now() - t0, phase_ms: timing.load_ms });
        const { api, meta } = loaded;
        if (hasTag(tags, 'skip')) {
            return finish({
                ok: true,
                id,
                status: 'pass',
                phase: 'skip',
                skipped: true,
                error: null,
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
            });
        }

        let wasmU8 = null;
        let compileError = null;
        notifyPhaseProgress(onProgress, { id, phase: 'compile', event: 'start', elapsed_ms: Date.now() - t0 });
        const compileStart = Date.now();
        const compileMetrics = {
            stdlib_vfs_mode: null,
            stdlib_vfs_ms: 0,
            compiler_session: false,
            compiler_session_stats: { available: false, reason: 'not_started' },
            compiler_session_cache_before: null,
            compiler_session_cache_after: null,
            compiler_session_prewarm_ms: 0,
            compiler_session_prewarm_count: null,
            compiler_session_prewarm_skipped_reason: null,
            compiler_session_prewarm_error: null,
            compiler_session_prewarm_cache_before: null,
            compiler_session_prewarm_cache_after: null,
            wasm_call_ms: 0,
        };
        try {
            const collectVfsStart = Date.now();
            const vfs = {
                ...collectVfsSources(source, req.file),
                ...(req.vfs && typeof req.vfs === 'object' ? req.vfs : {}),
            };
            timing.collect_vfs_ms = Date.now() - collectVfsStart;
            wasmU8 = compileWithFsStdlib(
                api,
                source,
                vfs,
                'debug',
                meta,
                Boolean(req.forceStdlibVfs),
                compileMetrics,
                true,
            );
        } catch (e) {
            compileError = formatError(e);
        } finally {
            timing.compile_ms = Date.now() - compileStart;
            timing.stdlib_vfs_mode = compileMetrics.stdlib_vfs_mode;
            timing.stdlib_vfs_ms = compileMetrics.stdlib_vfs_ms;
            timing.compiler_session = compileMetrics.compiler_session;
            timing.compiler_session_stats = compileMetrics.compiler_session_stats;
            timing.compiler_session_cache_before = compileMetrics.compiler_session_cache_before;
            timing.compiler_session_cache_after = compileMetrics.compiler_session_cache_after;
            timing.compiler_session_prewarm_ms = compileMetrics.compiler_session_prewarm_ms;
            timing.compiler_session_prewarm_count = compileMetrics.compiler_session_prewarm_count;
            timing.compiler_session_prewarm_skipped_reason =
                compileMetrics.compiler_session_prewarm_skipped_reason;
            timing.compiler_session_prewarm_error = compileMetrics.compiler_session_prewarm_error;
            timing.compiler_session_prewarm_cache_before =
                compileMetrics.compiler_session_prewarm_cache_before;
            timing.compiler_session_prewarm_cache_after =
                compileMetrics.compiler_session_prewarm_cache_after;
            timing.wasm_call_ms = compileMetrics.wasm_call_ms;
            notifyPhaseProgress(onProgress, {
                id,
                phase: 'compile',
                event: 'end',
                elapsed_ms: Date.now() - t0,
                phase_ms: timing.compile_ms,
                ok: compileError === null,
            });
        }

        if (hasTag(tags, 'compile_fail')) {
            const ok = (compileError !== null);
            return finish({
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'compile',
                compile_error: compileError,
                error: ok ? null : 'expected compile_fail, but compiled successfully',
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
            });
        }

        if (compileError !== null) {
            return finish({
                ok: false,
                id,
                status: 'fail',
                phase: 'compile',
                error: compileError,
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
            });
        }

        notifyPhaseProgress(onProgress, { id, phase: 'run', event: 'start', elapsed_ms: Date.now() - t0 });
        const runStart = Date.now();
        const runRes = await runTargetBytes(source, wasmU8, stdinText, argv);
        timing.run_ms = Date.now() - runStart;
        notifyPhaseProgress(onProgress, {
            id,
            phase: 'run',
            event: 'end',
            elapsed_ms: Date.now() - t0,
            phase_ms: timing.run_ms,
            ok: !runRes.trapped,
        });
        const decodedReturn = decodeExpectedReturn(
            expectedRet,
            runRes.returnValue,
            runRes.memory,
        );
        const decodedExitCode = decodeExitCode(runRes.returnValue);

        if (hasTag(tags, 'should_panic')) {
            const ok = runRes.trapped;
            return finish({
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'run',
                stdout: runRes.stdout,
                stderr: runRes.stderr,
                return_value: decodedReturn,
                exit_code: decodedExitCode,
                error: ok ? null : 'expected should_panic, but program finished without trap',
                runtime: {
                    trapped: runRes.trapped,
                    trapError: runRes.trapError,
                    runner: runRes.runner || null,
                    fallbackReason: runRes.fallbackReason || null,
                },
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
            });
        }

        const ok = !runRes.trapped;
        return finish({
            ok,
            id,
            status: ok ? 'pass' : 'fail',
            phase: 'run',
            stdout: runRes.stdout,
            stderr: runRes.stderr,
            return_value: decodedReturn,
            exit_code: decodedExitCode,
            error: ok ? null : (runRes.trapError || 'program trapped'),
            runtime: {
                trapped: runRes.trapped,
                trapError: runRes.trapError,
                runner: runRes.runner || null,
                fallbackReason: runRes.fallbackReason || null,
            },
            compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
        });
    } catch (e) {
        return finish({
            ok: false,
            status: 'error',
            error: String(e?.stack || e?.message || e),
        });
    }
}

async function main() {
    const raw = await readStdinAll();
    const req = JSON.parse(raw);
    const result = await runSingle(req);
    writeJson(result);
    if (!result.ok) {
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main().catch((e) => {
        writeJson({
            ok: false,
            status: 'error',
            error: String(e?.stack || e?.message || e),
        });
        process.exitCode = 1;
    });
}

module.exports = {
    createRunner,
    compilerSessionStatsDelta,
    ensureWasiScratchDir,
    isWasmerExecutableMissing,
    runWasixBytes,
    runSingle,
};

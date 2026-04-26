#!/usr/bin/env node
// nodesrc/run_test.js
// 目的: doctest 1 件を「コンパイル -> WASI 実行」して結果を返す。
//
// 入力:
// - JSON (stdin)
//   { "id": "...", "source": "...", "tags": [..], "stdin": "...", "distHint": "..." }
// 出力:
// - JSON (stdout)
//   { ok, id, status, stdout, stderr, error, compiler, runtime, duration_ms }

const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn } = require('node:child_process');
const { WASI } = require('node:wasi');
const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');
const { wasmerRunMountArgs } = require('./wasmer_args');

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

function runWasiBytes(wasmBytes, stdinText, argv = []) {
    return runWasiBytesWithImports(wasmBytes, stdinText, argv);
}

function runWasiBytesWithImports(wasmBytes, stdinText, argv = [], extraImports = {}) {
    const wasmPath = mkTmpPath('nepl-doctest') + '.wasm';
    const stdinPath = mkTmpPath('wasi-stdin');
    const stdoutPath = mkTmpPath('wasi-stdout');
    const stderrPath = mkTmpPath('wasi-stderr');
    const preopenRoot = process.cwd();

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

function runWasixBytesWithTtyFallback(wasmBytes, stdinText, argv = []) {
    return runWasiBytesWithImports(wasmBytes, stdinText, argv, {
        wasix_32v1: {
            tty_get: () => 1,
            tty_set: () => 1,
        },
    });
}

function isWasixTtyUnknownImport(result) {
    const detail = `${result && result.trapError ? result.trapError : ''}\n${result && result.stderr ? result.stderr : ''}`;
    return /wasix_32v1/.test(detail) && /tty_(get|set)/.test(detail) && /unknown import/i.test(detail);
}

function detectTarget(source) {
    const m = String(source || '').match(/^\s*#target\s+([^\s]+)/m);
    return m ? String(m[1]).trim() : '';
}

function runWasmerWasixBytes(wasmBytes, stdinText, argv = []) {
    const wasmPath = mkTmpPath('nepl-doctest') + '.wasm';
    const vfsRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-doctest-wasix-'));
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
                stdout,
                stderr,
                exitCode: null,
                returnValue: null,
                memory: null,
            });
        });
        child.on('close', (code) => {
            finish({
                trapped: code !== 0,
                trapError: code === 0 ? null : `wasmer exit code=${code}\n${stderr.trim()}`.trim(),
                stdout,
                stderr,
                exitCode: typeof code === 'number' ? code : null,
                returnValue: null,
                memory: null,
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
                stdout,
                stderr,
                exitCode: null,
                returnValue: null,
                memory: null,
            });
        }, timeoutMs);
    });
}

async function runWasixBytes(wasmBytes, stdinText, argv = []) {
    const wasmerResult = await runWasmerWasixBytes(wasmBytes, stdinText, argv);
    if (isWasixTtyUnknownImport(wasmerResult)) {
        return runWasixBytesWithTtyFallback(wasmBytes, stdinText, argv);
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

function toPosixPath(p) {
    return String(p).replace(/\\/g, '/');
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

function loadStdlibVfsFromFs(stdlibRootDir = path.resolve(process.cwd(), 'stdlib')) {
    const root = path.resolve(stdlibRootDir);
    if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
        return {};
    }
    const out = {};
    for (const f of walkFiles(root)) {
        if (!f.endsWith('.nepl')) continue;
        const rel = toPosixPath(path.relative(root, f));
        out[`/stdlib/${rel}`] = fs.readFileSync(f, 'utf8');
    }
    return out;
}

function compileWithFsStdlib(api, source, vfs, profile = 'debug') {
    const stdlibVfs = loadStdlibVfsFromFs();
    if (typeof api.compile_source_with_vfs_stdlib_and_profile === 'function') {
        return withConsoleSuppressed(() =>
            api.compile_source_with_vfs_stdlib_and_profile(
                '/virtual/entry.nepl',
                source,
                vfs,
                stdlibVfs,
                profile,
            )
        );
    }
    if (typeof api.compile_source_with_vfs_and_stdlib === 'function') {
        return withConsoleSuppressed(() =>
            api.compile_source_with_vfs_and_stdlib(
                '/virtual/entry.nepl',
                source,
                vfs,
                stdlibVfs,
            )
        );
    }
    if (typeof api.compile_source_with_vfs_and_profile === 'function') {
        return withConsoleSuppressed(() =>
            api.compile_source_with_vfs_and_profile(
                '/virtual/entry.nepl',
                source,
                { ...stdlibVfs, ...vfs },
                profile,
            )
        );
    }
    if (typeof api.compile_source_with_vfs === 'function') {
        return withConsoleSuppressed(() =>
            api.compile_source_with_vfs('/virtual/entry.nepl', source, { ...stdlibVfs, ...vfs })
        );
    }
    if (typeof api.compile_source_with_profile === 'function') {
        return withConsoleSuppressed(() => api.compile_source_with_profile(source, profile));
    }
    return withConsoleSuppressed(() => api.compile_source(source));
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
    return loaded;
}

async function runSingle(req, preloaded) {
    const t0 = Date.now();
    try {
        const id = req.id || '';
        const source = req.source || '';
        const tags = Array.isArray(req.tags) ? req.tags : [];
        const stdinText = req.stdin || '';
        const argv = Array.isArray(req.argv) ? req.argv.map((v) => String(v)) : [];
        const expectedRet = Object.prototype.hasOwnProperty.call(req, 'expected_ret') ? req.expected_ret : null;
        const loaded = preloaded || await createRunner(req.distHint || '');
        const { api, meta } = loaded;
        if (hasTag(tags, 'skip')) {
            return {
                ok: true,
                id,
                status: 'pass',
                phase: 'skip',
                skipped: true,
                error: null,
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            };
        }

        let wasmU8 = null;
        let compileError = null;
        try {
            const vfs = collectVfsSources(source, req.file);
            wasmU8 = compileWithFsStdlib(api, source, vfs, 'debug');
        } catch (e) {
            compileError = formatError(e);
        }

        if (hasTag(tags, 'compile_fail')) {
            const ok = (compileError !== null);
            return {
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'compile',
                compile_error: compileError,
                error: ok ? null : 'expected compile_fail, but compiled successfully',
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            };
        }

        if (compileError !== null) {
            return {
                ok: false,
                id,
                status: 'fail',
                phase: 'compile',
                error: compileError,
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            };
        }

        const runRes = await runTargetBytes(source, wasmU8, stdinText, argv);
        const decodedReturn = decodeExpectedReturn(
            expectedRet,
            runRes.returnValue,
            runRes.memory,
        );

        if (hasTag(tags, 'should_panic')) {
            const ok = runRes.trapped;
            return {
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'run',
                stdout: runRes.stdout,
                stderr: runRes.stderr,
                return_value: decodedReturn,
                error: ok ? null : 'expected should_panic, but program finished without trap',
                runtime: { trapped: runRes.trapped, trapError: runRes.trapError },
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            };
        }

        const ok = !runRes.trapped;
        return {
            ok,
            id,
            status: ok ? 'pass' : 'fail',
            phase: 'run',
            stdout: runRes.stdout,
            stderr: runRes.stderr,
            return_value: decodedReturn,
            error: ok ? null : (runRes.trapError || 'program trapped'),
            runtime: { trapped: runRes.trapped, trapError: runRes.trapError },
            compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
            duration_ms: Date.now() - t0,
        };
    } catch (e) {
        return {
            ok: false,
            status: 'error',
            error: String(e?.stack || e?.message || e),
            duration_ms: Date.now() - t0,
        };
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
    runSingle,
};

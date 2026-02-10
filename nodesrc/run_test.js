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
const { WASI } = require('node:wasi');
const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');

function readStdinAll() {
    return new Promise((resolve) => {
        const chunks = [];
        process.stdin.on('data', (c) => chunks.push(c));
        process.stdin.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
        process.stdin.resume();
    });
}

function writeJson(obj) {
    process.stdout.write(JSON.stringify(obj));
}

function mkTmpPath(prefix) {
    return path.join(os.tmpdir(), `${prefix}-${process.pid}-${Math.random().toString(16).slice(2)}`);
}

function safeUnlink(p) {
    try { fs.unlinkSync(p); } catch {}
}

function runWasiBytes(wasmBytes, stdinText) {
    const wasmPath = mkTmpPath('nepl-doctest') + '.wasm';
    const stdinPath = mkTmpPath('wasi-stdin');
    const stdoutPath = mkTmpPath('wasi-stdout');
    const stderrPath = mkTmpPath('wasi-stderr');

    fs.writeFileSync(wasmPath, Buffer.from(wasmBytes));
    fs.writeFileSync(stdinPath, Buffer.from(stdinText || '', 'utf-8'));
    fs.writeFileSync(stdoutPath, Buffer.alloc(0));
    fs.writeFileSync(stderrPath, Buffer.alloc(0));

    const stdinFd = fs.openSync(stdinPath, 'r');
    const stdoutFd = fs.openSync(stdoutPath, 'w+');
    const stderrFd = fs.openSync(stderrPath, 'w+');

    const wasi = new WASI({
        version: 'preview1',
        args: [wasmPath],
        env: {},
        stdin: stdinFd,
        stdout: stdoutFd,
        stderr: stderrFd,
    });

    let trapped = false;
    let trapError = null;
    try {
        const module = new WebAssembly.Module(Buffer.from(wasmBytes));
        const instance = new WebAssembly.Instance(module, {
            wasi_snapshot_preview1: wasi.wasiImport,
        });
        wasi.start(instance);
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
        trapError: trapError ? String(trapError?.message || trapError) : null,
        stdout: out,
        stderr: err,
    };
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
        const loaded = preloaded || await createRunner(req.distHint || '');
        const { api, meta } = loaded;

        let wasmU8 = null;
        let compileError = null;
        try {
            const vfs = collectVfsSources(source, req.file);
            if (typeof api.compile_source_with_vfs === 'function' && Object.keys(vfs).length > 0) {
                wasmU8 = withConsoleSuppressed(() =>
                    api.compile_source_with_vfs('/virtual/entry.nepl', source, vfs)
                );
            } else {
                wasmU8 = withConsoleSuppressed(() => api.compile_source(source));
            }
        } catch (e) {
            compileError = String(e?.message || e);
        }

        if (hasTag(tags, 'compile_fail')) {
            const ok = (compileError !== null);
            return {
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'compile',
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

        const runRes = runWasiBytes(wasmU8, stdinText);

        if (hasTag(tags, 'should_panic')) {
            const ok = runRes.trapped;
            return {
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'run',
                stdout: runRes.stdout,
                stderr: runRes.stderr,
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

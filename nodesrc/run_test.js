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
const { candidateDistDirs, findFirstExistingDir } = require('./util_paths');
const { loadCompilerFromDist } = require('./compiler_loader');

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

async function main() {
    const t0 = Date.now();
    try {
        const raw = await readStdinAll();
        const req = JSON.parse(raw);

        const id = req.id || '';
        const source = req.source || '';
        const tags = Array.isArray(req.tags) ? req.tags : [];
        const stdinText = req.stdin || '';

        const distHint = req.distHint || '';
        const distDir = findFirstExistingDir(candidateDistDirs(distHint));
        if (!distDir) {
            throw new Error('dist directory not found. Set NEPL_DIST or pass distHint.');
        }

        const { api, meta } = await loadCompilerFromDist(distDir);

        let wasmU8 = null;
        let compileError = null;
        try {
            wasmU8 = api.compile_source(source);
        } catch (e) {
            compileError = String(e?.message || e);
        }

        // compile_fail
        if (hasTag(tags, 'compile_fail')) {
            const ok = (compileError !== null);
            writeJson({
                ok,
                id,
                status: ok ? 'pass' : 'fail',
                phase: 'compile',
                error: ok ? null : 'expected compile_fail, but compiled successfully',
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            });
            return;
        }

        if (compileError !== null) {
            writeJson({
                ok: false,
                id,
                status: 'fail',
                phase: 'compile',
                error: compileError,
                compiler: { distDir: meta.distDir, js: meta.jsFile, wasm: meta.wasmFile },
                duration_ms: Date.now() - t0,
            });
            return;
        }

        const runRes = runWasiBytes(wasmU8, stdinText);

        // should_panic: trap することを期待（WASI の exit_code は拾いにくいので trap 判定）
        if (hasTag(tags, 'should_panic')) {
            const ok = runRes.trapped;
            writeJson({
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
            });
            return;
        }

        // 通常: trap しないことを期待
        const ok = !runRes.trapped;
        writeJson({
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
        });
    } catch (e) {
        writeJson({
            ok: false,
            status: 'error',
            error: String(e?.stack || e?.message || e),
            duration_ms: Date.now() - t0,
        });
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main();
}

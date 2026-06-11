#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function loadModules() {
    const shellModulePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'terminal', 'shell.js');
    const vfsModulePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'runtime', 'vfs.js');
    const compilerAssetsPath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'runtime', 'compiler-assets.js');
    const [shellModule, vfsModule, compilerAssetsModule] = await Promise.all([
        import(pathToFileURL(shellModulePath).href),
        import(pathToFileURL(vfsModulePath).href),
        import(pathToFileURL(compilerAssetsPath).href),
    ]);
    return { Shell: shellModule.Shell, VFS: vfsModule.VFS, compilerAssetsModule };
}

function createTerminalStub() {
    return {
        printed: [],
        written: [],
        errors: [],
        print(value) {
            this.printed.push(value);
        },
        write(value) {
            this.written.push(value);
        },
        printError(value) {
            this.errors.push(value);
        },
        clear() {},
    };
}

class FakeWorker {
    static instances = [];

    constructor() {
        this.onmessage = null;
        this.onerror = null;
        this.terminated = false;
        this.messages = [];
        FakeWorker.instances.push(this);
    }

    postMessage(message) {
        this.messages.push(message);
        queueMicrotask(() => {
            if (message.type === 'execute-neplg2') {
                this.onmessage?.({
                    data: {
                        type: 'compile_result',
                        outputs: {
                            wasm: new Uint8Array([0, 97, 115, 109]),
                            wat: '(module)',
                        },
                    },
                });
                this.onmessage?.({ data: { type: 'exit', code: 0 } });
                return;
            }
            if (message.type === 'run-wasm') {
                this.onmessage?.({
                    data: {
                        type: 'stdout',
                        fd: 1,
                        data: Array.from(new TextEncoder().encode('ok\n')),
                    },
                });
                this.onmessage?.({ data: { type: 'exit', code: 0 } });
            }
        });
    }

    terminate() {
        this.terminated = true;
    }
}

async function runShellWorkerRegression() {
    const { Shell, VFS, compilerAssetsModule } = await loadModules();
    const originalWorker = global.Worker;
    const originalWindow = global.window;
    const originalDocument = global.document;

    try {
        global.Worker = FakeWorker;
        global.window = {
            NEPLg2CompilerAssets: {
                moduleUrl: 'https://example.invalid/nepl-web.js',
                wasmUrl: 'https://example.invalid/nepl-web_bg.wasm',
            },
            wasmBindings: {
                compile_outputs_with_vfs() {
                    throw new Error('main-thread compile path must not be used');
                },
            },
        };
        global.document = {
            querySelector() {
                return null;
            },
        };

        const terminal = createTerminalStub();
        const vfs = new VFS();
        vfs.writeFile('/examples/demo.nepl', 'print "demo"\n');
        vfs.writeFile('/stdlib/std/io.nepl', 'fn io', { force: true });
        vfs.setReadOnly('/stdlib/std/io.nepl', true);
        vfs.writeFile('/data/input.txt', 'runtime data\n');
        vfs.writeFile('/out/cache.bin', new Uint8Array([1, 2, 3]));
        const shell = new Shell(terminal, vfs);

        const compilerAssets = compilerAssetsModule.resolveCompilerAssets(global.window, global.document);
        assert.deepEqual(compilerAssets, global.window.NEPLg2CompilerAssets);

        const buildResult = await shell.cmdNeplg2(['build', '-i', '/examples/demo.nepl', '--emit', 'wasm,wat']);
        assert.equal(buildResult, 'Build complete.');
        assert.equal(FakeWorker.instances[0].messages[0].type, 'execute-neplg2');
        assert.equal(FakeWorker.instances[0].messages[0].compilerMode, 'rust');
        assert.equal(FakeWorker.instances[0].messages[0].compiler.moduleUrl, global.window.NEPLg2CompilerAssets.moduleUrl);
        assert.deepEqual(FakeWorker.instances[0].messages[0].compileVfsData, {
            '/examples/demo.nepl': 'print "demo"\n',
        });
        assert.equal(FakeWorker.instances[0].messages[0].runtimeVfsData['/data/input.txt'], 'runtime data\n');
        assert.equal(FakeWorker.instances[0].messages[0].runtimeVfsData['/stdlib/std/io.nepl'], 'fn io');
        assert.deepEqual(Array.from(FakeWorker.instances[0].messages[0].runtimeVfsData['/out/cache.bin']), [1, 2, 3]);
        assert.ok(vfs.readFile('/out.wasm') instanceof Uint8Array);
        assert.equal(vfs.readFile('/out.wat'), '(module)');
        assert.equal(FakeWorker.instances[0].terminated, false);

        const secondBuildResult = await shell.cmdNeplg2(['build', '-i', '/examples/demo.nepl', '--emit', 'wasm']);
        assert.equal(secondBuildResult, 'Build complete.');
        assert.equal(FakeWorker.instances.length, 1);
        assert.equal(FakeWorker.instances[0].messages[1].type, 'execute-neplg2');
        assert.equal(FakeWorker.instances[0].messages[1].compilerMode, 'rust');

        const selfhostBuildResult = await shell.cmdNeplg2(['build', '-i', '/examples/demo.nepl', '--compiler', 'selfhost']);
        assert.equal(selfhostBuildResult, 'Build complete.');
        assert.equal(FakeWorker.instances.length, 1);
        assert.equal(FakeWorker.instances[0].messages[2].type, 'execute-neplg2');
        assert.equal(FakeWorker.instances[0].messages[2].compilerMode, 'selfhost');

        const runResult = await shell.cmdWasmi(['/out.wasm']);
        assert.equal(runResult, null);
        assert.equal(FakeWorker.instances[1].messages[0].type, 'run-wasm');
        assert.equal(FakeWorker.instances[1].terminated, true);

        const compileAndRunResult = await shell.cmdNeplg2(['run', '-i', '/examples/demo.nepl']);
        assert.equal(compileAndRunResult, null);
        assert.equal(FakeWorker.instances[0].messages[3].type, 'execute-neplg2');
        assert.equal(FakeWorker.instances[0].messages[3].compilerMode, 'rust');
        assert.equal(FakeWorker.instances[0].messages[3].runAfterBuild, false);
        assert.equal(FakeWorker.instances[2].messages[0].type, 'run-wasm');
        assert.equal(FakeWorker.instances[2].terminated, true);
        assert.equal(terminal.written.join(''), 'ok\nok\n');

        return {
            ok: true,
            checks: [
                'compiler asset urls resolve from the explicit window snapshot',
                'neplg2 build uses the worker compile protocol instead of main-thread bindings',
                'compile worker requests separate source overlay from runtime VFS state',
                'compile requests carry an explicit rust/selfhost compiler mode',
                'neplg2 build reuses one compiler worker across compile requests',
                'neplg2 run compiles through the persistent worker and executes through an ephemeral runtime worker',
                'compile outputs are written back to the VFS on the main thread',
                'wasmi execution also uses the worker protocol and streams stdout',
            ],
        };
    } finally {
        global.Worker = originalWorker;
        global.window = originalWindow;
        global.document = originalDocument;
        FakeWorker.instances.length = 0;
    }
}

if (require.main === module) {
    runShellWorkerRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + '\n'))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runShellWorkerRegression,
};

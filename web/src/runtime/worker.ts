import { WASI } from './wasi.js';
import { VFS } from './vfs.js';
import type { CompilerAssetUrls } from './compiler-assets.js';

type WorkerStdoutMessage = {
    type: 'stdout';
    fd: number;
    data: number[];
};

type WorkerCompileResultMessage = {
    type: 'compile_result';
    outputs: Record<string, string | Uint8Array>;
};

type WorkerExitMessage = {
    type: 'exit';
    code: number;
};

type WorkerErrorMessage = {
    type: 'error';
    message: string;
};

type WorkerMessage =
    | WorkerStdoutMessage
    | WorkerCompileResultMessage
    | WorkerExitMessage
    | WorkerErrorMessage
    | { type: 'stdin_request' };

type RunWasmRequest = {
    type: 'run-wasm';
    bin: Uint8Array;
    args: string[];
    env: Record<string, string>;
    vfsData: Record<string, string | Uint8Array>;
    sab: SharedArrayBuffer | null;
};

type ExecuteNeplg2Request = {
    type: 'execute-neplg2';
    compiler: CompilerAssetUrls;
    entryPath: string;
    source: string;
    vfsData: Record<string, string | Uint8Array>;
    emitValues: string[];
    attachSource: boolean;
    runAfterBuild: boolean;
    runArgs: string[];
    env: Record<string, string>;
    sab: SharedArrayBuffer | null;
};

type IncomingMessage = RunWasmRequest | ExecuteNeplg2Request;

let compilerInitPromise: Promise<any> | null = null;

class WorkerWASI extends WASI {
    stdinBuffer: Int32Array | null = null;
    stdinData: Uint8Array | null = null;
    private stdinOffset = 0;
    private stdinTotal = 0;

    constructor(args: string[], env: Map<string, string>, vfs: VFS, buffer: SharedArrayBuffer | null) {
        super(args, env, vfs, null as any);
        if (buffer) {
            this.stdinBuffer = new Int32Array(buffer, 0, 1);
            this.stdinData = new Uint8Array(buffer, 4);
        }
    }

    fd_write(fd: number, iovs: number, iovs_len: number, nwritten: number): number {
        if (!this.memory) {
            return 5;
        }
        const view = new DataView(this.memory.buffer);
        let totalWritten = 0;

        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);
            const buffer = new Uint8Array(this.memory.buffer, ptr, len);
            postWorkerMessage({ type: 'stdout', fd, data: Array.from(buffer) });
            totalWritten += len;
        }

        view.setUint32(nwritten, totalWritten, true);
        return 0;
    }

    fd_read(fd: number, iovs: number, iovs_len: number, nread: number): number {
        if (fd !== 0) {
            return super.fd_read(fd, iovs, iovs_len, nread);
        }
        if (!this.memory || !this.stdinBuffer || !this.stdinData) {
            return 5;
        }

        const view = new DataView(this.memory.buffer);

        if (this.stdinOffset >= this.stdinTotal) {
            this.stdinOffset = 0;
            this.stdinTotal = 0;
            postWorkerMessage({ type: 'stdin_request' });

            try {
                Atomics.wait(this.stdinBuffer, 0, 0);
            } catch (error) {
                console.error('Atomics.wait failed in worker:', error);
                view.setUint32(nread, 0, true);
                return 0;
            }

            this.stdinTotal = Atomics.load(this.stdinBuffer, 0);
            if (this.stdinTotal < 0) {
                view.setUint32(nread, 0, true);
                return 0;
            }
        }

        let bytesRead = 0;
        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);
            const remaining = this.stdinTotal - this.stdinOffset;
            const toRead = Math.min(len, remaining);

            if (toRead > 0) {
                const mem = new Uint8Array(this.memory.buffer, ptr, toRead);
                mem.set(this.stdinData.subarray(this.stdinOffset, this.stdinOffset + toRead));
                this.stdinOffset += toRead;
                bytesRead += toRead;
            }
        }

        if (this.stdinOffset >= this.stdinTotal) {
            Atomics.store(this.stdinBuffer, 0, 0);
        }

        view.setUint32(nread, bytesRead, true);
        return 0;
    }
}

function postWorkerMessage(message: WorkerMessage) {
    self.postMessage(message);
}

function buildEnvMap(env: Record<string, string> | undefined): Map<string, string> {
    const envMap = new Map<string, string>();
    for (const [key, value] of Object.entries(env || {})) {
        envMap.set(key, value);
    }
    return envMap;
}

function buildVfs(vfsData: Record<string, string | Uint8Array>): VFS {
    const vfs = new VFS();
    vfs.deserialize(vfsData);
    return vfs;
}

async function loadCompilerBindings(assets: CompilerAssetUrls): Promise<any> {
    if (!compilerInitPromise) {
        compilerInitPromise = (async () => {
            const compilerModule = await import(/* @vite-ignore */ assets.moduleUrl);
            if (typeof compilerModule.default === 'function') {
                await compilerModule.default({ module_or_path: assets.wasmUrl });
            }
            return compilerModule;
        })();
    }
    return compilerInitPromise;
}

function cloneCompileOutputs(outputs: any): Record<string, string | Uint8Array> {
    const cloned: Record<string, string | Uint8Array> = {};
    for (const [key, value] of Object.entries(outputs || {})) {
        if (value instanceof Uint8Array) {
            cloned[key] = new Uint8Array(value);
        } else if (ArrayBuffer.isView(value)) {
            cloned[key] = new Uint8Array(value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength));
        } else if (value instanceof ArrayBuffer) {
            cloned[key] = new Uint8Array(value.slice(0));
        } else if (typeof value === 'string') {
            cloned[key] = value;
        }
    }
    return cloned;
}

async function runWasmBinary(bin: Uint8Array, args: string[], env: Record<string, string>, vfsData: Record<string, string | Uint8Array>, sab: SharedArrayBuffer | null) {
    const wasi = new WorkerWASI(args, buildEnvMap(env), buildVfs(vfsData), sab);
    const instanceResult: any = await WebAssembly.instantiate(bin, wasi.imports);
    const instance = instanceResult instanceof WebAssembly.Instance
        ? instanceResult
        : instanceResult.instance;
    wasi.setMemory(instance.exports.memory as WebAssembly.Memory);

    if (instance.exports._start) {
        (instance.exports._start as Function)();
    } else if (instance.exports.main) {
        (instance.exports.main as Function)();
    }
}

async function executeNeplg2(request: ExecuteNeplg2Request) {
    const compilerModule = await loadCompilerBindings(request.compiler);
    const emitArg: string | string[] = request.emitValues.length === 1 ? request.emitValues[0] : request.emitValues;
    const outputs = compilerModule.compile_outputs_with_vfs(
        request.entryPath,
        request.source,
        request.vfsData,
        emitArg,
        request.attachSource
    );
    const clonedOutputs = cloneCompileOutputs(outputs);
    postWorkerMessage({ type: 'compile_result', outputs: clonedOutputs });

    if (!request.runAfterBuild) {
        postWorkerMessage({ type: 'exit', code: 0 });
        return;
    }

    const wasmOutput = clonedOutputs.wasm;
    if (!(wasmOutput instanceof Uint8Array)) {
        throw new Error('Compiled outputs do not contain a runnable wasm binary');
    }

    await runWasmBinary(
        wasmOutput,
        request.runArgs,
        request.env,
        request.vfsData,
        request.sab
    );
    postWorkerMessage({ type: 'exit', code: 0 });
}

self.onmessage = async (event: MessageEvent<IncomingMessage>) => {
    const message = event.data;
    try {
        if (message.type === 'run-wasm') {
            await runWasmBinary(message.bin, message.args, message.env, message.vfsData, message.sab);
            postWorkerMessage({ type: 'exit', code: 0 });
            return;
        }

        if (message.type === 'execute-neplg2') {
            await executeNeplg2(message);
        }
    } catch (error: any) {
        postWorkerMessage({
            type: 'error',
            message: error?.message ? String(error.message) : String(error),
        });
    }
};

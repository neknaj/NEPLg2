import { WASI } from './wasi.js';
import { VFS } from './vfs.js';
import type { CompilerAssetUrls } from './compiler-assets.js';
import {
    GUI_WEB_EVENT_KIND_ACTION,
    GUI_WEB_EVENT_KIND_KEYBOARD,
    GUI_WEB_EVENT_KIND_POINTER,
    GUI_WEB_EVENT_KIND_TIMER,
    GUI_WEB_EVENT_KIND_TEXT_INPUT,
    GUI_WEB_EVENT_KIND_WINDOW,
    GUI_WEB_EVENT_POLL_INVALID,
    GUI_WEB_EVENT_POLL_UNSUPPORTED,
    guiWebSharedKeyboardKindToRaw,
    guiWebSharedPointerButtonToRaw,
    guiWebSharedPointerKindToRaw,
    guiWebSharedWindowKindToRaw,
    takeGuiWebSharedActionId,
    takeGuiWebSharedInputEvent,
    waitGuiWebSharedActionId,
    waitGuiWebSharedInputEvent,
    type GuiWebSharedInputEventRecord,
    type GuiWebSharedInputEventTakeResult,
} from '../gui-preview/shared-event-queue.js';

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
    phase: 'compile' | 'runtime' | 'worker';
    recoverable: boolean;
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
    guiSab: SharedArrayBuffer | null;
};

type ExecuteNeplg2Request = {
    type: 'execute-neplg2';
    compilerMode?: CompilerMode;
    compiler: CompilerAssetUrls;
    entryPath: string;
    source: string;
    compileVfsData: Record<string, string>;
    runtimeVfsData: Record<string, string | Uint8Array>;
    emitValues: string[];
    attachSource: boolean;
    runAfterBuild: boolean;
    runArgs: string[];
    env: Record<string, string>;
    sab: SharedArrayBuffer | null;
};

type IncomingMessage = RunWasmRequest | ExecuteNeplg2Request;

type CompilerMode = 'rust' | 'selfhost';

type LastGuiWebInputEvent =
    | { kind: 'empty' }
    | { kind: 'event'; event: GuiWebSharedInputEventRecord };

let compilerInitPromise: Promise<any> | null = null;
let compilerSession: any | null = null;
let compilerSessionChecked = false;

class WorkerWASI extends WASI {
    stdinBuffer: Int32Array | null = null;
    stdinData: Uint8Array | null = null;
    guiEventBuffer: SharedArrayBuffer | null = null;
    private lastGuiWebInputEvent: LastGuiWebInputEvent = { kind: 'empty' };
    private stdinOffset = 0;
    private stdinTotal = 0;

    constructor(args: string[], env: Map<string, string>, vfs: VFS, buffer: SharedArrayBuffer | null, guiBuffer: SharedArrayBuffer | null) {
        super(args, env, vfs, null as any);
        if (buffer) {
            this.stdinBuffer = new Int32Array(buffer, 0, 1);
            this.stdinData = new Uint8Array(buffer, 4);
        }
        this.guiEventBuffer = guiBuffer;
        this.imports.nepl_gui_web = {
            poll_action_id: this.nepl_gui_web_poll_action_id.bind(this),
            wait_action_id: this.nepl_gui_web_wait_action_id.bind(this),
            poll_event_kind: this.nepl_gui_web_poll_event_kind.bind(this),
            wait_event_kind: this.nepl_gui_web_wait_event_kind.bind(this),
            last_event_window_id: this.nepl_gui_web_last_event_window_id.bind(this),
            last_event_action_id: this.nepl_gui_web_last_event_action_id.bind(this),
            last_event_point_x_milli: this.nepl_gui_web_last_event_point_x_milli.bind(this),
            last_event_point_y_milli: this.nepl_gui_web_last_event_point_y_milli.bind(this),
            last_event_pointer_kind: this.nepl_gui_web_last_event_pointer_kind.bind(this),
            last_event_pointer_id: this.nepl_gui_web_last_event_pointer_id.bind(this),
            last_event_pointer_button: this.nepl_gui_web_last_event_pointer_button.bind(this),
            last_event_keyboard_kind: this.nepl_gui_web_last_event_keyboard_kind.bind(this),
            last_event_key_code: this.nepl_gui_web_last_event_key_code.bind(this),
            last_event_key_modifiers: this.nepl_gui_web_last_event_key_modifiers.bind(this),
            last_event_text_scalar_value: this.nepl_gui_web_last_event_text_scalar_value.bind(this),
            last_event_window_kind: this.nepl_gui_web_last_event_window_kind.bind(this),
            last_event_window_width: this.nepl_gui_web_last_event_window_width.bind(this),
            last_event_window_height: this.nepl_gui_web_last_event_window_height.bind(this),
            last_event_timer_id: this.nepl_gui_web_last_event_timer_id.bind(this),
            last_event_timer_tick: this.nepl_gui_web_last_event_timer_tick.bind(this),
        };
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

    nepl_gui_web_poll_action_id(): number {
        if (!this.guiEventBuffer) {
            return -1;
        }
        return takeGuiWebSharedActionId(this.guiEventBuffer);
    }

    nepl_gui_web_wait_action_id(timeoutMs: number): number {
        if (!this.guiEventBuffer) {
            return -1;
        }
        const normalizedTimeout = Number.isFinite(timeoutMs) && timeoutMs >= 0
            ? timeoutMs
            : 0;
        return waitGuiWebSharedActionId(this.guiEventBuffer, normalizedTimeout);
    }

    nepl_gui_web_poll_event_kind(): number {
        if (!this.guiEventBuffer) {
            this.lastGuiWebInputEvent = { kind: 'empty' };
            return GUI_WEB_EVENT_POLL_UNSUPPORTED;
        }
        return this.storeGuiWebInputEventTakeResult(takeGuiWebSharedInputEvent(this.guiEventBuffer));
    }

    nepl_gui_web_wait_event_kind(timeoutMs: number): number {
        if (!this.guiEventBuffer) {
            this.lastGuiWebInputEvent = { kind: 'empty' };
            return GUI_WEB_EVENT_POLL_UNSUPPORTED;
        }
        const normalizedTimeout = Number.isFinite(timeoutMs) && timeoutMs >= 0
            ? timeoutMs
            : 0;
        return this.storeGuiWebInputEventTakeResult(waitGuiWebSharedInputEvent(this.guiEventBuffer, normalizedTimeout));
    }

    nepl_gui_web_last_event_window_id(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.windowId;
    }

    nepl_gui_web_last_event_action_id(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'action') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.actionId;
    }

    nepl_gui_web_last_event_point_x_milli(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'action' && this.lastGuiWebInputEvent.event.kind !== 'pointer') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.pointXMilli;
    }

    nepl_gui_web_last_event_point_y_milli(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'action' && this.lastGuiWebInputEvent.event.kind !== 'pointer') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.pointYMilli;
    }

    nepl_gui_web_last_event_pointer_kind(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'pointer') {
            return 0;
        }
        return guiWebSharedPointerKindToRaw(this.lastGuiWebInputEvent.event.pointerKind);
    }

    nepl_gui_web_last_event_pointer_id(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'pointer') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.pointerId;
    }

    nepl_gui_web_last_event_pointer_button(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'pointer') {
            return 0;
        }
        return guiWebSharedPointerButtonToRaw(this.lastGuiWebInputEvent.event.button);
    }

    nepl_gui_web_last_event_keyboard_kind(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'keyboard') {
            return 0;
        }
        return guiWebSharedKeyboardKindToRaw(this.lastGuiWebInputEvent.event.keyboardKind);
    }

    nepl_gui_web_last_event_key_code(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'keyboard') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.keyCode;
    }

    nepl_gui_web_last_event_key_modifiers(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'keyboard') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.modifierBits;
    }

    nepl_gui_web_last_event_text_scalar_value(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'text-input') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.scalarValue;
    }

    nepl_gui_web_last_event_window_kind(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'window') {
            return 0;
        }
        return guiWebSharedWindowKindToRaw(this.lastGuiWebInputEvent.event.windowKind);
    }

    nepl_gui_web_last_event_window_width(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'window') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.width;
    }

    nepl_gui_web_last_event_window_height(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'window') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.height;
    }

    nepl_gui_web_last_event_timer_id(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'timer') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.timerId;
    }

    nepl_gui_web_last_event_timer_tick(): number {
        if (this.lastGuiWebInputEvent.kind !== 'event') {
            return 0;
        }
        if (this.lastGuiWebInputEvent.event.kind !== 'timer') {
            return 0;
        }
        return this.lastGuiWebInputEvent.event.tick;
    }

    private storeGuiWebInputEventTakeResult(result: GuiWebSharedInputEventTakeResult): number {
        if (result.kind === 'empty') {
            this.lastGuiWebInputEvent = { kind: 'empty' };
            return 0;
        }
        if (result.kind === 'invalid') {
            this.lastGuiWebInputEvent = { kind: 'empty' };
            return GUI_WEB_EVENT_POLL_INVALID;
        }
        this.lastGuiWebInputEvent = {
            kind: 'event',
            event: result.event,
        };
        if (result.event.kind === 'action') {
            return GUI_WEB_EVENT_KIND_ACTION;
        }
        if (result.event.kind === 'pointer') {
            return GUI_WEB_EVENT_KIND_POINTER;
        }
        if (result.event.kind === 'keyboard') {
            return GUI_WEB_EVENT_KIND_KEYBOARD;
        }
        if (result.event.kind === 'text-input') {
            return GUI_WEB_EVENT_KIND_TEXT_INPUT;
        }
        if (result.event.kind === 'window') {
            return GUI_WEB_EVENT_KIND_WINDOW;
        }
        if (result.event.kind === 'timer') {
            return GUI_WEB_EVENT_KIND_TIMER;
        }
        this.lastGuiWebInputEvent = { kind: 'empty' };
        return GUI_WEB_EVENT_POLL_INVALID;
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

function compilerApiForSession(compilerModule: any): any {
    if (!compilerSessionChecked) {
        compilerSessionChecked = true;
        if (typeof compilerModule.CompilerSession === 'function') {
            const session = new compilerModule.CompilerSession();
            if (typeof session.compile_outputs_with_vfs === 'function') {
                compilerSession = session;
            }
        }
    }
    return compilerSession || compilerModule;
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

async function runWasmBinary(
    bin: Uint8Array,
    args: string[],
    env: Record<string, string>,
    vfsData: Record<string, string | Uint8Array>,
    sab: SharedArrayBuffer | null,
    guiSab: SharedArrayBuffer | null,
) {
    const wasi = new WorkerWASI(args, buildEnvMap(env), buildVfs(vfsData), sab, guiSab);
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

async function compileNeplg2Outputs(request: ExecuteNeplg2Request): Promise<Record<string, string | Uint8Array>> {
    const compilerModule = await loadCompilerBindings(request.compiler);
    const mode = request.compilerMode || 'rust';
    if (mode === 'selfhost') {
        return compileNeplg2OutputsWithSelfhost(compilerModule, request);
    }
    const compilerApi = compilerApiForSession(compilerModule);
    const emitArg: string | string[] = request.emitValues.length === 1 ? request.emitValues[0] : request.emitValues;
    const outputs = compilerApi.compile_outputs_with_vfs(
        request.entryPath,
        request.source,
        request.compileVfsData,
        emitArg,
        request.attachSource
    );
    return cloneCompileOutputs(outputs);
}

function selfhostCompileApi(compilerModule: any): any | null {
    const candidates = [
        compilerModule?.compile_outputs_with_vfs_selfhost,
        compilerModule?.selfhost_compile_outputs_with_vfs,
        compilerModule?.compile_outputs_with_vfs_using_selfhost,
    ];
    for (const candidate of candidates) {
        if (typeof candidate === 'function') {
            return { kind: 'function', call: candidate };
        }
    }
    if (typeof compilerModule?.SelfhostCompilerSession === 'function') {
        const session = new compilerModule.SelfhostCompilerSession();
        if (typeof session.compile_outputs_with_vfs === 'function') {
            return { kind: 'session', call: session.compile_outputs_with_vfs.bind(session) };
        }
    }
    return null;
}

async function compileNeplg2OutputsWithSelfhost(compilerModule: any, request: ExecuteNeplg2Request): Promise<Record<string, string | Uint8Array>> {
    const api = selfhostCompileApi(compilerModule);
    if (!api) {
        throw new Error('selfhost compiler mode is selected, but this playground artifact does not expose a selfhost compile_outputs_with_vfs API yet. Use Rust compiler mode until the selfhost compiler reaches runnable artifact output.');
    }
    const emitArg: string | string[] = request.emitValues.length === 1 ? request.emitValues[0] : request.emitValues;
    const outputs = await api.call(
        request.entryPath,
        request.source,
        request.compileVfsData,
        emitArg,
        request.attachSource
    );
    return cloneCompileOutputs(outputs);
}

async function executeNeplg2(request: ExecuteNeplg2Request) {
    const clonedOutputs = await compileNeplg2Outputs(request);
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
        request.runtimeVfsData,
        request.sab,
        null
    );
    postWorkerMessage({ type: 'exit', code: 0 });
}

self.onmessage = async (event: MessageEvent<IncomingMessage>) => {
    const message = event.data;
    try {
        if (message.type === 'run-wasm') {
            await runWasmBinary(message.bin, message.args, message.env, message.vfsData, message.sab, message.guiSab);
            postWorkerMessage({ type: 'exit', code: 0 });
            return;
        }

        if (message.type === 'execute-neplg2') {
            await executeNeplg2(message);
        }
    } catch (error: any) {
        const phase = message.type === 'execute-neplg2' && !message.runAfterBuild ? 'compile' : 'runtime';
        postWorkerMessage({
            type: 'error',
            message: error?.message ? String(error.message) : String(error),
            phase,
            recoverable: phase === 'compile',
        });
    }
};

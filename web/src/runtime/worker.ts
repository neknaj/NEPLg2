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
import {
    acquireGuiVideoMemoryWriteSlot,
    closeGuiVideoMemorySurface,
    createGuiVideoMemorySurface,
    discardGuiVideoMemoryWriteSlot,
    publishGuiVideoMemoryWriteSlot,
    writeGuiVideoMemoryRgba8888Row,
    type GuiVideoMemoryDirtyRegion,
    type GuiVideoMemoryError,
    type GuiVideoMemorySurface,
    type GuiVideoMemoryWriteSlot,
} from '../gui-preview/video-memory-surface.js';
import {
    createGuiVideoMemoryHostAckBuffer,
    GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE,
    GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT,
    GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT,
    GUI_VIDEO_MEMORY_HOST_STATUS_OK,
    GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED,
    GUI_VIDEO_MEMORY_HOST_STATUS_STALE_FRAME,
    GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED,
    waitGuiVideoMemoryHostAck,
} from '../gui-preview/video-memory-host-abi.js';

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
    phase: 'compile' | 'runtime' | 'worker' | 'compiler-init';
    recoverable: boolean;
};

type WorkerGuiVideoMemoryPresentMessage = {
    type: 'gui_video_memory_present';
    requestId: number;
    ack: SharedArrayBuffer;
    windowId: number;
    title: string;
    buffer: SharedArrayBuffer;
};

type WorkerMessage =
    | WorkerStdoutMessage
    | WorkerCompileResultMessage
    | WorkerExitMessage
    | WorkerErrorMessage
    | WorkerGuiVideoMemoryPresentMessage
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

type GuiWebVideoMemoryHostFrameRecord = {
    frameId: number;
    slot: GuiVideoMemoryWriteSlot;
};

type GuiWebVideoMemoryHostSurfaceRecord = {
    handle: number;
    surface: GuiVideoMemorySurface;
    nextFrameId: number;
    frames: GuiWebVideoMemoryHostFrameRecord[];
};

let compilerInitPromise: Promise<any> | null = null;
let compilerSession: any | null = null;
let compilerSessionChecked = false;

class CompilerInitializationError extends Error {
    original: unknown;

    constructor(error: unknown) {
        const message = error instanceof Error && error.message
            ? error.message
            : String(error);
        super(`compiler initialization failed: ${message}`);
        this.name = 'CompilerInitializationError';
        this.original = error;
    }
}

class WorkerWASI extends WASI {
    stdinBuffer: Int32Array | null = null;
    stdinData: Uint8Array | null = null;
    guiEventBuffer: SharedArrayBuffer | null = null;
    private lastGuiWebInputEvent: LastGuiWebInputEvent = { kind: 'empty' };
    private nextGuiVideoMemorySurfaceHandle = 1;
    private nextGuiVideoMemoryPresentRequestId = 1;
    private guiVideoMemorySurfaces: GuiWebVideoMemoryHostSurfaceRecord[] = [];
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
            video_memory_create_surface: this.nepl_gui_web_video_memory_create_surface.bind(this),
            video_memory_acquire_write_slot: this.nepl_gui_web_video_memory_acquire_write_slot.bind(this),
            video_memory_write_slot_bytes: this.nepl_gui_web_video_memory_write_slot_bytes.bind(this),
            video_memory_write_rgba8888_row: this.nepl_gui_web_video_memory_write_rgba8888_row.bind(this),
            video_memory_fill_rect_rgba8888: this.nepl_gui_web_video_memory_fill_rect_rgba8888.bind(this),
            video_memory_discard_write_slot: this.nepl_gui_web_video_memory_discard_write_slot.bind(this),
            video_memory_publish_slot: this.nepl_gui_web_video_memory_publish_slot.bind(this),
            video_memory_present_surface: this.nepl_gui_web_video_memory_present_surface.bind(this),
            video_memory_close_surface: this.nepl_gui_web_video_memory_close_surface.bind(this),
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

    nepl_gui_web_video_memory_create_surface(width: number, height: number, slotCount: number): number {
        const created = createGuiVideoMemorySurface(width, height, slotCount);
        if (created.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(created.error);
        }
        const handle = this.nextGuiVideoMemorySurfaceHandle;
        this.nextGuiVideoMemorySurfaceHandle += 1;
        this.guiVideoMemorySurfaces = [
            ...this.guiVideoMemorySurfaces,
            {
                handle,
                surface: created.value,
                nextFrameId: 1,
                frames: [],
            },
        ];
        return handle;
    }

    nepl_gui_web_video_memory_acquire_write_slot(surfaceHandle: number): number {
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        if (!surface) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const acquired = acquireGuiVideoMemoryWriteSlot(surface.surface);
        if (acquired.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(acquired.error);
        }
        const frameId = surface.nextFrameId;
        surface.nextFrameId += 1;
        surface.frames = [
            ...surface.frames,
            {
                frameId,
                slot: acquired.value,
            },
        ];
        return frameId;
    }

    nepl_gui_web_video_memory_write_slot_bytes(
        surfaceHandle: number,
        frameId: number,
        dstOffset: number,
        srcPtr: number,
        byteLen: number,
    ): number {
        const frame = this.findGuiVideoMemoryFrame(surfaceHandle, frameId);
        if (
            !frame
            || !isNonNegativeInteger(dstOffset)
            || !isNonNegativeInteger(byteLen)
            || dstOffset + byteLen > frame.slot.surface.pixelByteLength
        ) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const source = this.memoryBytes(srcPtr, byteLen);
        if (typeof source === 'number') {
            return source;
        }
        frame.slot.pixels.set(source, dstOffset);
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
    }

    nepl_gui_web_video_memory_write_rgba8888_row(
        surfaceHandle: number,
        frameId: number,
        x: number,
        y: number,
        width: number,
        srcPtr: number,
    ): number {
        const frame = this.findGuiVideoMemoryFrame(surfaceHandle, frameId);
        if (!frame || !isPositiveInteger(width)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const byteLen = width * 4;
        if (!Number.isSafeInteger(byteLen)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const source = this.memoryBytes(srcPtr, byteLen);
        if (typeof source === 'number') {
            return source;
        }
        const written = writeGuiVideoMemoryRgba8888Row(frame.slot, x, y, width, source);
        if (written.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(written.error);
        }
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
    }

    nepl_gui_web_video_memory_fill_rect_rgba8888(
        surfaceHandle: number,
        frameId: number,
        x: number,
        y: number,
        width: number,
        height: number,
        r: number,
        g: number,
        b: number,
        a: number,
    ): number {
        const frame = this.findGuiVideoMemoryFrame(surfaceHandle, frameId);
        if (!frame || !isValidRect(frame.slot.surface, x, y, width, height) || !areValidColorChannels(r, g, b, a)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        for (let row = y; row < y + height; row += 1) {
            let offset = row * frame.slot.surface.strideBytes + x * 4;
            for (let column = 0; column < width; column += 1) {
                frame.slot.pixels[offset] = r;
                frame.slot.pixels[offset + 1] = g;
                frame.slot.pixels[offset + 2] = b;
                frame.slot.pixels[offset + 3] = a;
                offset += 4;
            }
        }
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
    }

    nepl_gui_web_video_memory_discard_write_slot(surfaceHandle: number, frameId: number): number {
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        const frame = this.findGuiVideoMemoryFrame(surfaceHandle, frameId);
        if (!surface || !frame) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const discarded = discardGuiVideoMemoryWriteSlot(frame.slot);
        if (discarded.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(discarded.error);
        }
        surface.frames = surface.frames.filter((candidate) => candidate.frameId !== frameId);
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
    }

    nepl_gui_web_video_memory_publish_slot(
        surfaceHandle: number,
        frameId: number,
        dirtyKind: number,
        x: number,
        y: number,
        width: number,
        height: number,
    ): number {
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        const frame = this.findGuiVideoMemoryFrame(surfaceHandle, frameId);
        if (!surface || !frame) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const dirty = this.decodeGuiVideoMemoryDirtyRegion(frame.slot.surface, dirtyKind, x, y, width, height);
        if (typeof dirty === 'number') {
            return dirty;
        }
        const published = publishGuiVideoMemoryWriteSlot(frame.slot, dirty);
        if (published.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(published.error);
        }
        surface.frames = surface.frames.filter((candidate) => candidate.frameId !== frameId);
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
    }

    nepl_gui_web_video_memory_present_surface(
        windowId: number,
        titlePtr: number,
        titleLen: number,
        surfaceHandle: number,
    ): number {
        if (!isPositiveInteger(windowId)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        if (!surface) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const title = this.decodeGuiVideoMemoryTitle(titlePtr, titleLen);
        if (typeof title === 'number') {
            return title;
        }
        return this.presentGuiVideoMemorySurface(windowId, title, surface);
    }

    nepl_gui_web_video_memory_close_surface(surfaceHandle: number): number {
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        if (!surface) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const closed = closeGuiVideoMemorySurface(surface.surface);
        if (closed.kind === 'err') {
            return guiVideoMemoryHostStatusFromError(closed.error);
        }
        this.guiVideoMemorySurfaces = this.guiVideoMemorySurfaces.filter((candidate) => candidate.handle !== surfaceHandle);
        return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
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

    private presentGuiVideoMemorySurface(windowId: number, title: string, surface: GuiWebVideoMemoryHostSurfaceRecord): number {
        const ack = createGuiVideoMemoryHostAckBuffer();
        if (ack.kind === 'err') {
            return ack.status;
        }
        const requestId = this.nextGuiVideoMemoryPresentRequestId;
        this.nextGuiVideoMemoryPresentRequestId += 1;
        postWorkerMessage({
            type: 'gui_video_memory_present',
            requestId,
            ack: ack.value,
            windowId,
            title,
            buffer: surface.surface.buffer,
        });
        return waitGuiVideoMemoryHostAck(ack.value);
    }

    private findGuiVideoMemorySurface(surfaceHandle: number): GuiWebVideoMemoryHostSurfaceRecord | null {
        if (!isPositiveInteger(surfaceHandle)) {
            return null;
        }
        for (const surface of this.guiVideoMemorySurfaces) {
            if (surface.handle === surfaceHandle) {
                return surface;
            }
        }
        return null;
    }

    private findGuiVideoMemoryFrame(surfaceHandle: number, frameId: number): GuiWebVideoMemoryHostFrameRecord | null {
        const surface = this.findGuiVideoMemorySurface(surfaceHandle);
        if (!surface || !isPositiveInteger(frameId)) {
            return null;
        }
        for (const frame of surface.frames) {
            if (frame.frameId === frameId) {
                return frame;
            }
        }
        return null;
    }

    private decodeGuiVideoMemoryDirtyRegion(
        surface: GuiVideoMemorySurface,
        dirtyKind: number,
        x: number,
        y: number,
        width: number,
        height: number,
    ): GuiVideoMemoryDirtyRegion | number {
        if (dirtyKind === 1) {
            return { kind: 'full' };
        }
        if (dirtyKind !== 2 || !isValidRect(surface, x, y, width, height)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        return {
            kind: 'rect',
            x,
            y,
            width,
            height,
        };
    }

    private memoryBytes(ptr: number, len: number): Uint8Array | number {
        if (!this.memory || !isNonNegativeInteger(ptr) || !isNonNegativeInteger(len)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        if (ptr + len > this.memory.buffer.byteLength) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        return new Uint8Array(this.memory.buffer, ptr, len);
    }

    private decodeGuiVideoMemoryTitle(ptr: number, len: number): string | number {
        const bytes = this.memoryBytes(ptr, len);
        if (typeof bytes === 'number') {
            return bytes;
        }
        try {
            return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
        } catch {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
    }
}

function isNonNegativeInteger(value: number): boolean {
    return Number.isInteger(value) && value >= 0;
}

function isPositiveInteger(value: number): boolean {
    return Number.isInteger(value) && value > 0;
}

function areValidColorChannels(r: number, g: number, b: number, a: number): boolean {
    return isByte(r) && isByte(g) && isByte(b) && isByte(a);
}

function isByte(value: number): boolean {
    return Number.isInteger(value) && value >= 0 && value <= 255;
}

function isValidRect(surface: GuiVideoMemorySurface, x: number, y: number, width: number, height: number): boolean {
    return isNonNegativeInteger(x)
        && isNonNegativeInteger(y)
        && isNonNegativeInteger(width)
        && isNonNegativeInteger(height)
        && x + width <= surface.width
        && y + height <= surface.height;
}

function guiVideoMemoryHostStatusFromError(error: GuiVideoMemoryError): number {
    if (error.kind === 'shared-buffer-unavailable' || error.kind === 'wait-unavailable' || error.kind === 'unsupported-pixel-format' || error.kind === 'unsupported-stride' || error.kind === 'unsupported-command') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED;
    }
    if (error.kind === 'no-writable-slot') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT;
    }
    if (error.kind === 'no-published-slot') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED;
    }
    if (error.kind === 'resource-exhausted') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED;
    }
    if (error.kind === 'stale-resize-generation' || error.kind === 'writer-closed') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_STALE_FRAME;
    }
    if (error.kind === 'presenter-unavailable' || error.kind === 'present-failed') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE;
    }
    return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
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
        })().catch((error) => {
            resetCompilerInitializationState();
            throw new CompilerInitializationError(error);
        });
    }
    return compilerInitPromise;
}

function resetCompilerInitializationState() {
    compilerInitPromise = null;
    compilerSession = null;
    compilerSessionChecked = false;
}

function compilerApiForSession(compilerModule: any): any {
    if (!compilerSessionChecked) {
        try {
            let selectedSession: any | null = null;
            if (typeof compilerModule.CompilerSession === 'function') {
                const session = new compilerModule.CompilerSession();
                if (typeof session.compile_outputs_with_vfs === 'function') {
                    selectedSession = session;
                }
            }
            compilerSession = selectedSession;
            compilerSessionChecked = true;
        } catch (error) {
            resetCompilerInitializationState();
            throw new CompilerInitializationError(error);
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
        const isCompilerInitFailure = error instanceof CompilerInitializationError;
        const phase = isCompilerInitFailure
            ? 'compiler-init'
            : message.type === 'execute-neplg2' && !message.runAfterBuild ? 'compile' : 'runtime';
        postWorkerMessage({
            type: 'error',
            message: error?.message ? String(error.message) : String(error),
            phase,
            recoverable: phase === 'compile',
        });
    }
};

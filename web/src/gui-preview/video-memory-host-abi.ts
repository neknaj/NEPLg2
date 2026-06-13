export const GUI_VIDEO_MEMORY_HOST_ACK_PENDING = 0;
export const GUI_VIDEO_MEMORY_HOST_ACK_DONE = 1;

export const GUI_VIDEO_MEMORY_HOST_STATUS_OK = 0;
export const GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED = -1;
export const GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT = -2;
export const GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED = -3;
export const GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT = -4;
export const GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE = -5;
export const GUI_VIDEO_MEMORY_HOST_STATUS_STALE_FRAME = -6;

const GUI_VIDEO_MEMORY_HOST_ACK_WORDS = 2;
const GUI_VIDEO_MEMORY_HOST_ACK_STATE = 0;
const GUI_VIDEO_MEMORY_HOST_ACK_STATUS = 1;

export type GuiVideoMemoryHostAckBufferResult =
    | { kind: 'ok'; value: SharedArrayBuffer }
    | { kind: 'err'; status: number };

export function createGuiVideoMemoryHostAckBuffer(): GuiVideoMemoryHostAckBufferResult {
    if (typeof globalThis.SharedArrayBuffer !== 'function') {
        return { kind: 'err', status: GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED };
    }
    const buffer = new SharedArrayBuffer(GUI_VIDEO_MEMORY_HOST_ACK_WORDS * Int32Array.BYTES_PER_ELEMENT);
    const view = new Int32Array(buffer);
    Atomics.store(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE, GUI_VIDEO_MEMORY_HOST_ACK_PENDING);
    Atomics.store(view, GUI_VIDEO_MEMORY_HOST_ACK_STATUS, GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE);
    return { kind: 'ok', value: buffer };
}

export function resolveGuiVideoMemoryHostAck(buffer: SharedArrayBuffer, status: number): void {
    const view = new Int32Array(buffer);
    Atomics.store(view, GUI_VIDEO_MEMORY_HOST_ACK_STATUS, status);
    Atomics.store(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE, GUI_VIDEO_MEMORY_HOST_ACK_DONE);
    Atomics.notify(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE, 1);
}

export function waitGuiVideoMemoryHostAck(buffer: SharedArrayBuffer, timeoutMs?: number): number {
    const view = new Int32Array(buffer);
    const normalizedTimeout = timeoutMs === undefined
        ? undefined
        : Number.isFinite(timeoutMs) && timeoutMs >= 0
            ? timeoutMs
            : 0;
    const state = Atomics.load(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE);
    if (state !== GUI_VIDEO_MEMORY_HOST_ACK_DONE) {
        let waitResult: 'ok' | 'not-equal' | 'timed-out';
        try {
            waitResult = Atomics.wait(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE, GUI_VIDEO_MEMORY_HOST_ACK_PENDING, normalizedTimeout);
        } catch {
            return GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED;
        }
        if (waitResult === 'timed-out') {
            return GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE;
        }
    }
    if (Atomics.load(view, GUI_VIDEO_MEMORY_HOST_ACK_STATE) !== GUI_VIDEO_MEMORY_HOST_ACK_DONE) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE;
    }
    return Atomics.load(view, GUI_VIDEO_MEMORY_HOST_ACK_STATUS);
}

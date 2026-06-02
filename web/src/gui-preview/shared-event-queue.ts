import type { GuiWebInputEvent } from './input-bridge.js';

export const GUI_WEB_EVENT_QUEUE_CAPACITY = 64;
export const GUI_WEB_EVENT_QUEUE_HEADER_LENGTH = 2;
export const GUI_WEB_EVENT_QUEUE_SLOT_LENGTH = 5;
export const GUI_WEB_EVENT_KIND_ACTION = 1;
export const GUI_WEB_EVENT_POLL_UNSUPPORTED = -1;
export const GUI_WEB_EVENT_POLL_INVALID = -2;
export const GUI_WEB_EVENT_QUEUE_READ_INDEX = 0;
export const GUI_WEB_EVENT_QUEUE_WRITE_INDEX = 1;

const GUI_WEB_EVENT_SLOT_KIND = 0;
const GUI_WEB_EVENT_SLOT_WINDOW_ID = 1;
const GUI_WEB_EVENT_SLOT_ACTION_ID = 2;
const GUI_WEB_EVENT_SLOT_POINT_X_MILLI = 3;
const GUI_WEB_EVENT_SLOT_POINT_Y_MILLI = 4;

export type GuiWebSharedEventQueueErrorKind =
    | 'shared-event-buffer-unavailable'
    | 'unsupported-event-kind'
    | 'event-queue-full';

export type GuiWebSharedEventQueueError = {
    kind: GuiWebSharedEventQueueErrorKind;
    path: string;
    expected: string;
    actual: string;
};

export type GuiWebSharedEventQueueResult<Value> =
    | { kind: 'ok'; value: Value }
    | { kind: 'err'; error: GuiWebSharedEventQueueError };

export type GuiWebSharedInputEventRecord =
    | {
        kind: 'action';
        windowId: number;
        actionId: number;
        pointXMilli: number;
        pointYMilli: number;
    };

export type GuiWebSharedInputEventTakeResult =
    | { kind: 'empty' }
    | { kind: 'event'; event: GuiWebSharedInputEventRecord }
    | { kind: 'invalid'; rawKind: number };

export function createGuiWebSharedEventBuffer(): GuiWebSharedEventQueueResult<SharedArrayBuffer> {
    if (typeof SharedArrayBuffer === 'undefined') {
        return err('shared-event-buffer-unavailable', '$', 'SharedArrayBuffer constructor', 'unavailable');
    }
    return {
        kind: 'ok',
        value: new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * guiWebSharedEventQueueInt32Length()),
    };
}

export function resetGuiWebSharedEventBuffer(buffer: SharedArrayBuffer): GuiWebSharedEventQueueResult<'reset'> {
    const queue = new Int32Array(buffer);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX, 0);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 0);
    return { kind: 'ok', value: 'reset' };
}

export function writeGuiWebSharedInputEvent(
    buffer: SharedArrayBuffer,
    event: GuiWebInputEvent,
): GuiWebSharedEventQueueResult<'queued'> {
    if (event.kind === 'action') {
        return writeGuiWebSharedActionEvent(buffer, event);
    }
    return err('unsupported-event-kind', '$.kind', 'action', event.kind);
}

export function takeGuiWebSharedActionId(buffer: SharedArrayBuffer): number {
    return guiWebSharedActionIdFromTakeResult(takeGuiWebSharedInputEvent(buffer));
}

export function takeGuiWebSharedInputEvent(buffer: SharedArrayBuffer): GuiWebSharedInputEventTakeResult {
    const queue = new Int32Array(buffer);
    const readIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX);
    const writeIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX);
    if (readIndex === writeIndex) {
        return { kind: 'empty' };
    }

    const base = guiWebSharedEventSlotBase(readIndex);
    const kind = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_KIND);
    const windowId = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID);
    const actionId = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_ACTION_ID);
    const pointXMilli = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI);
    const pointYMilli = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI);
    const nextReadIndex = guiWebSharedEventNextIndex(readIndex);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX, nextReadIndex);
    if (kind !== GUI_WEB_EVENT_KIND_ACTION || actionId <= 0) {
        return { kind: 'invalid', rawKind: kind };
    }
    return {
        kind: 'event',
        event: {
            kind: 'action',
            windowId,
            actionId,
            pointXMilli,
            pointYMilli,
        },
    };
}

export function waitGuiWebSharedInputEvent(buffer: SharedArrayBuffer, timeoutMs: number): GuiWebSharedInputEventTakeResult {
    const first = takeGuiWebSharedInputEvent(buffer);
    if (first.kind !== 'empty') {
        return first;
    }

    const queue = new Int32Array(buffer);
    const writeIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX);
    try {
        Atomics.wait(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, writeIndex, timeoutMs);
    } catch {
        return { kind: 'empty' };
    }
    return takeGuiWebSharedInputEvent(buffer);
}

export function waitGuiWebSharedActionId(buffer: SharedArrayBuffer, timeoutMs: number): number {
    return guiWebSharedActionIdFromTakeResult(waitGuiWebSharedInputEvent(buffer, timeoutMs));
}

export function guiWebSharedEventQueueInt32Length(): number {
    return GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + GUI_WEB_EVENT_QUEUE_CAPACITY * GUI_WEB_EVENT_QUEUE_SLOT_LENGTH;
}

function writeGuiWebSharedActionEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'action' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const readIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX);
    const writeIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX);
    const nextWriteIndex = guiWebSharedEventNextIndex(writeIndex);
    if (nextWriteIndex === readIndex) {
        return err('event-queue-full', '$', 'available event queue slot', 'full');
    }

    const base = guiWebSharedEventSlotBase(writeIndex);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_ACTION);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_ACTION_ID, event.actionId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, Math.round(event.point.x * 1000));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, Math.round(event.point.y * 1000));
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, nextWriteIndex);
    Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    return { kind: 'ok', value: 'queued' };
}

function guiWebSharedEventNextIndex(index: number): number {
    return (index + 1) % GUI_WEB_EVENT_QUEUE_CAPACITY;
}

function guiWebSharedEventSlotBase(index: number): number {
    return GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + index * GUI_WEB_EVENT_QUEUE_SLOT_LENGTH;
}

function guiWebSharedActionIdFromTakeResult(result: GuiWebSharedInputEventTakeResult): number {
    if (result.kind === 'empty') {
        return 0;
    }
    if (result.kind === 'invalid') {
        return GUI_WEB_EVENT_POLL_INVALID;
    }
    if (result.event.kind === 'action') {
        return result.event.actionId;
    }
    return GUI_WEB_EVENT_POLL_INVALID;
}

function err(
    kind: GuiWebSharedEventQueueErrorKind,
    path: string,
    expected: string,
    actual: string,
): GuiWebSharedEventQueueResult<never> {
    return {
        kind: 'err',
        error: {
            kind,
            path,
            expected,
            actual,
        },
    };
}

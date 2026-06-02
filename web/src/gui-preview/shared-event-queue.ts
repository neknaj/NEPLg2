import type { GuiWebInputEvent } from './input-bridge.js';
import type { GuiWebKeyboardEventKind } from './input-bridge.js';
import type { GuiWebPointerButton, GuiWebPointerEventKind } from './input-bridge.js';
import type { GuiWebWindowEventKind } from './input-bridge.js';

export const GUI_WEB_EVENT_QUEUE_CAPACITY = 64;
export const GUI_WEB_ACTION_QUEUE_CAPACITY = 64;
export const GUI_WEB_EVENT_QUEUE_HEADER_LENGTH = 4;
export const GUI_WEB_EVENT_QUEUE_SLOT_LENGTH = 8;
export const GUI_WEB_EVENT_KIND_ACTION = 1;
export const GUI_WEB_EVENT_KIND_POINTER = 2;
export const GUI_WEB_EVENT_KIND_KEYBOARD = 3;
export const GUI_WEB_EVENT_KIND_TEXT_INPUT = 4;
export const GUI_WEB_EVENT_KIND_WINDOW = 5;
export const GUI_WEB_EVENT_POLL_UNSUPPORTED = -1;
export const GUI_WEB_EVENT_POLL_INVALID = -2;
export const GUI_WEB_EVENT_QUEUE_READ_INDEX = 0;
export const GUI_WEB_EVENT_QUEUE_WRITE_INDEX = 1;
export const GUI_WEB_ACTION_QUEUE_READ_INDEX = 2;
export const GUI_WEB_ACTION_QUEUE_WRITE_INDEX = 3;
export const GUI_WEB_POINTER_KIND_MOVE = 1;
export const GUI_WEB_POINTER_KIND_DOWN = 2;
export const GUI_WEB_POINTER_KIND_UP = 3;
export const GUI_WEB_POINTER_KIND_CANCEL = 4;
export const GUI_WEB_POINTER_BUTTON_NONE = 0;
export const GUI_WEB_POINTER_BUTTON_PRIMARY = 1;
export const GUI_WEB_POINTER_BUTTON_SECONDARY = 2;
export const GUI_WEB_POINTER_BUTTON_MIDDLE = 3;
export const GUI_WEB_KEYBOARD_KIND_DOWN = 1;
export const GUI_WEB_KEYBOARD_KIND_UP = 2;
export const GUI_WEB_WINDOW_KIND_RESIZED = 1;
export const GUI_WEB_WINDOW_KIND_FOCUSED = 2;
export const GUI_WEB_WINDOW_KIND_UNFOCUSED = 3;
export const GUI_WEB_WINDOW_KIND_CLOSE_REQUESTED = 4;

const GUI_WEB_EVENT_SLOT_KIND = 0;
const GUI_WEB_EVENT_SLOT_WINDOW_ID = 1;
const GUI_WEB_EVENT_SLOT_VALUE0 = 2;
const GUI_WEB_EVENT_SLOT_POINT_X_MILLI = 3;
const GUI_WEB_EVENT_SLOT_POINT_Y_MILLI = 4;
const GUI_WEB_EVENT_SLOT_VALUE1 = 5;
const GUI_WEB_EVENT_SLOT_VALUE2 = 6;
const GUI_WEB_EVENT_SLOT_VALUE3 = 7;

export type GuiWebSharedEventQueueErrorKind =
    | 'shared-event-buffer-unavailable'
    | 'unsupported-event-kind'
    | 'event-queue-full'
    | 'action-queue-full';

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
    }
    | {
        kind: 'pointer';
        windowId: number;
        pointerKind: GuiWebPointerEventKind;
        pointerId: number;
        button: GuiWebPointerButton;
        pointXMilli: number;
        pointYMilli: number;
    }
    | {
        kind: 'keyboard';
        windowId: number;
        keyboardKind: GuiWebKeyboardEventKind;
        keyCode: number;
        modifierBits: number;
    }
    | {
        kind: 'text-input';
        windowId: number;
        scalarValue: number;
    }
    | {
        kind: 'window';
        windowId: number;
        windowKind: GuiWebWindowEventKind;
        width: number;
        height: number;
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
    Atomics.store(queue, GUI_WEB_ACTION_QUEUE_READ_INDEX, 0);
    Atomics.store(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX, 0);
    return { kind: 'ok', value: 'reset' };
}

export function writeGuiWebSharedInputEvent(
    buffer: SharedArrayBuffer,
    event: GuiWebInputEvent,
): GuiWebSharedEventQueueResult<'queued'> {
    if (event.kind === 'action') {
        return writeGuiWebSharedActionEvent(buffer, event);
    }
    if (event.kind === 'pointer') {
        return writeGuiWebSharedPointerEvent(buffer, event);
    }
    if (event.kind === 'keyboard') {
        return writeGuiWebSharedKeyboardEvent(buffer, event);
    }
    if (event.kind === 'text-input') {
        return writeGuiWebSharedTextInputEvent(buffer, event);
    }
    if (event.kind === 'window') {
        return writeGuiWebSharedWindowEvent(buffer, event);
    }
    return err('unsupported-event-kind', '$.kind', 'action, pointer, keyboard, text-input, or window', String(event));
}

export function takeGuiWebSharedActionId(buffer: SharedArrayBuffer): number {
    const queue = new Int32Array(buffer);
    const readIndex = Atomics.load(queue, GUI_WEB_ACTION_QUEUE_READ_INDEX);
    const writeIndex = Atomics.load(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX);
    if (readIndex === writeIndex) {
        return 0;
    }
    const actionId = Atomics.load(queue, guiWebSharedActionSlotBase(readIndex));
    Atomics.store(queue, GUI_WEB_ACTION_QUEUE_READ_INDEX, guiWebSharedActionNextIndex(readIndex));
    if (actionId <= 0) {
        return GUI_WEB_EVENT_POLL_INVALID;
    }
    return actionId;
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
    const value0 = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_VALUE0);
    const pointXMilli = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI);
    const pointYMilli = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI);
    const value1 = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_VALUE1);
    const value2 = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_VALUE2);
    const value3 = Atomics.load(queue, base + GUI_WEB_EVENT_SLOT_VALUE3);
    const nextReadIndex = guiWebSharedEventNextIndex(readIndex);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX, nextReadIndex);
    if (kind === GUI_WEB_EVENT_KIND_ACTION) {
        if (value0 <= 0) {
            return { kind: 'invalid', rawKind: kind };
        }
        return {
            kind: 'event',
            event: {
                kind: 'action',
                windowId,
                actionId: value0,
                pointXMilli,
                pointYMilli,
            },
        };
    }
    if (kind === GUI_WEB_EVENT_KIND_POINTER) {
        const pointerKind = guiWebSharedPointerKindFromRaw(value1);
        if (pointerKind.kind === 'err') {
            return { kind: 'invalid', rawKind: kind };
        }
        const button = guiWebSharedPointerButtonFromRaw(value3);
        if (button.kind === 'err') {
            return { kind: 'invalid', rawKind: kind };
        }
        if (value2 <= 0) {
            return { kind: 'invalid', rawKind: kind };
        }
        return {
            kind: 'event',
            event: {
                kind: 'pointer',
                windowId,
                pointerKind: pointerKind.value,
                pointerId: value2,
                button: button.value,
                pointXMilli,
                pointYMilli,
            },
        };
    }
    if (kind === GUI_WEB_EVENT_KIND_KEYBOARD) {
        const keyboardKind = guiWebSharedKeyboardKindFromRaw(value1);
        if (keyboardKind.kind === 'err') {
            return { kind: 'invalid', rawKind: kind };
        }
        if (value0 <= 0 || value2 < 0) {
            return { kind: 'invalid', rawKind: kind };
        }
        return {
            kind: 'event',
            event: {
                kind: 'keyboard',
                windowId,
                keyboardKind: keyboardKind.value,
                keyCode: value0,
                modifierBits: value2,
            },
        };
    }
    if (kind === GUI_WEB_EVENT_KIND_TEXT_INPUT) {
        if (!guiWebSharedIsUnicodeScalarValue(value0)) {
            return { kind: 'invalid', rawKind: kind };
        }
        return {
            kind: 'event',
            event: {
                kind: 'text-input',
                windowId,
                scalarValue: value0,
            },
        };
    }
    if (kind === GUI_WEB_EVENT_KIND_WINDOW) {
        const windowKind = guiWebSharedWindowKindFromRaw(value0);
        if (windowKind.kind === 'err' || value1 <= 0 || value2 <= 0) {
            return { kind: 'invalid', rawKind: kind };
        }
        return {
            kind: 'event',
            event: {
                kind: 'window',
                windowId,
                windowKind: windowKind.value,
                width: value1,
                height: value2,
            },
        };
    }
    return { kind: 'invalid', rawKind: kind };
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
    const first = takeGuiWebSharedActionId(buffer);
    if (first !== 0) {
        return first;
    }
    const queue = new Int32Array(buffer);
    const writeIndex = Atomics.load(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX);
    try {
        Atomics.wait(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX, writeIndex, timeoutMs);
    } catch {
        return 0;
    }
    return takeGuiWebSharedActionId(buffer);
}

export function guiWebSharedEventQueueInt32Length(): number {
    return GUI_WEB_EVENT_QUEUE_HEADER_LENGTH
        + GUI_WEB_EVENT_QUEUE_CAPACITY * GUI_WEB_EVENT_QUEUE_SLOT_LENGTH
        + GUI_WEB_ACTION_QUEUE_CAPACITY;
}

export function guiWebSharedPointerKindToRaw(kind: GuiWebPointerEventKind): number {
    switch (kind) {
        case 'move':
            return GUI_WEB_POINTER_KIND_MOVE;
        case 'down':
            return GUI_WEB_POINTER_KIND_DOWN;
        case 'up':
            return GUI_WEB_POINTER_KIND_UP;
        case 'cancel':
            return GUI_WEB_POINTER_KIND_CANCEL;
    }
}

export function guiWebSharedPointerButtonToRaw(button: GuiWebPointerButton): number {
    switch (button) {
        case 'none':
            return GUI_WEB_POINTER_BUTTON_NONE;
        case 'primary':
            return GUI_WEB_POINTER_BUTTON_PRIMARY;
        case 'secondary':
            return GUI_WEB_POINTER_BUTTON_SECONDARY;
        case 'middle':
            return GUI_WEB_POINTER_BUTTON_MIDDLE;
    }
}

export function guiWebSharedKeyboardKindToRaw(kind: GuiWebKeyboardEventKind): number {
    switch (kind) {
        case 'down':
            return GUI_WEB_KEYBOARD_KIND_DOWN;
        case 'up':
            return GUI_WEB_KEYBOARD_KIND_UP;
    }
}

export function guiWebSharedWindowKindToRaw(kind: GuiWebWindowEventKind): number {
    switch (kind) {
        case 'resized':
            return GUI_WEB_WINDOW_KIND_RESIZED;
        case 'focused':
            return GUI_WEB_WINDOW_KIND_FOCUSED;
        case 'unfocused':
            return GUI_WEB_WINDOW_KIND_UNFOCUSED;
        case 'close-requested':
            return GUI_WEB_WINDOW_KIND_CLOSE_REQUESTED;
    }
}

function writeGuiWebSharedActionEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'action' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const eventWrite = guiWebSharedEventWritePlan(queue);
    const actionWrite = guiWebSharedActionWritePlan(queue);
    if (eventWrite.kind === 'err' && actionWrite.kind === 'err') {
        return eventWrite;
    }

    if (eventWrite.kind === 'ok') {
        const base = guiWebSharedEventSlotBase(eventWrite.value.writeIndex);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_ACTION);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE0, event.actionId);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, Math.round(event.point.x * 1000));
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, Math.round(event.point.y * 1000));
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE1, 0);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE2, 0);
        Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE3, 0);
        Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, eventWrite.value.nextWriteIndex);
        Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    }
    if (actionWrite.kind === 'ok') {
        Atomics.store(queue, guiWebSharedActionSlotBase(actionWrite.value.writeIndex), event.actionId);
        Atomics.store(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX, actionWrite.value.nextWriteIndex);
        Atomics.notify(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX, 1);
    }
    return { kind: 'ok', value: 'queued' };
}

function writeGuiWebSharedPointerEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'pointer' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const eventWrite = guiWebSharedEventWritePlan(queue);
    if (eventWrite.kind === 'err') {
        return eventWrite;
    }

    const base = guiWebSharedEventSlotBase(eventWrite.value.writeIndex);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_POINTER);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE0, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, Math.round(event.point.x * 1000));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, Math.round(event.point.y * 1000));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE1, guiWebSharedPointerKindToRaw(event.pointerKind));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE2, event.pointerId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE3, guiWebSharedPointerButtonToRaw(event.button));
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, eventWrite.value.nextWriteIndex);
    Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    return { kind: 'ok', value: 'queued' };
}

function writeGuiWebSharedKeyboardEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'keyboard' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const eventWrite = guiWebSharedEventWritePlan(queue);
    if (eventWrite.kind === 'err') {
        return eventWrite;
    }

    const base = guiWebSharedEventSlotBase(eventWrite.value.writeIndex);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_KEYBOARD);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE0, event.keyCode);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE1, guiWebSharedKeyboardKindToRaw(event.keyboardKind));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE2, event.modifierBits);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE3, 0);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, eventWrite.value.nextWriteIndex);
    Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    return { kind: 'ok', value: 'queued' };
}

function writeGuiWebSharedTextInputEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'text-input' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const eventWrite = guiWebSharedEventWritePlan(queue);
    if (eventWrite.kind === 'err') {
        return eventWrite;
    }

    const base = guiWebSharedEventSlotBase(eventWrite.value.writeIndex);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_TEXT_INPUT);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE0, event.scalarValue);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE1, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE2, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE3, 0);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, eventWrite.value.nextWriteIndex);
    Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    return { kind: 'ok', value: 'queued' };
}

function writeGuiWebSharedWindowEvent(
    buffer: SharedArrayBuffer,
    event: Extract<GuiWebInputEvent, { kind: 'window' }>,
): GuiWebSharedEventQueueResult<'queued'> {
    const queue = new Int32Array(buffer);
    const eventWrite = guiWebSharedEventWritePlan(queue);
    if (eventWrite.kind === 'err') {
        return eventWrite;
    }

    const base = guiWebSharedEventSlotBase(eventWrite.value.writeIndex);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_KIND, GUI_WEB_EVENT_KIND_WINDOW);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_WINDOW_ID, event.windowId);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE0, guiWebSharedWindowKindToRaw(event.windowKind));
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_X_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_POINT_Y_MILLI, 0);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE1, event.size.width);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE2, event.size.height);
    Atomics.store(queue, base + GUI_WEB_EVENT_SLOT_VALUE3, 0);
    Atomics.store(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, eventWrite.value.nextWriteIndex);
    Atomics.notify(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    return { kind: 'ok', value: 'queued' };
}

function guiWebSharedEventWritePlan(queue: Int32Array): GuiWebSharedEventQueueResult<{ writeIndex: number; nextWriteIndex: number }> {
    const readIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX);
    const writeIndex = Atomics.load(queue, GUI_WEB_EVENT_QUEUE_WRITE_INDEX);
    const nextWriteIndex = guiWebSharedEventNextIndex(writeIndex);
    if (nextWriteIndex === readIndex) {
        return err('event-queue-full', '$', 'available event queue slot', 'full');
    }
    return { kind: 'ok', value: { writeIndex, nextWriteIndex } };
}

function guiWebSharedActionWritePlan(queue: Int32Array): GuiWebSharedEventQueueResult<{ writeIndex: number; nextWriteIndex: number }> {
    const readIndex = Atomics.load(queue, GUI_WEB_ACTION_QUEUE_READ_INDEX);
    const writeIndex = Atomics.load(queue, GUI_WEB_ACTION_QUEUE_WRITE_INDEX);
    const nextWriteIndex = guiWebSharedActionNextIndex(writeIndex);
    if (nextWriteIndex === readIndex) {
        return err('action-queue-full', '$.action', 'available action queue slot', 'full');
    }
    return { kind: 'ok', value: { writeIndex, nextWriteIndex } };
}

function guiWebSharedEventNextIndex(index: number): number {
    return (index + 1) % GUI_WEB_EVENT_QUEUE_CAPACITY;
}

function guiWebSharedActionNextIndex(index: number): number {
    return (index + 1) % GUI_WEB_ACTION_QUEUE_CAPACITY;
}

function guiWebSharedEventSlotBase(index: number): number {
    return GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + index * GUI_WEB_EVENT_QUEUE_SLOT_LENGTH;
}

function guiWebSharedActionSlotBase(index: number): number {
    return GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + GUI_WEB_EVENT_QUEUE_CAPACITY * GUI_WEB_EVENT_QUEUE_SLOT_LENGTH + index;
}

function guiWebSharedPointerKindFromRaw(raw: number): GuiWebSharedEventQueueResult<GuiWebPointerEventKind> {
    switch (raw) {
        case GUI_WEB_POINTER_KIND_MOVE:
            return { kind: 'ok', value: 'move' };
        case GUI_WEB_POINTER_KIND_DOWN:
            return { kind: 'ok', value: 'down' };
        case GUI_WEB_POINTER_KIND_UP:
            return { kind: 'ok', value: 'up' };
        case GUI_WEB_POINTER_KIND_CANCEL:
            return { kind: 'ok', value: 'cancel' };
        default:
            return err('unsupported-event-kind', '$.pointerKind', 'known pointer kind', String(raw));
    }
}

function guiWebSharedPointerButtonFromRaw(raw: number): GuiWebSharedEventQueueResult<GuiWebPointerButton> {
    switch (raw) {
        case GUI_WEB_POINTER_BUTTON_NONE:
            return { kind: 'ok', value: 'none' };
        case GUI_WEB_POINTER_BUTTON_PRIMARY:
            return { kind: 'ok', value: 'primary' };
        case GUI_WEB_POINTER_BUTTON_SECONDARY:
            return { kind: 'ok', value: 'secondary' };
        case GUI_WEB_POINTER_BUTTON_MIDDLE:
            return { kind: 'ok', value: 'middle' };
        default:
            return err('unsupported-event-kind', '$.button', 'known pointer button', String(raw));
    }
}

function guiWebSharedKeyboardKindFromRaw(raw: number): GuiWebSharedEventQueueResult<GuiWebKeyboardEventKind> {
    switch (raw) {
        case GUI_WEB_KEYBOARD_KIND_DOWN:
            return { kind: 'ok', value: 'down' };
        case GUI_WEB_KEYBOARD_KIND_UP:
            return { kind: 'ok', value: 'up' };
        default:
            return err('unsupported-event-kind', '$.keyboardKind', 'known keyboard kind', String(raw));
    }
}

function guiWebSharedWindowKindFromRaw(raw: number): GuiWebSharedEventQueueResult<GuiWebWindowEventKind> {
    switch (raw) {
        case GUI_WEB_WINDOW_KIND_RESIZED:
            return { kind: 'ok', value: 'resized' };
        case GUI_WEB_WINDOW_KIND_FOCUSED:
            return { kind: 'ok', value: 'focused' };
        case GUI_WEB_WINDOW_KIND_UNFOCUSED:
            return { kind: 'ok', value: 'unfocused' };
        case GUI_WEB_WINDOW_KIND_CLOSE_REQUESTED:
            return { kind: 'ok', value: 'close-requested' };
        default:
            return err('unsupported-event-kind', '$.windowKind', 'known window kind', String(raw));
    }
}

function guiWebSharedIsUnicodeScalarValue(value: number): boolean {
    return value >= 0
        && value <= 0x10FFFF
        && !(value >= 0xD800 && value <= 0xDFFF);
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

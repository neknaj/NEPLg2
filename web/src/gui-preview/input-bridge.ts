export type GuiWebInputPoint = {
    x: number;
    y: number;
};

export type GuiWebInputSize = {
    width: number;
    height: number;
};

export type GuiWebPointerEventKind =
    | 'move'
    | 'down'
    | 'up'
    | 'cancel';

export type GuiWebPointerButton =
    | 'none'
    | 'primary'
    | 'secondary'
    | 'middle';

export type GuiWebKeyboardEventKind =
    | 'down'
    | 'up';

export type GuiWebWindowEventKind =
    | 'resized'
    | 'focused'
    | 'unfocused'
    | 'close-requested';

export type GuiWebInputEvent =
    | {
        kind: 'action';
        windowId: number;
        actionId: number;
        point: GuiWebInputPoint;
    }
    | {
        kind: 'pointer';
        windowId: number;
        pointerKind: GuiWebPointerEventKind;
        pointerId: number;
        button: GuiWebPointerButton;
        point: GuiWebInputPoint;
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
        size: GuiWebInputSize;
    };

export type GuiWebInputErrorKind =
    | 'invalid-input-event'
    | 'invalid-action-event'
    | 'invalid-pointer-event'
    | 'invalid-keyboard-event'
    | 'invalid-text-input-event'
    | 'invalid-window-event';

export type GuiWebInputError = {
    kind: GuiWebInputErrorKind;
    path: string;
    expected: string;
    actual: string;
};

export type GuiWebInputResult<Value> =
    | { kind: 'ok'; value: Value }
    | { kind: 'err'; error: GuiWebInputError };

export type GuiWebInputBridge = {
    kind: 'gui-input-bridge';
    pushEvent: (input: unknown) => GuiWebInputResult<'queued'>;
    takeEvents: () => GuiWebInputResult<GuiWebInputEvent[]>;
    resetEvents: () => GuiWebInputResult<'reset'>;
};

export type GuiWebInputEventListener = {
    kind: 'gui-input-listener';
    onInputEvent: (event: GuiWebInputEvent) => void;
};

type UnknownRecord = Record<string, unknown>;

let inputEvents: GuiWebInputEvent[] = [];
let inputEventListeners: GuiWebInputEventListener[] = [];

export const guiWebInputBridge: GuiWebInputBridge = {
    kind: 'gui-input-bridge',
    pushEvent: queueGuiWebInputEvent,
    takeEvents: takeGuiWebInputEvents,
    resetEvents: resetGuiWebInputEvents,
};

export function queueGuiWebInputEvent(input: unknown): GuiWebInputResult<'queued'> {
    const event = decodeGuiWebInputEvent(input);
    if (event.kind === 'err') {
        return event;
    }
    inputEvents = guiWebInputEventsWithQueuedEvent(inputEvents, event.value);
    notifyGuiWebInputEventListeners(event.value);
    return { kind: 'ok', value: 'queued' };
}

export function takeGuiWebInputEvents(): GuiWebInputResult<GuiWebInputEvent[]> {
    const events = inputEvents;
    inputEvents = [];
    return { kind: 'ok', value: events };
}

export function resetGuiWebInputEvents(): GuiWebInputResult<'reset'> {
    inputEvents = [];
    return { kind: 'ok', value: 'reset' };
}

export function registerGuiWebInputEventListener(listener: GuiWebInputEventListener): GuiWebInputResult<'registered'> {
    inputEventListeners = [
        ...inputEventListeners,
        listener,
    ];
    return { kind: 'ok', value: 'registered' };
}

export function clearGuiWebInputEventListeners(): GuiWebInputResult<'cleared'> {
    inputEventListeners = [];
    return { kind: 'ok', value: 'cleared' };
}

export function decodeGuiWebInputEvent(input: unknown): GuiWebInputResult<GuiWebInputEvent> {
    const event = asRecord(input, '$', 'invalid-input-event', 'object input event');
    if (event.kind === 'err') {
        return event;
    }
    const kind = readString(event.value, 'kind', '$.kind', 'invalid-input-event');
    if (kind.kind === 'err') {
        return kind;
    }
    if (kind.value === 'action') {
        return decodeGuiWebActionInputEvent(event.value);
    }
    if (kind.value === 'pointer') {
        return decodeGuiWebPointerInputEvent(event.value);
    }
    if (kind.value === 'keyboard') {
        return decodeGuiWebKeyboardInputEvent(event.value);
    }
    if (kind.value === 'text-input') {
        return decodeGuiWebTextInputEvent(event.value);
    }
    if (kind.value === 'window') {
        return decodeGuiWebWindowInputEvent(event.value);
    }
    return err('invalid-input-event', '$.kind', 'action, pointer, keyboard, text-input, or window', kind.value);
}

function guiWebInputEventsWithQueuedEvent(events: GuiWebInputEvent[], event: GuiWebInputEvent): GuiWebInputEvent[] {
    if (event.kind === 'pointer' && event.pointerKind === 'move') {
        return guiWebInputEventsWithPointerMove(events, event);
    }
    return [
        ...events,
        event,
    ];
}

function guiWebInputEventsWithPointerMove(
    events: GuiWebInputEvent[],
    event: Extract<GuiWebInputEvent, { kind: 'pointer' }>,
): GuiWebInputEvent[] {
    const lastIndex = events.length - 1;
    const lastEvent = events[lastIndex];
    if (
        lastEvent
        && lastEvent.kind === 'pointer'
        && lastEvent.pointerKind === 'move'
        && lastEvent.windowId === event.windowId
        && lastEvent.pointerId === event.pointerId
        && lastEvent.button === event.button
    ) {
        return [
            ...events.slice(0, lastIndex),
            event,
        ];
    }
    return [
        ...events,
        event,
    ];
}

function decodeGuiWebActionInputEvent(event: UnknownRecord): GuiWebInputResult<GuiWebInputEvent> {
    const windowId = readPositiveInteger(event, 'windowId', '$.windowId', 'invalid-action-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const actionId = readPositiveInteger(event, 'actionId', '$.actionId', 'invalid-action-event');
    if (actionId.kind === 'err') {
        return actionId;
    }
    const point = readPoint(event, 'point', '$.point', 'invalid-action-event');
    if (point.kind === 'err') {
        return point;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'action',
            windowId: windowId.value,
            actionId: actionId.value,
            point: point.value,
        },
    };
}

function decodeGuiWebPointerInputEvent(event: UnknownRecord): GuiWebInputResult<GuiWebInputEvent> {
    const windowId = readPositiveInteger(event, 'windowId', '$.windowId', 'invalid-pointer-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const pointerKind = readPointerKind(event, 'pointerKind', '$.pointerKind');
    if (pointerKind.kind === 'err') {
        return pointerKind;
    }
    const pointerId = readPositiveInteger(event, 'pointerId', '$.pointerId', 'invalid-pointer-event');
    if (pointerId.kind === 'err') {
        return pointerId;
    }
    const button = readPointerButton(event, 'button', '$.button');
    if (button.kind === 'err') {
        return button;
    }
    const point = readPoint(event, 'point', '$.point', 'invalid-pointer-event');
    if (point.kind === 'err') {
        return point;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'pointer',
            windowId: windowId.value,
            pointerKind: pointerKind.value,
            pointerId: pointerId.value,
            button: button.value,
            point: point.value,
        },
    };
}

function decodeGuiWebKeyboardInputEvent(event: UnknownRecord): GuiWebInputResult<GuiWebInputEvent> {
    const windowId = readPositiveInteger(event, 'windowId', '$.windowId', 'invalid-keyboard-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const keyboardKind = readKeyboardKind(event, 'keyboardKind', '$.keyboardKind');
    if (keyboardKind.kind === 'err') {
        return keyboardKind;
    }
    const keyCode = readPositiveInteger(event, 'keyCode', '$.keyCode', 'invalid-keyboard-event');
    if (keyCode.kind === 'err') {
        return keyCode;
    }
    const modifierBits = readNonNegativeInteger(event, 'modifierBits', '$.modifierBits', 'invalid-keyboard-event');
    if (modifierBits.kind === 'err') {
        return modifierBits;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'keyboard',
            windowId: windowId.value,
            keyboardKind: keyboardKind.value,
            keyCode: keyCode.value,
            modifierBits: modifierBits.value,
        },
    };
}

function decodeGuiWebTextInputEvent(event: UnknownRecord): GuiWebInputResult<GuiWebInputEvent> {
    const windowId = readPositiveInteger(event, 'windowId', '$.windowId', 'invalid-text-input-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const scalarValue = readUnicodeScalarValue(event, 'scalarValue', '$.scalarValue');
    if (scalarValue.kind === 'err') {
        return scalarValue;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'text-input',
            windowId: windowId.value,
            scalarValue: scalarValue.value,
        },
    };
}

function decodeGuiWebWindowInputEvent(event: UnknownRecord): GuiWebInputResult<GuiWebInputEvent> {
    const windowId = readPositiveInteger(event, 'windowId', '$.windowId', 'invalid-window-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const windowKind = readWindowKind(event, 'windowKind', '$.windowKind');
    if (windowKind.kind === 'err') {
        return windowKind;
    }
    const size = readSize(event, 'size', '$.size', 'invalid-window-event');
    if (size.kind === 'err') {
        return size;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'window',
            windowId: windowId.value,
            windowKind: windowKind.value,
            size: size.value,
        },
    };
}

function notifyGuiWebInputEventListeners(event: GuiWebInputEvent) {
    for (const listener of inputEventListeners) {
        listener.onInputEvent(event);
    }
}

function readSize(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<GuiWebInputSize> {
    const size = readRecord(record, name, path, kind);
    if (size.kind === 'err') {
        return size;
    }
    const width = readPositiveInteger(size.value, 'width', `${path}.width`, kind);
    if (width.kind === 'err') {
        return width;
    }
    const height = readPositiveInteger(size.value, 'height', `${path}.height`, kind);
    if (height.kind === 'err') {
        return height;
    }
    return {
        kind: 'ok',
        value: {
            width: width.value,
            height: height.value,
        },
    };
}

function readPoint(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<GuiWebInputPoint> {
    const point = readRecord(record, name, path, kind);
    if (point.kind === 'err') {
        return point;
    }
    const x = readFiniteNumber(point.value, 'x', `${path}.x`, kind);
    if (x.kind === 'err') {
        return x;
    }
    const y = readFiniteNumber(point.value, 'y', `${path}.y`, kind);
    if (y.kind === 'err') {
        return y;
    }
    return {
        kind: 'ok',
        value: {
            x: x.value,
            y: y.value,
        },
    };
}

function readPointerKind(record: UnknownRecord, name: string, path: string): GuiWebInputResult<GuiWebPointerEventKind> {
    const value = readString(record, name, path, 'invalid-pointer-event');
    if (value.kind === 'err') {
        return value;
    }
    switch (value.value) {
        case 'move':
        case 'down':
        case 'up':
        case 'cancel':
            return { kind: 'ok', value: value.value };
        default:
            return err('invalid-pointer-event', path, 'move, down, up, or cancel', value.value);
    }
}

function readPointerButton(record: UnknownRecord, name: string, path: string): GuiWebInputResult<GuiWebPointerButton> {
    const value = readString(record, name, path, 'invalid-pointer-event');
    if (value.kind === 'err') {
        return value;
    }
    switch (value.value) {
        case 'none':
        case 'primary':
        case 'secondary':
        case 'middle':
            return { kind: 'ok', value: value.value };
        default:
            return err('invalid-pointer-event', path, 'none, primary, secondary, or middle', value.value);
    }
}

function readKeyboardKind(record: UnknownRecord, name: string, path: string): GuiWebInputResult<GuiWebKeyboardEventKind> {
    const value = readString(record, name, path, 'invalid-keyboard-event');
    if (value.kind === 'err') {
        return value;
    }
    switch (value.value) {
        case 'down':
        case 'up':
            return { kind: 'ok', value: value.value };
        default:
            return err('invalid-keyboard-event', path, 'down or up', value.value);
    }
}

function readWindowKind(record: UnknownRecord, name: string, path: string): GuiWebInputResult<GuiWebWindowEventKind> {
    const value = readString(record, name, path, 'invalid-window-event');
    if (value.kind === 'err') {
        return value;
    }
    switch (value.value) {
        case 'resized':
        case 'focused':
        case 'unfocused':
        case 'close-requested':
            return { kind: 'ok', value: value.value };
        default:
            return err('invalid-window-event', path, 'resized, focused, unfocused, or close-requested', value.value);
    }
}

function readRecord(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<UnknownRecord> {
    return asRecord(record[name], path, kind, 'object');
}

function readString(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<string> {
    const value = record[name];
    if (typeof value === 'string') {
        return { kind: 'ok', value };
    }
    return err(kind, path, 'string', actualType(value));
}

function readFiniteNumber(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<number> {
    const value = record[name];
    if (typeof value === 'number' && Number.isFinite(value)) {
        return { kind: 'ok', value };
    }
    return err(kind, path, 'finite number', actualType(value));
}

function readPositiveInteger(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<number> {
    const value = readFiniteNumber(record, name, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (Number.isInteger(value.value) && value.value > 0) {
        return value;
    }
    return err(kind, path, 'positive integer', String(value.value));
}

function readNonNegativeInteger(record: UnknownRecord, name: string, path: string, kind: GuiWebInputErrorKind): GuiWebInputResult<number> {
    const value = readFiniteNumber(record, name, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (Number.isInteger(value.value) && value.value >= 0) {
        return value;
    }
    return err(kind, path, 'non-negative integer', String(value.value));
}

function readUnicodeScalarValue(record: UnknownRecord, name: string, path: string): GuiWebInputResult<number> {
    const value = readNonNegativeInteger(record, name, path, 'invalid-text-input-event');
    if (value.kind === 'err') {
        return value;
    }
    if (isUnicodeScalarValue(value.value)) {
        return value;
    }
    return err('invalid-text-input-event', path, 'Unicode scalar value', String(value.value));
}

function isUnicodeScalarValue(value: number): boolean {
    return value >= 0
        && value <= 0x10FFFF
        && !(value >= 0xD800 && value <= 0xDFFF);
}

function asRecord(input: unknown, path: string, kind: GuiWebInputErrorKind, expected: string): GuiWebInputResult<UnknownRecord> {
    if (typeof input === 'object' && input !== null && !Array.isArray(input)) {
        return { kind: 'ok', value: input as UnknownRecord };
    }
    return err(kind, path, expected, actualType(input));
}

function err(kind: GuiWebInputErrorKind, path: string, expected: string, actual: string): GuiWebInputResult<never> {
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

function actualType(value: unknown): string {
    if (Array.isArray(value)) {
        return 'array';
    }
    if (value === null) {
        return 'null';
    }
    return typeof value;
}

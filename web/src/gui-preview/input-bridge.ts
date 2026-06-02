export type GuiWebInputPoint = {
    x: number;
    y: number;
};

export type GuiWebInputEvent =
    | {
        kind: 'action';
        windowId: number;
        actionId: number;
        point: GuiWebInputPoint;
    };

export type GuiWebInputErrorKind =
    | 'invalid-input-event'
    | 'invalid-action-event';

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
    inputEvents = [
        ...inputEvents,
        event.value,
    ];
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
    if (kind.value !== 'action') {
        return err('invalid-input-event', '$.kind', 'action', kind.value);
    }
    const windowId = readPositiveInteger(event.value, 'windowId', '$.windowId', 'invalid-action-event');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const actionId = readPositiveInteger(event.value, 'actionId', '$.actionId', 'invalid-action-event');
    if (actionId.kind === 'err') {
        return actionId;
    }
    const point = readPoint(event.value, 'point', '$.point');
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

function notifyGuiWebInputEventListeners(event: GuiWebInputEvent) {
    for (const listener of inputEventListeners) {
        listener.onInputEvent(event);
    }
}

function readPoint(record: UnknownRecord, name: string, path: string): GuiWebInputResult<GuiWebInputPoint> {
    const point = readRecord(record, name, path, 'invalid-action-event');
    if (point.kind === 'err') {
        return point;
    }
    const x = readFiniteNumber(point.value, 'x', `${path}.x`, 'invalid-action-event');
    if (x.kind === 'err') {
        return x;
    }
    const y = readFiniteNumber(point.value, 'y', `${path}.y`, 'invalid-action-event');
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

import { decodeGuiWebHostFrame } from './host-bridge.js';
import type { GuiPreviewCommandFrame, GuiPreviewDrawCommand } from './commands.js';
import type { GuiWebHostDecodeError, GuiWebHostResult } from './host-bridge.js';

export type GuiWebRuntimePresenter = {
    presentHostFrame: (input: unknown) => GuiWebHostResult<string>;
};

export type GuiWebRuntimePresenterState =
    | { kind: 'missing' }
    | { kind: 'mounted'; presenter: GuiWebRuntimePresenter };

export type GuiWebRuntimeBridge = {
    kind: 'gui-runtime-bridge';
    presentCommands: (input: unknown) => GuiWebRuntimeResult<string>;
    beginFrame: (input: unknown) => GuiWebRuntimeResult<number>;
    pushCommand: (input: unknown) => GuiWebRuntimeResult<'pushed'>;
    endFrame: (input: unknown) => GuiWebRuntimeResult<string>;
    discardFrame: (input: unknown) => GuiWebRuntimeResult<'discarded'>;
};

export type GuiWebRuntimeErrorKind =
    | GuiWebHostDecodeError['kind']
    | 'presenter-missing'
    | 'invalid-install-target'
    | 'invalid-frame-state';

export type GuiWebRuntimeError = {
    kind: GuiWebRuntimeErrorKind;
    path: string;
    expected: string;
    actual: string;
};

export type GuiWebRuntimeResult<Value> =
    | { kind: 'ok'; value: Value }
    | { kind: 'err'; error: GuiWebRuntimeError };

type UnknownRecord = Record<string, unknown>;

type GuiWebRuntimeBuildingFrame = {
    frameId: number;
    windowId: number;
    title: string;
    width: number;
    height: number;
    commands: GuiPreviewDrawCommand[];
};

type GuiWebRuntimeFrameStore = {
    nextFrameId: number;
    frames: GuiWebRuntimeBuildingFrame[];
};

let runtimePresenterState: GuiWebRuntimePresenterState = { kind: 'missing' };
let runtimeFrameStore: GuiWebRuntimeFrameStore = {
    nextFrameId: 1,
    frames: [],
};

export const guiWebRuntimeBridge: GuiWebRuntimeBridge = {
    kind: 'gui-runtime-bridge',
    presentCommands: presentGuiWebRuntimeFrame,
    beginFrame: beginGuiWebRuntimeFrame,
    pushCommand: pushGuiWebRuntimeCommand,
    endFrame: endGuiWebRuntimeFrame,
    discardFrame: discardGuiWebRuntimeFrame,
};

export function registerGuiWebRuntimePresenter(presenter: GuiWebRuntimePresenter): GuiWebRuntimeBridge {
    runtimePresenterState = {
        kind: 'mounted',
        presenter,
    };
    return guiWebRuntimeBridge;
}

export function clearGuiWebRuntimePresenter(presenter: GuiWebRuntimePresenter): GuiWebRuntimePresenterState {
    if (runtimePresenterState.kind === 'mounted' && runtimePresenterState.presenter === presenter) {
        runtimePresenterState = { kind: 'missing' };
    }
    return runtimePresenterState;
}

export function getGuiWebRuntimePresenterState(): GuiWebRuntimePresenterState {
    return runtimePresenterState;
}

export function presentGuiWebRuntimeFrame(input: unknown): GuiWebRuntimeResult<string> {
    if (runtimePresenterState.kind === 'missing') {
        return err('presenter-missing', '$', 'registered GUI runtime presenter', 'missing');
    }
    return runtimePresenterState.presenter.presentHostFrame(input);
}

export function beginGuiWebRuntimeFrame(input: unknown): GuiWebRuntimeResult<number> {
    const record = asRecord(input, '$', 'invalid-frame', 'frame begin object');
    if (record.kind === 'err') {
        return record;
    }
    const windowId = readPositiveInteger(record.value, 'windowId', '$.windowId', 'invalid-frame');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const title = readString(record.value, 'title', '$.title', 'invalid-frame');
    if (title.kind === 'err') {
        return title;
    }
    const width = readPositiveNumber(record.value, 'width', '$.width', 'invalid-frame');
    if (width.kind === 'err') {
        return width;
    }
    const height = readPositiveNumber(record.value, 'height', '$.height', 'invalid-frame');
    if (height.kind === 'err') {
        return height;
    }

    const frameId = runtimeFrameStore.nextFrameId;
    runtimeFrameStore = {
        nextFrameId: frameId + 1,
        frames: [
            ...runtimeFrameStore.frames,
            {
                frameId,
                windowId: windowId.value,
                title: title.value,
                width: width.value,
                height: height.value,
                commands: [],
            },
        ],
    };
    return { kind: 'ok', value: frameId };
}

export function pushGuiWebRuntimeCommand(input: unknown): GuiWebRuntimeResult<'pushed'> {
    const record = asRecord(input, '$', 'invalid-command', 'frame command object');
    if (record.kind === 'err') {
        return record;
    }
    const frameId = readPositiveInteger(record.value, 'frameId', '$.frameId', 'invalid-frame-state');
    if (frameId.kind === 'err') {
        return frameId;
    }
    const frame = lookupBuildingFrame(frameId.value);
    if (frame.kind === 'err') {
        return frame;
    }
    const command = decodeRuntimeCommand(record.value.command, '$.command');
    if (command.kind === 'err') {
        return command;
    }

    frame.value.commands.push(command.value);
    return { kind: 'ok', value: 'pushed' };
}

export function endGuiWebRuntimeFrame(input: unknown): GuiWebRuntimeResult<string> {
    const frameId = decodeRuntimeFrameId(input, 'frame end object');
    if (frameId.kind === 'err') {
        return frameId;
    }
    const frame = lookupBuildingFrame(frameId.value);
    if (frame.kind === 'err') {
        return frame;
    }
    const presentedFrame: GuiPreviewCommandFrame & { windowId: number } = {
        windowId: frame.value.windowId,
        title: frame.value.title,
        width: frame.value.width,
        height: frame.value.height,
        commands: frame.value.commands,
    };
    const presented = presentGuiWebRuntimeFrame(presentedFrame);
    if (presented.kind === 'ok') {
        removeBuildingFrame(frameId.value);
    }
    return presented;
}

export function discardGuiWebRuntimeFrame(input: unknown): GuiWebRuntimeResult<'discarded'> {
    const frameId = decodeRuntimeFrameId(input, 'frame discard object');
    if (frameId.kind === 'err') {
        return frameId;
    }
    const frame = removeBuildingFrame(frameId.value);
    if (frame.kind === 'err') {
        return frame;
    }
    return { kind: 'ok', value: 'discarded' };
}

export function resetGuiWebRuntimeFrameStore(): GuiWebRuntimeFrameStore {
    runtimeFrameStore = {
        nextFrameId: 1,
        frames: [],
    };
    return runtimeFrameStore;
}

export function installGuiWebRuntimeBridge(target: unknown): GuiWebRuntimeResult<'installed'> {
    const record = asRecord(target, '$', 'invalid-install-target', 'object target');
    if (record.kind === 'err') {
        return record;
    }
    const propertyPath = '$.neplGuiHost';
    const descriptor = Object.getOwnPropertyDescriptor(record.value, 'neplGuiHost');
    if (!descriptor && !Object.isExtensible(record.value)) {
        return err('invalid-install-target', propertyPath, 'extensible object target', 'non-extensible object');
    }
    if (descriptor && descriptor.configurable === false && descriptor.writable === false) {
        return err('invalid-install-target', propertyPath, 'writable bridge property', 'non-writable property');
    }

    try {
        Object.defineProperty(record.value, 'neplGuiHost', {
            value: guiWebRuntimeBridge,
            enumerable: false,
            configurable: true,
            writable: false,
        });
    } catch (error) {
        return err('invalid-install-target', propertyPath, 'installable bridge property', actualType(error));
    }

    return { kind: 'ok', value: 'installed' };
}

function decodeRuntimeFrameId(input: unknown, expected: string): GuiWebRuntimeResult<number> {
    const record = asRecord(input, '$', 'invalid-frame-state', expected);
    if (record.kind === 'err') {
        return record;
    }
    return readPositiveInteger(record.value, 'frameId', '$.frameId', 'invalid-frame-state');
}

function decodeRuntimeCommand(input: unknown, path: string): GuiWebRuntimeResult<GuiPreviewDrawCommand> {
    const decoded = decodeGuiWebHostFrame({
        title: 'stream-command',
        width: 1,
        height: 1,
        commands: [input],
    });
    if (decoded.kind === 'err') {
        return remapDecodeError(decoded.error, '$.commands.0', path);
    }
    return { kind: 'ok', value: decoded.value.commands[0] };
}

function lookupBuildingFrame(frameId: number): GuiWebRuntimeResult<GuiWebRuntimeBuildingFrame> {
    for (const frame of runtimeFrameStore.frames) {
        if (frame.frameId === frameId) {
            return { kind: 'ok', value: frame };
        }
    }
    return err('invalid-frame-state', '$.frameId', 'open frame id', String(frameId));
}

function removeBuildingFrame(frameId: number): GuiWebRuntimeResult<GuiWebRuntimeBuildingFrame> {
    const nextFrames: GuiWebRuntimeBuildingFrame[] = [];
    let removed: GuiWebRuntimeResult<GuiWebRuntimeBuildingFrame> = err('invalid-frame-state', '$.frameId', 'open frame id', String(frameId));
    for (const frame of runtimeFrameStore.frames) {
        if (frame.frameId === frameId) {
            removed = { kind: 'ok', value: frame };
        } else {
            nextFrames.push(frame);
        }
    }
    if (removed.kind === 'ok') {
        runtimeFrameStore = {
            nextFrameId: runtimeFrameStore.nextFrameId,
            frames: nextFrames,
        };
    }
    return removed;
}

function readString(record: UnknownRecord, name: string, path: string, kind: GuiWebRuntimeErrorKind): GuiWebRuntimeResult<string> {
    const value = record[name];
    if (typeof value === 'string') {
        return { kind: 'ok', value };
    }
    return err(kind, path, 'string', actualType(value));
}

function readNumber(record: UnknownRecord, name: string, path: string, kind: GuiWebRuntimeErrorKind): GuiWebRuntimeResult<number> {
    const value = record[name];
    if (typeof value === 'number' && Number.isFinite(value)) {
        return { kind: 'ok', value };
    }
    return err(kind, path, 'finite number', actualType(value));
}

function readPositiveNumber(record: UnknownRecord, name: string, path: string, kind: GuiWebRuntimeErrorKind): GuiWebRuntimeResult<number> {
    const value = readNumber(record, name, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (value.value > 0) {
        return value;
    }
    return err(kind, path, 'number greater than 0', String(value.value));
}

function readPositiveInteger(record: UnknownRecord, name: string, path: string, kind: GuiWebRuntimeErrorKind): GuiWebRuntimeResult<number> {
    const value = readPositiveNumber(record, name, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (Number.isInteger(value.value)) {
        return value;
    }
    return err(kind, path, 'positive integer', String(value.value));
}

function asRecord(input: unknown, path: string, kind: GuiWebRuntimeErrorKind, expected: string): GuiWebRuntimeResult<UnknownRecord> {
    if (typeof input === 'object' && input !== null && !Array.isArray(input)) {
        return { kind: 'ok', value: input as UnknownRecord };
    }
    return err(kind, path, expected, actualType(input));
}

function remapDecodeError(error: GuiWebHostDecodeError, fromPrefix: string, toPrefix: string): GuiWebRuntimeResult<never> {
    const path = error.path === fromPrefix
        ? toPrefix
        : error.path.startsWith(`${fromPrefix}.`)
            ? `${toPrefix}${error.path.slice(fromPrefix.length)}`
            : error.path;
    return err(error.kind, path, error.expected, error.actual);
}

function err(kind: GuiWebRuntimeErrorKind, path: string, expected: string, actual: string): GuiWebRuntimeResult<never> {
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

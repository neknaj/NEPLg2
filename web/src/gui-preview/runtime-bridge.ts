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
};

export type GuiWebRuntimeErrorKind =
    | GuiWebHostDecodeError['kind']
    | 'presenter-missing'
    | 'invalid-install-target';

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

let runtimePresenterState: GuiWebRuntimePresenterState = { kind: 'missing' };

export const guiWebRuntimeBridge: GuiWebRuntimeBridge = {
    kind: 'gui-runtime-bridge',
    presentCommands: presentGuiWebRuntimeFrame,
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

export function installGuiWebRuntimeBridge(target: unknown): GuiWebRuntimeResult<'installed'> {
    const record = asRecord(target, '$', 'object target');
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

function asRecord(input: unknown, path: string, expected: string): GuiWebRuntimeResult<UnknownRecord> {
    if (typeof input === 'object' && input !== null && !Array.isArray(input)) {
        return { kind: 'ok', value: input as UnknownRecord };
    }
    return err('invalid-install-target', path, expected, actualType(input));
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

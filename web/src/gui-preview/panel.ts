import {
    GuiPreviewCanvasViewport,
    renderGuiPreviewFrameToCanvas,
} from './canvas-renderer.js';
import { presentGuiPreviewCanvasBackground } from './bitmap-presenter.js';
import type { GuiPreviewCommandFrame } from './commands.js';
import { presentNewestGuiVideoMemoryFrameToCanvas } from './video-memory-presenter.js';
import {
    openGuiVideoMemorySurface,
    type GuiVideoMemoryError,
    type GuiVideoMemorySurface,
} from './video-memory-surface.js';
import { queueGuiWebInputEvent } from './input-bridge.js';
import type { GuiWebInputEvent, GuiWebKeyboardEventKind } from './input-bridge.js';
import type { GuiWebPointerButton, GuiWebPointerEventKind } from './input-bridge.js';
import type { GuiWebRuntimeErrorKind, GuiWebRuntimeResult } from './runtime-bridge.js';

type GuiHostFrameState =
    | { kind: 'none' }
    | { kind: 'command-frame'; frame: GuiPreviewCommandFrame; windowId: number }
    | { kind: 'video-memory'; buffer: SharedArrayBuffer; surface: GuiVideoMemorySurface; windowId: number };

type GuiCanvasContextState =
    | { kind: 'ready'; ctx: CanvasRenderingContext2D }
    | { kind: 'unavailable'; message: string };

type GuiActiveHostWindowLookup =
    | { kind: 'missing' }
    | { kind: 'found'; windowId: number };

type GuiWebPointerInputEvent = Extract<GuiWebInputEvent, { kind: 'pointer' }>;

type GuiHostPointerMoveState =
    | { kind: 'idle' }
    | { kind: 'scheduled'; event: GuiWebPointerInputEvent };

export type GuiPreviewDebugRecord =
    | { kind: 'waiting-for-frame' }
    | { kind: 'canvas-unavailable'; message: string }
    | { kind: 'render-error'; windowId: number; errorKind: string }
    | { kind: 'frame-presented'; windowId: number; commandCount: number; inputTargetCount: number }
    | { kind: 'video-memory-presented'; windowId: number; epoch: number; width: number; height: number; dirtyKind: string }
    | { kind: 'video-memory-error'; windowId: number; errorKind: string }
    | { kind: 'input-queued'; windowId: number; eventKind: GuiWebInputEvent['kind'] }
    | { kind: 'action-queued'; windowId: number; actionId: number }
    | { kind: 'input-error'; windowId: number; eventKind: GuiWebInputEvent['kind']; errorKind: string };

export type GuiPreviewDebugSink =
    | { kind: 'none' }
    | { kind: 'present'; report: (record: GuiPreviewDebugRecord) => void };

type GuiWebKeyCodeLookup =
    | { kind: 'mapped'; keyCode: number }
    | { kind: 'unmapped' };

type GuiWebScalarLookup =
    | { kind: 'scalar'; value: number }
    | { kind: 'none' };

export type GuiPreviewDrawableSurfaceSize = {
    width: number;
    height: number;
};

export class GuiPreviewPanel {
    contentEl: HTMLElement;
    rootEl: HTMLElement;
    canvas: HTMLCanvasElement;
    debugSink: GuiPreviewDebugSink;
    contextState: GuiCanvasContextState;
    fontSize: number;
    viewport: GuiPreviewCanvasViewport;
    hostFrame: GuiHostFrameState;
    hostPointerMove: GuiHostPointerMoveState;

    constructor(contentEl: HTMLElement, debugSink: GuiPreviewDebugSink) {
        this.contentEl = contentEl;
        this.rootEl = document.createElement('div');
        this.rootEl.className = 'gui-preview-panel';
        this.canvas = document.createElement('canvas');
        this.canvas.className = 'gui-preview-canvas';
        this.canvas.tabIndex = 0;
        this.debugSink = debugSink;
        const ctx = this.canvas.getContext('2d');
        this.contextState = ctx
            ? { kind: 'ready', ctx }
            : { kind: 'unavailable', message: 'Canvas2D unavailable' };
        this.fontSize = 14;
        this.viewport = { left: 0, top: 0, scale: 1 };
        this.hostFrame = { kind: 'none' };
        this.hostPointerMove = { kind: 'idle' };

        this.rootEl.appendChild(this.canvas);
        this.contentEl.appendChild(this.rootEl);

        this.canvas.addEventListener('click', (event) => this.handleCanvasClick(event));
        this.canvas.addEventListener('pointerdown', (event) => this.handleCanvasPointerDown(event));
        this.canvas.addEventListener('pointermove', (event) => this.handleCanvasPointerMove(event));
        this.canvas.addEventListener('pointerup', (event) => this.handleCanvasPointerUp(event));
        this.canvas.addEventListener('pointercancel', (event) => this.handleCanvasPointerCancel(event));
        this.canvas.addEventListener('mousemove', (event) => this.handleCanvasPointer(event));
        this.canvas.addEventListener('keydown', (event) => this.handleCanvasKeyDown(event));
        this.canvas.addEventListener('keyup', (event) => this.handleCanvasKeyUp(event));
        this.resizeEditor();
        this.reportDebug({ kind: 'waiting-for-frame' });
    }

    presentHostFrame(frame: GuiPreviewCommandFrame, windowId: number) {
        this.hostFrame = { kind: 'command-frame', frame, windowId };
        this.hostPointerMove = { kind: 'idle' };
        this.reportDebug({
            kind: 'frame-presented',
            windowId,
            commandCount: frame.commands.length,
            inputTargetCount: frame.inputTargets.length,
        });
        this.render();
        this.focusInputSurface();
    }

    presentVideoMemorySurface(buffer: SharedArrayBuffer, windowId: number): GuiWebRuntimeResult<string> {
        if (this.contextState.kind === 'unavailable') {
            this.reportDebug({ kind: 'canvas-unavailable', message: this.contextState.message });
            return guiPreviewRuntimeErr('video-memory-present-failed', '$.canvas', 'available Canvas2D context', this.contextState.message);
        }
        const surface = this.openVideoMemorySurface(buffer);
        if (surface.kind === 'err') {
            this.reportDebug({ kind: 'video-memory-error', windowId, errorKind: surface.error.actual });
            return surface;
        }
        const presented = presentNewestGuiVideoMemoryFrameToCanvas(this.contextState.ctx, surface.value);
        if (presented.kind === 'err') {
            this.reportDebug({ kind: 'video-memory-error', windowId, errorKind: presented.error.kind });
            return guiPreviewRuntimeErr('video-memory-present-failed', '$.buffer', 'published video memory frame', guiVideoMemoryErrorActual(presented.error));
        }
        this.hostFrame = {
            kind: 'video-memory',
            buffer,
            surface: surface.value,
            windowId,
        };
        this.hostPointerMove = { kind: 'idle' };
        this.viewport = { left: 0, top: 0, scale: 1 };
        this.reportDebug({
            kind: 'video-memory-presented',
            windowId,
            epoch: presented.value.epoch,
            width: presented.value.width,
            height: presented.value.height,
            dirtyKind: presented.value.dirty.kind,
        });
        this.focusInputSurface();
        return { kind: 'ok', value: 'video-memory-presented' };
    }

    focusInputSurface() {
        if (this.activeHostWindow().kind === 'found') {
            this.canvas.focus({ preventScroll: true });
        }
    }

    setFontSize(size: number) {
        this.fontSize = size;
        this.render();
    }

    resizeEditor() {
        const size = this.drawableSurfaceCssSize();
        const pixelRatio = window.devicePixelRatio || 1;
        this.canvas.width = Math.max(1, Math.floor(size.width * pixelRatio));
        this.canvas.height = Math.max(1, Math.floor(size.height * pixelRatio));
        this.render();
    }

    render() {
        const size = this.drawableSurfaceCssSize();
        if (this.contextState.kind === 'unavailable') {
            this.reportDebug({ kind: 'canvas-unavailable', message: this.contextState.message });
            return;
        }
        const ctx = this.contextState.ctx;
        if (this.hostFrame.kind === 'command-frame') {
            const rendered = renderGuiPreviewFrameToCanvas(ctx, this.hostFrame.frame, size.width, size.height, { fontSize: this.fontSize });
            if (rendered.kind === 'err') {
                this.reportDebug({
                    kind: 'render-error',
                    windowId: this.hostFrame.windowId,
                    errorKind: rendered.error.kind,
                });
                return;
            }
            this.viewport = rendered.viewport;
            return;
        }
        if (this.hostFrame.kind === 'video-memory') {
            return;
        }
        presentGuiPreviewCanvasBackground(ctx);
    }

    drawableSurfaceCssSize(): GuiPreviewDrawableSurfaceSize {
        const rect = this.canvas.getBoundingClientRect();
        const width = rect.width > 0
            ? rect.width
            : this.canvas.clientWidth;
        const height = rect.height > 0
            ? rect.height
            : this.canvas.clientHeight;
        return {
            width: Math.max(1, Math.floor(width)),
            height: Math.max(1, Math.floor(height)),
        };
    }

    handleCanvasClick(event: MouseEvent) {
        if (this.hostFrame.kind !== 'command-frame') {
            return;
        }
        const point = this.toScenePoint(event);
        const target = this.hitHostInputTarget(this.hostFrame.frame, point);
        if (target.kind !== 'found') {
            return;
        }
        const queued = queueGuiWebInputEvent({
            kind: 'action',
            windowId: this.hostFrame.windowId,
            actionId: target.actionId,
            point,
        });
        if (queued.kind === 'ok') {
            this.reportDebug({
                kind: 'action-queued',
                windowId: this.hostFrame.windowId,
                actionId: target.actionId,
            });
        } else {
            this.reportInputError(this.hostFrame.windowId, 'action', queued.error.kind);
        }
    }

    handleCanvasPointer(event: MouseEvent) {
        if (this.hostFrame.kind !== 'command-frame') {
            this.canvas.style.cursor = 'default';
            return;
        }
        const point = this.toScenePoint(event);
        const target = this.hitHostInputTarget(this.hostFrame.frame, point);
        this.canvas.style.cursor = target.kind === 'found' ? 'pointer' : 'default';
    }

    handleCanvasPointerDown(event: PointerEvent) {
        this.queueHostPointerEvent(event, 'down');
    }

    handleCanvasPointerMove(event: PointerEvent) {
        this.queueHostPointerMoveEvent(event);
    }

    handleCanvasPointerUp(event: PointerEvent) {
        this.queueHostPointerEvent(event, 'up');
    }

    handleCanvasPointerCancel(event: PointerEvent) {
        this.queueHostPointerEvent(event, 'cancel');
    }

    queueHostPointerEvent(event: PointerEvent, pointerKind: GuiWebPointerEventKind) {
        const active = this.activeHostWindow();
        if (active.kind === 'missing') {
            return;
        }
        this.flushHostPointerMoveEvent();
        this.focusInputSurface();
        const point = this.toScenePoint(event);
        const queued = queueGuiWebInputEvent({
            kind: 'pointer',
            windowId: active.windowId,
            pointerKind,
            pointerId: event.pointerId,
            button: guiWebPointerButtonFromDomButton(event.button),
            point,
        });
        if (queued.kind === 'err') {
            this.reportInputError(active.windowId, 'pointer', queued.error.kind);
            return;
        }
        this.reportInputQueued(active.windowId, 'pointer');
    }

    queueHostPointerMoveEvent(event: PointerEvent) {
        const active = this.activeHostWindow();
        if (active.kind === 'missing') {
            return;
        }
        const shouldSchedule = this.hostPointerMove.kind === 'idle';
        this.hostPointerMove = {
            kind: 'scheduled',
            event: {
                kind: 'pointer',
                windowId: active.windowId,
                pointerKind: 'move',
                pointerId: event.pointerId,
                button: guiWebPointerButtonFromDomButtons(event.buttons),
                point: this.toScenePoint(event),
            },
        };
        if (shouldSchedule) {
            window.requestAnimationFrame(() => this.flushHostPointerMoveEvent());
        }
    }

    flushHostPointerMoveEvent() {
        const pending = this.hostPointerMove;
        this.hostPointerMove = { kind: 'idle' };
        if (pending.kind === 'idle') {
            return;
        }
        const queued = queueGuiWebInputEvent(pending.event);
        if (queued.kind === 'err') {
            this.reportInputError(pending.event.windowId, 'pointer', queued.error.kind);
            return;
        }
        this.reportInputQueued(pending.event.windowId, 'pointer');
    }

    handleCanvasKeyDown(event: KeyboardEvent) {
        if (this.activeHostWindow().kind === 'missing') {
            return;
        }
        const queuedKeyboard = this.queueHostKeyboardEvent(event, 'down');
        const queuedText = this.queueHostTextInputEvent(event);
        if (queuedKeyboard || queuedText) {
            event.preventDefault();
            event.stopPropagation();
        }
    }

    handleCanvasKeyUp(event: KeyboardEvent) {
        if (this.activeHostWindow().kind === 'missing') {
            return;
        }
        const queuedKeyboard = this.queueHostKeyboardEvent(event, 'up');
        if (queuedKeyboard) {
            event.preventDefault();
            event.stopPropagation();
        }
    }

    queueHostKeyboardEvent(event: KeyboardEvent, keyboardKind: GuiWebKeyboardEventKind): boolean {
        const active = this.activeHostWindow();
        if (active.kind === 'missing' || event.metaKey) {
            return false;
        }
        const keyCode = guiWebKeyCodeFromDomKey(event.key);
        if (keyCode.kind === 'unmapped') {
            return false;
        }
        const queued = queueGuiWebInputEvent({
            kind: 'keyboard',
            windowId: active.windowId,
            keyboardKind,
            keyCode: keyCode.keyCode,
            modifierBits: guiWebModifierBitsFromDomEvent(event),
        });
        if (queued.kind === 'err') {
            this.reportInputError(active.windowId, 'keyboard', queued.error.kind);
            return false;
        }
        this.reportInputQueued(active.windowId, 'keyboard');
        return true;
    }

    queueHostTextInputEvent(event: KeyboardEvent): boolean {
        const active = this.activeHostWindow();
        if (active.kind === 'missing' || event.isComposing || event.ctrlKey || event.altKey || event.metaKey) {
            return false;
        }
        const scalar = guiWebSingleScalarFromDomKey(event.key);
        if (scalar.kind === 'none') {
            return false;
        }
        const queued = queueGuiWebInputEvent({
            kind: 'text-input',
            windowId: active.windowId,
            scalarValue: scalar.value,
        });
        if (queued.kind === 'err') {
            this.reportInputError(active.windowId, 'text-input', queued.error.kind);
            return false;
        }
        this.reportInputQueued(active.windowId, 'text-input');
        return true;
    }

    toScenePoint(event: MouseEvent): { x: number; y: number } {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: (event.clientX - rect.left - this.viewport.left) / this.viewport.scale,
            y: (event.clientY - rect.top - this.viewport.top) / this.viewport.scale,
        };
    }

    hitHostInputTarget(frame: GuiPreviewCommandFrame, point: { x: number; y: number }): { kind: 'missing' } | { kind: 'found'; actionId: number } {
        for (const target of frame.inputTargets) {
            if (
                point.x >= target.rect.x
                && point.y >= target.rect.y
                && point.x < target.rect.x + target.rect.width
                && point.y < target.rect.y + target.rect.height
            ) {
                return { kind: 'found', actionId: target.actionId };
            }
        }
        return { kind: 'missing' };
    }

    dispose() {
        this.hostPointerMove = { kind: 'idle' };
        this.rootEl.remove();
    }

    private openVideoMemorySurface(buffer: SharedArrayBuffer): GuiWebRuntimeResult<GuiVideoMemorySurface> {
        if (this.hostFrame.kind === 'video-memory' && this.hostFrame.buffer === buffer) {
            return { kind: 'ok', value: this.hostFrame.surface };
        }
        const opened = openGuiVideoMemorySurface(buffer);
        if (opened.kind === 'ok') {
            return opened;
        }
        return guiPreviewRuntimeErr('video-memory-open-failed', '$.buffer', 'valid video memory surface', guiVideoMemoryErrorActual(opened.error));
    }

    private activeHostWindow(): GuiActiveHostWindowLookup {
        switch (this.hostFrame.kind) {
            case 'none':
                return { kind: 'missing' };
            case 'command-frame':
            case 'video-memory':
                return { kind: 'found', windowId: this.hostFrame.windowId };
        }
    }

    private reportInputQueued(windowId: number, eventKind: GuiWebInputEvent['kind']) {
        this.reportDebug({ kind: 'input-queued', windowId, eventKind });
    }

    private reportInputError(windowId: number, eventKind: GuiWebInputEvent['kind'], errorKind: string) {
        this.reportDebug({ kind: 'input-error', windowId, eventKind, errorKind });
    }

    private reportDebug(record: GuiPreviewDebugRecord) {
        if (this.debugSink.kind === 'present') {
            this.debugSink.report(record);
        }
    }
}

function guiWebPointerButtonFromDomButton(button: number): GuiWebPointerButton {
    switch (button) {
        case 0:
            return 'primary';
        case 1:
            return 'middle';
        case 2:
            return 'secondary';
        default:
            return 'none';
    }
}

function guiWebPointerButtonFromDomButtons(buttons: number): GuiWebPointerButton {
    if ((buttons & 1) !== 0) {
        return 'primary';
    }
    if ((buttons & 4) !== 0) {
        return 'middle';
    }
    if ((buttons & 2) !== 0) {
        return 'secondary';
    }
    return 'none';
}

function guiWebKeyCodeFromDomKey(key: string): GuiWebKeyCodeLookup {
    switch (key) {
        case 'Tab':
            return { kind: 'mapped', keyCode: 9 };
        case 'Enter':
            return { kind: 'mapped', keyCode: 13 };
        case ' ':
        case 'Spacebar':
            return { kind: 'mapped', keyCode: 32 };
        case 'ArrowUp':
            return { kind: 'mapped', keyCode: 1001 };
        case 'ArrowDown':
            return { kind: 'mapped', keyCode: 1002 };
        case 'ArrowRight':
            return { kind: 'mapped', keyCode: 1003 };
        case 'ArrowLeft':
            return { kind: 'mapped', keyCode: 1004 };
        default:
            return { kind: 'unmapped' };
    }
}

function guiWebModifierBitsFromDomEvent(event: KeyboardEvent): number {
    let bits = 0;
    if (event.shiftKey) {
        bits |= 1;
    }
    if (event.altKey) {
        bits |= 2;
    }
    if (event.ctrlKey) {
        bits |= 4;
    }
    return bits;
}

function guiWebSingleScalarFromDomKey(key: string): GuiWebScalarLookup {
    let count = 0;
    let scalar = 0;
    for (const part of key) {
        count += 1;
        if (count === 1) {
            const code = part.codePointAt(0);
            if (typeof code !== 'number') {
                return { kind: 'none' };
            }
            scalar = code;
        }
    }
    if (count === 1) {
        return { kind: 'scalar', value: scalar };
    }
    return { kind: 'none' };
}

function guiPreviewRuntimeErr<Value>(
    kind: GuiWebRuntimeErrorKind,
    path: string,
    expected: string,
    actual: string,
): GuiWebRuntimeResult<Value> {
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

function guiVideoMemoryErrorActual(error: GuiVideoMemoryError): string {
    switch (error.kind) {
        case 'shared-buffer-unavailable':
        case 'no-writable-slot':
        case 'no-published-slot':
        case 'presenter-unavailable':
        case 'present-failed':
        case 'writer-closed':
        case 'wait-unavailable':
            return error.kind;
        case 'invalid-surface-config':
            return `${error.kind}:${error.width}x${error.height}:slots=${error.slotCount}`;
        case 'invalid-buffer-length':
            return `${error.kind}:actual=${error.actual}:minimum=${error.minimum}`;
        case 'invalid-header-magic':
        case 'unsupported-header-version':
        case 'invalid-surface-state':
        case 'unsupported-pixel-format':
            return `${error.kind}:actual=${error.actual}`;
        case 'invalid-header-layout':
            return `${error.kind}:slots=${error.slotCount}:header=${error.headerWords}:offset=${error.pixelPlaneByteOffset}:length=${error.pixelPlaneByteLength}`;
        case 'invalid-slot-state':
            return `${error.kind}:slot=${error.slotIndex}:actual=${error.actual}`;
        case 'stale-resize-generation':
            return `${error.kind}:expected=${error.expected}:actual=${error.actual}`;
        case 'invalid-dirty-region':
            return `${error.kind}:${error.x},${error.y},${error.width},${error.height}:surface=${error.surfaceWidth}x${error.surfaceHeight}`;
        case 'unsupported-stride':
            return `${error.kind}:stride=${error.strideBytes}:expected=${error.expectedStrideBytes}`;
        case 'unsupported-command':
            return `${error.kind}:${error.commandKind}`;
    }
}

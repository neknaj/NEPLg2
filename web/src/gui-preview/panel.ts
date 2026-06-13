import {
    GuiPreviewCanvasViewport,
    renderGuiPreviewFrameToCanvas,
} from './canvas-renderer.js';
import { presentGuiPreviewCanvasBackground } from './bitmap-presenter.js';
import type { GuiPreviewCommandFrame } from './commands.js';
import { queueGuiWebInputEvent } from './input-bridge.js';
import type { GuiWebInputEvent, GuiWebKeyboardEventKind } from './input-bridge.js';
import type { GuiWebPointerButton, GuiWebPointerEventKind } from './input-bridge.js';

type GuiHostFrameState =
    | { kind: 'none' }
    | { kind: 'presented'; frame: GuiPreviewCommandFrame; windowId: number };

type GuiCanvasContextState =
    | { kind: 'ready'; ctx: CanvasRenderingContext2D }
    | { kind: 'unavailable'; message: string };

type GuiWebPointerInputEvent = Extract<GuiWebInputEvent, { kind: 'pointer' }>;

type GuiHostPointerMoveState =
    | { kind: 'idle' }
    | { kind: 'scheduled'; event: GuiWebPointerInputEvent };

export type GuiPreviewDebugRecord =
    | { kind: 'waiting-for-frame' }
    | { kind: 'canvas-unavailable'; message: string }
    | { kind: 'render-error'; windowId: number; errorKind: string }
    | { kind: 'frame-presented'; windowId: number; commandCount: number; inputTargetCount: number }
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
        this.hostFrame = { kind: 'presented', frame, windowId };
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

    focusInputSurface() {
        if (this.hostFrame.kind === 'presented') {
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
        if (this.hostFrame.kind === 'presented') {
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
        if (this.hostFrame.kind !== 'presented') {
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
        if (this.hostFrame.kind !== 'presented') {
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
        if (this.hostFrame.kind !== 'presented') {
            return;
        }
        this.flushHostPointerMoveEvent();
        this.focusInputSurface();
        const point = this.toScenePoint(event);
        const queued = queueGuiWebInputEvent({
            kind: 'pointer',
            windowId: this.hostFrame.windowId,
            pointerKind,
            pointerId: event.pointerId,
            button: guiWebPointerButtonFromDomButton(event.button),
            point,
        });
        if (queued.kind === 'err') {
            this.reportInputError(this.hostFrame.windowId, 'pointer', queued.error.kind);
            return;
        }
        this.reportInputQueued(this.hostFrame.windowId, 'pointer');
    }

    queueHostPointerMoveEvent(event: PointerEvent) {
        if (this.hostFrame.kind !== 'presented') {
            return;
        }
        const shouldSchedule = this.hostPointerMove.kind === 'idle';
        this.hostPointerMove = {
            kind: 'scheduled',
            event: {
                kind: 'pointer',
                windowId: this.hostFrame.windowId,
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
        if (this.hostFrame.kind !== 'presented') {
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
        if (this.hostFrame.kind !== 'presented') {
            return;
        }
        const queuedKeyboard = this.queueHostKeyboardEvent(event, 'up');
        if (queuedKeyboard) {
            event.preventDefault();
            event.stopPropagation();
        }
    }

    queueHostKeyboardEvent(event: KeyboardEvent, keyboardKind: GuiWebKeyboardEventKind): boolean {
        if (this.hostFrame.kind !== 'presented' || event.metaKey) {
            return false;
        }
        const keyCode = guiWebKeyCodeFromDomKey(event.key);
        if (keyCode.kind === 'unmapped') {
            return false;
        }
        const queued = queueGuiWebInputEvent({
            kind: 'keyboard',
            windowId: this.hostFrame.windowId,
            keyboardKind,
            keyCode: keyCode.keyCode,
            modifierBits: guiWebModifierBitsFromDomEvent(event),
        });
        if (queued.kind === 'err') {
            this.reportInputError(this.hostFrame.windowId, 'keyboard', queued.error.kind);
            return false;
        }
        this.reportInputQueued(this.hostFrame.windowId, 'keyboard');
        return true;
    }

    queueHostTextInputEvent(event: KeyboardEvent): boolean {
        if (this.hostFrame.kind !== 'presented' || event.isComposing || event.ctrlKey || event.altKey || event.metaKey) {
            return false;
        }
        const scalar = guiWebSingleScalarFromDomKey(event.key);
        if (scalar.kind === 'none') {
            return false;
        }
        const queued = queueGuiWebInputEvent({
            kind: 'text-input',
            windowId: this.hostFrame.windowId,
            scalarValue: scalar.value,
        });
        if (queued.kind === 'err') {
            this.reportInputError(this.hostFrame.windowId, 'text-input', queued.error.kind);
            return false;
        }
        this.reportInputQueued(this.hostFrame.windowId, 'text-input');
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

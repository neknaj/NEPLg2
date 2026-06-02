import { GuiPreviewPanel } from './panel.js';
import { decodeGuiWebHostPresentedFrame } from './host-bridge.js';
import type { GuiWebHostResult } from './host-bridge.js';
import { queueGuiWebInputEvent } from './input-bridge.js';
import type { GuiWebWindowEventKind } from './input-bridge.js';
import type { GuiPreviewDebugRecord } from './panel.js';

type GuiWindowRect = {
    left: number;
    top: number;
    width: number;
    height: number;
};

type GuiWindowSource =
    | { kind: 'host-frame'; windowId: number; title: string };

type DockButtonState =
    | { kind: 'none' }
    | { kind: 'mounted'; button: HTMLButtonElement };

type WindowMode =
    | { kind: 'normal' }
    | { kind: 'minimized'; previousMode: RestorableWindowMode }
    | { kind: 'maximized'; restoreRect: GuiWindowRect };

type RestorableWindowMode =
    | { kind: 'normal' }
    | { kind: 'maximized'; restoreRect: GuiWindowRect };

type ResizeHandle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

type WindowMoveState =
    | { kind: 'idle' }
    | {
        kind: 'drag';
        id: string;
        startX: number;
        startY: number;
        initialRect: GuiWindowRect;
    }
    | {
        kind: 'resize';
        id: string;
        handle: ResizeHandle;
        startX: number;
        startY: number;
        initialRect: GuiWindowRect;
    };

type FloatingGuiWindow = {
    id: string;
    source: GuiWindowSource;
    rect: GuiWindowRect;
    mode: WindowMode;
    frameEl: HTMLElement;
    titleEl: HTMLElement;
    contentEl: HTMLElement;
    dockButton: DockButtonState;
    preview: GuiPreviewPanel;
};

type WindowLookup =
    | { kind: 'missing' }
    | { kind: 'found'; windowState: FloatingGuiWindow };

type DebugPanelMode =
    | { kind: 'collapsed' }
    | { kind: 'expanded' };

type GuiWindowDebugRecord =
    | GuiPreviewDebugRecord
    | { kind: 'window-event-queued'; windowId: number; windowKind: GuiWebWindowEventKind; width: number; height: number }
    | { kind: 'window-event-error'; windowId: number; windowKind: GuiWebWindowEventKind; errorKind: string };

const MIN_WINDOW_WIDTH = 360;
const MIN_WINDOW_HEIGHT = 260;
const WINDOW_MARGIN = 8;
const RESIZE_HANDLES: ResizeHandle[] = ['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'];

export class GuiFloatingWindowManager {
    layerEl: HTMLElement;
    dockEl: HTMLElement;
    debugPanel: GuiWindowDebugPanel;
    windows: Map<string, FloatingGuiWindow>;
    nextId: number;
    nextZIndex: number;
    activeMove: WindowMoveState;
    fontSize: number;

    constructor(layerEl: HTMLElement) {
        this.layerEl = layerEl;
        this.dockEl = document.createElement('div');
        this.dockEl.className = 'gui-window-dock';
        this.debugPanel = new GuiWindowDebugPanel();
        this.windows = new Map();
        this.nextId = 1;
        this.nextZIndex = 80;
        this.activeMove = { kind: 'idle' };
        this.fontSize = 14;
        this.layerEl.appendChild(this.dockEl);
        this.layerEl.appendChild(this.debugPanel.rootEl);

        document.addEventListener('pointermove', (event) => this.handlePointerMove(event));
        document.addEventListener('pointerup', () => this.stopMove());
        document.addEventListener('pointercancel', () => this.stopMove());
    }

    presentHostFrame(input: unknown): GuiWebHostResult<string> {
        const decoded = decodeGuiWebHostPresentedFrame(input);
        if (decoded.kind === 'err') {
            return decoded;
        }
        const source: GuiWindowSource = {
            kind: 'host-frame',
            windowId: decoded.value.windowId,
            title: decoded.value.frame.title,
        };
        const id = this.openWindow(source);
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing') {
            return {
                kind: 'err',
                error: {
                    kind: 'invalid-frame',
                    path: '$.windowId',
                    expected: 'mounted GUI window',
                    actual: id,
                },
            };
        }
        lookup.windowState.preview.presentHostFrame(decoded.value.frame, decoded.value.windowId);
        lookup.windowState.source = source;
        this.updateTitle(lookup.windowState);
        this.focusWindow(id);
        return { kind: 'ok', value: id };
    }

    closeHostFrameWindow(windowId: number): GuiWebHostResult<string> {
        if (!Number.isInteger(windowId) || windowId <= 0) {
            return {
                kind: 'err',
                error: {
                    kind: 'invalid-frame',
                    path: '$.windowId',
                    expected: 'positive integer window id',
                    actual: String(windowId),
                },
            };
        }
        const lookup = this.findHostFrameWindow(windowId);
        if (lookup.kind === 'missing') {
            return { kind: 'ok', value: `missing:${windowId}` };
        }
        this.closeWindowState(lookup.windowState, { emitCloseRequest: false });
        return { kind: 'ok', value: lookup.windowState.id };
    }

    private openWindow(source: GuiWindowSource): string {
        const existing = this.findReusableWindow(source);
        if (existing.kind === 'found') {
            existing.windowState.source = source;
            this.restoreWindow(existing.windowState.id);
            this.focusWindow(existing.windowState.id);
            return existing.windowState.id;
        }

        const id = `gui-window-${this.nextId}`;
        this.nextId += 1;
        const rect = this.defaultRect();

        const frameEl = document.createElement('section');
        frameEl.className = 'gui-floating-window';
        frameEl.dataset.guiWindowId = id;
        frameEl.tabIndex = -1;

        const titlebarEl = document.createElement('div');
        titlebarEl.className = 'gui-window-titlebar';

        const titleEl = document.createElement('span');
        titleEl.className = 'gui-window-title';

        const controlsEl = document.createElement('div');
        controlsEl.className = 'gui-window-controls';
        controlsEl.appendChild(this.createControlButton('−', 'Minimize GUI window', () => this.minimizeWindow(id)));
        controlsEl.appendChild(this.createControlButton('□', 'Maximize or restore GUI window', () => this.toggleMaximizeWindow(id)));
        controlsEl.appendChild(this.createControlButton('×', 'Close GUI window', () => this.closeWindow(id), 'close'));

        titlebarEl.appendChild(titleEl);
        titlebarEl.appendChild(controlsEl);
        frameEl.appendChild(titlebarEl);

        const contentEl = document.createElement('div');
        contentEl.className = 'gui-window-content';
        frameEl.appendChild(contentEl);

        const preview = new GuiPreviewPanel(contentEl, {
            kind: 'present',
            report: (record) => this.debugPanel.record(record),
        });
        preview.setFontSize(this.fontSize);

        const windowState: FloatingGuiWindow = {
            id,
            source,
            rect,
            mode: { kind: 'normal' },
            frameEl,
            titleEl,
            contentEl,
            dockButton: { kind: 'none' },
            preview,
        };

        this.mountResizeHandles(frameEl, windowState);
        titlebarEl.addEventListener('pointerdown', (event) => this.startDrag(event, windowState));
        titlebarEl.addEventListener('dblclick', () => this.toggleMaximizeWindow(id));
        frameEl.addEventListener('pointerdown', () => this.focusWindow(id));
        this.layerEl.appendChild(frameEl);
        this.windows.set(id, windowState);
        this.updateTitle(windowState);
        this.applyRect(windowState, rect);
        this.focusWindow(id);
        window.requestAnimationFrame(() => windowState.preview.resizeEditor());
        return id;
    }

    resizeAll() {
        for (const windowState of this.windows.values()) {
            if (windowState.mode.kind === 'maximized') {
                windowState.rect = this.maximizedRect();
            } else {
                windowState.rect = this.clampRect(windowState.rect);
            }
            this.applyRect(windowState, windowState.rect);
        }
    }

    setFontSize(size: number) {
        this.fontSize = size;
        for (const windowState of this.windows.values()) {
            windowState.preview.setFontSize(size);
        }
    }

    private sourceMatches(windowState: FloatingGuiWindow, source: GuiWindowSource): boolean {
        return windowState.source.windowId === source.windowId;
    }

    private findReusableWindow(source: GuiWindowSource): WindowLookup {
        for (const windowState of this.windows.values()) {
            if (this.sourceMatches(windowState, source)) {
                return { kind: 'found', windowState };
            }
        }
        return { kind: 'missing' };
    }

    private createControlButton(label: string, title: string, onClick: () => void, variant: 'default' | 'close' = 'default'): HTMLButtonElement {
        const button = document.createElement('button');
        button.className = variant === 'close'
            ? 'gui-window-control is-close'
            : 'gui-window-control';
        button.type = 'button';
        button.textContent = label;
        button.title = title;
        button.addEventListener('click', (event) => {
            event.stopPropagation();
            onClick();
        });
        return button;
    }

    private mountResizeHandles(frameEl: HTMLElement, windowState: FloatingGuiWindow) {
        for (const handle of RESIZE_HANDLES) {
            const handleEl = document.createElement('div');
            handleEl.className = `gui-window-resize gui-window-resize-${handle}`;
            handleEl.addEventListener('pointerdown', (event) => this.startResize(event, windowState, handle));
            frameEl.appendChild(handleEl);
        }
    }

    private startDrag(event: PointerEvent, windowState: FloatingGuiWindow) {
        if (event.target instanceof HTMLButtonElement || windowState.mode.kind === 'maximized') {
            return;
        }
        event.preventDefault();
        event.stopPropagation();
        this.focusWindow(windowState.id);
        this.activeMove = {
            kind: 'drag',
            id: windowState.id,
            startX: event.clientX,
            startY: event.clientY,
            initialRect: { ...windowState.rect },
        };
        document.body.classList.add('is-moving-gui-window');
    }

    private startResize(event: PointerEvent, windowState: FloatingGuiWindow, handle: ResizeHandle) {
        if (windowState.mode.kind === 'maximized') {
            return;
        }
        event.preventDefault();
        event.stopPropagation();
        this.focusWindow(windowState.id);
        this.activeMove = {
            kind: 'resize',
            id: windowState.id,
            handle,
            startX: event.clientX,
            startY: event.clientY,
            initialRect: { ...windowState.rect },
        };
        document.body.classList.add('is-resizing-gui-window');
    }

    private handlePointerMove(event: PointerEvent) {
        if (this.activeMove.kind === 'idle') {
            return;
        }
        const lookup = this.lookupWindow(this.activeMove.id);
        if (lookup.kind === 'missing') {
            return;
        }
        const windowState = lookup.windowState;
        event.preventDefault();
        const dx = event.clientX - this.activeMove.startX;
        const dy = event.clientY - this.activeMove.startY;
        const nextRect = this.activeMove.kind === 'drag'
            ? this.dragRect(this.activeMove.initialRect, dx, dy)
            : this.resizeRect(this.activeMove.initialRect, this.activeMove.handle, dx, dy);
        windowState.mode = { kind: 'normal' };
        windowState.frameEl.classList.remove('is-maximized');
        this.applyRect(windowState, nextRect);
    }

    private stopMove() {
        if (this.activeMove.kind === 'idle') {
            return;
        }
        this.activeMove = { kind: 'idle' };
        document.body.classList.remove('is-moving-gui-window');
        document.body.classList.remove('is-resizing-gui-window');
    }

    private dragRect(initial: GuiWindowRect, dx: number, dy: number): GuiWindowRect {
        return this.clampRect({
            ...initial,
            left: initial.left + dx,
            top: initial.top + dy,
        });
    }

    private resizeRect(initial: GuiWindowRect, handle: ResizeHandle, dx: number, dy: number): GuiWindowRect {
        let left = initial.left;
        let top = initial.top;
        let width = initial.width;
        let height = initial.height;
        if (handle.includes('e')) {
            width += dx;
        }
        if (handle.includes('s')) {
            height += dy;
        }
        if (handle.includes('w')) {
            left += dx;
            width -= dx;
        }
        if (handle.includes('n')) {
            top += dy;
            height -= dy;
        }
        const minWidth = this.minWidth();
        const minHeight = this.minHeight();
        if (width < minWidth) {
            if (handle.includes('w')) {
                left = initial.left + initial.width - minWidth;
            }
            width = minWidth;
        }
        if (height < minHeight) {
            if (handle.includes('n')) {
                top = initial.top + initial.height - minHeight;
            }
            height = minHeight;
        }
        return this.clampRect({ left, top, width, height });
    }

    private focusWindow(id: string) {
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing') {
            return;
        }
        const windowState = lookup.windowState;
        this.nextZIndex += 1;
        windowState.frameEl.style.zIndex = String(this.nextZIndex);
        for (const other of this.windows.values()) {
            other.frameEl.classList.toggle('is-active', other.id === id);
        }
        windowState.frameEl.focus({ preventScroll: true });
        windowState.preview.focusInputSurface();
    }

    private minimizeWindow(id: string) {
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing' || lookup.windowState.mode.kind === 'minimized') {
            return;
        }
        const windowState = lookup.windowState;
        windowState.mode = {
            kind: 'minimized',
            previousMode: this.restorableMode(windowState.mode),
        };
        windowState.frameEl.classList.add('is-minimized');
        windowState.frameEl.setAttribute('aria-hidden', 'true');
        this.ensureDockButton(windowState);
    }

    private restoreWindow(id: string) {
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing') {
            return;
        }
        const windowState = lookup.windowState;
        if (windowState.mode.kind === 'minimized') {
            windowState.mode = windowState.mode.previousMode;
        }
        windowState.frameEl.classList.remove('is-minimized');
        windowState.frameEl.removeAttribute('aria-hidden');
        if (windowState.dockButton.kind === 'mounted') {
            windowState.dockButton.button.remove();
            windowState.dockButton = { kind: 'none' };
        }
        this.applyRect(windowState, windowState.rect);
    }

    private restorableMode(mode: WindowMode): RestorableWindowMode {
        if (mode.kind === 'maximized') {
            return { kind: 'maximized', restoreRect: { ...mode.restoreRect } };
        }
        return { kind: 'normal' };
    }

    private toggleMaximizeWindow(id: string) {
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing') {
            return;
        }
        const windowState = lookup.windowState;
        this.restoreWindow(id);
        if (windowState.mode.kind === 'maximized') {
            const rect = windowState.mode.restoreRect;
            windowState.mode = { kind: 'normal' };
            windowState.frameEl.classList.remove('is-maximized');
            this.applyRect(windowState, rect);
            return;
        }
        windowState.mode = { kind: 'maximized', restoreRect: { ...windowState.rect } };
        windowState.frameEl.classList.add('is-maximized');
        this.applyRect(windowState, this.maximizedRect());
        this.focusWindow(id);
    }

    private closeWindow(id: string) {
        const lookup = this.lookupWindow(id);
        if (lookup.kind === 'missing') {
            return;
        }
        this.closeWindowState(lookup.windowState, { emitCloseRequest: true });
    }

    private closeWindowState(windowState: FloatingGuiWindow, options: { emitCloseRequest: boolean }) {
        if (this.activeMove.kind !== 'idle' && this.activeMove.id === windowState.id) {
            this.stopMove();
        }
        windowState.preview.dispose();
        if (windowState.dockButton.kind === 'mounted') {
            windowState.dockButton.button.remove();
        }
        windowState.frameEl.remove();
        this.windows.delete(windowState.id);
        if (options.emitCloseRequest) {
            this.queueHostWindowEvent(windowState, 'close-requested');
        }
    }

    private lookupWindow(id: string): WindowLookup {
        const windowState = this.windows.get(id);
        if (!windowState) {
            return { kind: 'missing' };
        }
        return { kind: 'found', windowState };
    }

    private findHostFrameWindow(windowId: number): WindowLookup {
        for (const windowState of this.windows.values()) {
            if (windowState.source.kind === 'host-frame' && windowState.source.windowId === windowId) {
                return { kind: 'found', windowState };
            }
        }
        return { kind: 'missing' };
    }

    private ensureDockButton(windowState: FloatingGuiWindow) {
        if (windowState.dockButton.kind === 'mounted') {
            return;
        }
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'gui-window-dock-item';
        button.textContent = this.titleFor(windowState);
        button.addEventListener('click', () => {
            this.restoreWindow(windowState.id);
            this.focusWindow(windowState.id);
        });
        windowState.dockButton = { kind: 'mounted', button };
        this.dockEl.appendChild(button);
    }

    private updateTitle(windowState: FloatingGuiWindow) {
        const title = this.titleFor(windowState);
        windowState.titleEl.textContent = title;
        if (windowState.dockButton.kind === 'mounted') {
            windowState.dockButton.button.textContent = title;
        }
    }

    private titleFor(windowState: FloatingGuiWindow): string {
        return windowState.source.title;
    }

    private applyRect(windowState: FloatingGuiWindow, rect: GuiWindowRect) {
        const previousWidth = windowState.rect.width;
        const previousHeight = windowState.rect.height;
        const next = this.clampRect(rect);
        windowState.rect = next;
        windowState.frameEl.style.left = `${next.left}px`;
        windowState.frameEl.style.top = `${next.top}px`;
        windowState.frameEl.style.width = `${next.width}px`;
        windowState.frameEl.style.height = `${next.height}px`;
        if (previousWidth !== next.width || previousHeight !== next.height) {
            this.queueHostWindowEvent(windowState, 'resized');
        }
        window.requestAnimationFrame(() => windowState.preview.resizeEditor());
    }

    private queueHostWindowEvent(windowState: FloatingGuiWindow, windowKind: GuiWebWindowEventKind) {
        if (windowState.source.kind !== 'host-frame') {
            return;
        }
        const result = queueGuiWebInputEvent({
            kind: 'window',
            windowId: windowState.source.windowId,
            windowKind,
            size: {
                width: Math.max(1, Math.floor(windowState.rect.width)),
                height: Math.max(1, Math.floor(windowState.rect.height)),
            },
        });
        if (result.kind === 'err') {
            windowState.frameEl.dataset.guiInputError = result.error.kind;
            this.debugPanel.record({
                kind: 'window-event-error',
                windowId: windowState.source.windowId,
                windowKind,
                errorKind: result.error.kind,
            });
            return;
        }
        this.debugPanel.record({
            kind: 'window-event-queued',
            windowId: windowState.source.windowId,
            windowKind,
            width: Math.max(1, Math.floor(windowState.rect.width)),
            height: Math.max(1, Math.floor(windowState.rect.height)),
        });
    }

    private defaultRect(): GuiWindowRect {
        const bounds = this.layerBounds();
        const width = Math.min(Math.max(this.minWidth(), Math.floor(bounds.width * 0.48)), Math.max(this.minWidth(), bounds.width - WINDOW_MARGIN * 2));
        const height = Math.min(Math.max(this.minHeight(), Math.floor(bounds.height * 0.56)), Math.max(this.minHeight(), bounds.height - WINDOW_MARGIN * 2));
        const offset = (this.windows.size % 6) * 28;
        return this.clampRect({
            left: 44 + offset,
            top: 36 + offset,
            width,
            height,
        });
    }

    private maximizedRect(): GuiWindowRect {
        const bounds = this.layerBounds();
        return this.clampRect({
            left: WINDOW_MARGIN,
            top: WINDOW_MARGIN,
            width: bounds.width - WINDOW_MARGIN * 2,
            height: bounds.height - WINDOW_MARGIN * 2,
        });
    }

    private clampRect(rect: GuiWindowRect): GuiWindowRect {
        const bounds = this.layerBounds();
        const minWidth = this.minWidth();
        const minHeight = this.minHeight();
        const maxWidth = Math.max(minWidth, bounds.width - WINDOW_MARGIN * 2);
        const maxHeight = Math.max(minHeight, bounds.height - WINDOW_MARGIN * 2);
        const width = Math.min(Math.max(rect.width, minWidth), maxWidth);
        const height = Math.min(Math.max(rect.height, minHeight), maxHeight);
        const leftMax = Math.max(WINDOW_MARGIN, bounds.width - width - WINDOW_MARGIN);
        const topMax = Math.max(WINDOW_MARGIN, bounds.height - height - WINDOW_MARGIN);
        return {
            left: Math.min(Math.max(WINDOW_MARGIN, rect.left), leftMax),
            top: Math.min(Math.max(WINDOW_MARGIN, rect.top), topMax),
            width,
            height,
        };
    }

    private minWidth(): number {
        const bounds = this.layerBounds();
        return Math.max(240, Math.min(MIN_WINDOW_WIDTH, bounds.width - WINDOW_MARGIN * 2));
    }

    private minHeight(): number {
        const bounds = this.layerBounds();
        return Math.max(180, Math.min(MIN_WINDOW_HEIGHT, bounds.height - WINDOW_MARGIN * 2));
    }

    private layerBounds(): { width: number; height: number } {
        const rect = this.layerEl.getBoundingClientRect();
        return {
            width: Math.max(1, rect.width || window.innerWidth),
            height: Math.max(1, rect.height || window.innerHeight),
        };
    }
}

class GuiWindowDebugPanel {
    rootEl: HTMLElement;
    toggleEl: HTMLButtonElement;
    summaryEl: HTMLElement;
    detailEl: HTMLElement;
    mode: DebugPanelMode;
    records: GuiWindowDebugRecord[];

    constructor() {
        this.rootEl = document.createElement('aside');
        this.rootEl.className = 'gui-debug-panel is-collapsed';
        this.rootEl.setAttribute('aria-label', 'GUI debug panel');
        this.rootEl.setAttribute('aria-live', 'off');
        this.toggleEl = document.createElement('button');
        this.toggleEl.type = 'button';
        this.toggleEl.className = 'gui-debug-toggle';
        this.toggleEl.textContent = 'Debug';
        this.toggleEl.setAttribute('aria-controls', 'gui-debug-detail');
        this.summaryEl = document.createElement('div');
        this.summaryEl.className = 'gui-debug-summary';
        this.summaryEl.setAttribute('aria-live', 'off');
        this.detailEl = document.createElement('div');
        this.detailEl.id = 'gui-debug-detail';
        this.detailEl.className = 'gui-debug-detail';
        this.detailEl.setAttribute('aria-live', 'off');
        this.mode = { kind: 'collapsed' };
        this.records = [];

        this.toggleEl.addEventListener('click', () => this.toggle());
        this.rootEl.appendChild(this.toggleEl);
        this.rootEl.appendChild(this.summaryEl);
        this.rootEl.appendChild(this.detailEl);
        this.render();
    }

    record(record: GuiWindowDebugRecord) {
        this.records = [record, ...this.records].slice(0, 8);
        this.render();
    }

    private toggle() {
        this.mode = this.mode.kind === 'collapsed'
            ? { kind: 'expanded' }
            : { kind: 'collapsed' };
        this.render();
    }

    private render() {
        this.rootEl.classList.toggle('is-collapsed', this.mode.kind === 'collapsed');
        this.rootEl.classList.toggle('is-expanded', this.mode.kind === 'expanded');
        this.toggleEl.setAttribute('aria-expanded', this.mode.kind === 'expanded' ? 'true' : 'false');
        this.detailEl.setAttribute('aria-hidden', this.mode.kind === 'expanded' ? 'false' : 'true');
        const latest = this.records[0];
        this.summaryEl.textContent = latest
            ? this.describeRecord(latest)
            : 'GUI queue idle';
        this.detailEl.replaceChildren(...this.records.map((record) => {
            const row = document.createElement('div');
            row.className = 'gui-debug-row';
            row.textContent = this.describeRecord(record);
            return row;
        }));
    }

    private describeRecord(record: GuiWindowDebugRecord): string {
        switch (record.kind) {
            case 'waiting-for-frame':
                return 'host frame waiting';
            case 'canvas-unavailable':
                return `canvas unavailable: ${record.message}`;
            case 'frame-presented':
                return `window ${record.windowId}: frame commands ${record.commandCount}, targets ${record.inputTargetCount}`;
            case 'input-queued':
                return `window ${record.windowId}: queued ${record.eventKind}`;
            case 'action-queued':
                return `window ${record.windowId}: queued action ${record.actionId}`;
            case 'input-error':
                return `window ${record.windowId}: ${record.eventKind} error ${record.errorKind}`;
            case 'window-event-queued':
                return `window ${record.windowId}: queued ${record.windowKind} ${record.width}x${record.height}`;
            case 'window-event-error':
                return `window ${record.windowId}: ${record.windowKind} error ${record.errorKind}`;
        }
    }
}

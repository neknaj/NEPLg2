import {
    createGuiPreviewScene,
    GuiPreviewKind,
    guiPreviewKindFromPath,
    summarizeGuiPreviewScene,
} from './renderer.js';
import {
    GuiPreviewCanvasViewport,
    renderGuiPreviewFrameToCanvas,
    renderGuiPreviewSceneToCanvas,
} from './canvas-renderer.js';
import type { GuiPreviewCommandFrame } from './commands.js';

export type GuiPreviewSource =
    | { kind: 'none' }
    | { kind: 'path'; path: string };

type GuiHostFrameState =
    | { kind: 'none' }
    | { kind: 'presented'; frame: GuiPreviewCommandFrame };

const ignoreKindChange = (_kind: GuiPreviewKind) => {};

export type GuiPreviewPanelOptions = {
    kind?: GuiPreviewKind;
    source?: GuiPreviewSource;
    onKindChange?: (kind: GuiPreviewKind) => void;
};

export class GuiPreviewPanel {
    contentEl: HTMLElement;
    rootEl: HTMLElement;
    toolbarEl: HTMLElement;
    selectEl: HTMLSelectElement;
    metricsEl: HTMLElement;
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    kind: GuiPreviewKind;
    source: GuiPreviewSource;
    counterValue: number;
    fontSize: number;
    viewport: GuiPreviewCanvasViewport;
    onKindChange: (kind: GuiPreviewKind) => void;
    hostFrame: GuiHostFrameState;

    constructor(contentEl: HTMLElement, options: GuiPreviewPanelOptions = {}) {
        this.contentEl = contentEl;
        this.rootEl = document.createElement('div');
        this.rootEl.className = 'gui-preview-panel';
        this.toolbarEl = document.createElement('div');
        this.toolbarEl.className = 'gui-preview-toolbar';
        this.selectEl = document.createElement('select');
        this.selectEl.className = 'gui-preview-select';
        this.metricsEl = document.createElement('div');
        this.metricsEl.className = 'gui-preview-metrics';
        this.canvas = document.createElement('canvas');
        this.canvas.className = 'gui-preview-canvas';
        const ctx = this.canvas.getContext('2d');
        if (!ctx) {
            throw new Error('Could not get GUI preview 2D context');
        }
        this.ctx = ctx;
        this.kind = options.kind || 'mandelbrot';
        this.source = options.source || { kind: 'none' };
        this.counterValue = 0;
        this.fontSize = 14;
        this.viewport = { left: 0, top: 0, scale: 1 };
        this.onKindChange = options.onKindChange || ignoreKindChange;
        this.hostFrame = { kind: 'none' };

        this.mountToolbar();
        this.rootEl.appendChild(this.toolbarEl);
        this.rootEl.appendChild(this.canvas);
        this.contentEl.appendChild(this.rootEl);

        this.selectEl.addEventListener('change', () => {
            this.setKind(this.selectEl.value as GuiPreviewKind);
        });
        this.canvas.addEventListener('click', (event) => this.handleCanvasClick(event));
        this.canvas.addEventListener('mousemove', (event) => this.handleCanvasPointer(event));
        this.syncSelect();
        this.resizeEditor();
    }

    setOnKindChange(handler: (kind: GuiPreviewKind) => void) {
        this.onKindChange = handler;
    }

    mountToolbar() {
        const options: Array<{ value: GuiPreviewKind; label: string }> = [
            { value: 'mandelbrot', label: 'Mandelbrot' },
            { value: 'life', label: 'Life' },
            { value: 'counter', label: 'Counter' },
        ];
        for (const option of options) {
            const optionEl = document.createElement('option');
            optionEl.value = option.value;
            optionEl.textContent = option.label;
            this.selectEl.appendChild(optionEl);
        }
        this.toolbarEl.appendChild(this.selectEl);
        this.toolbarEl.appendChild(this.metricsEl);
    }

    setSourcePath(path: string) {
        this.hostFrame = { kind: 'none' };
        this.source = { kind: 'path', path };
        this.setKind(guiPreviewKindFromPath(path));
    }

    clearSourcePath() {
        this.hostFrame = { kind: 'none' };
        this.source = { kind: 'none' };
        this.setKind('mandelbrot');
    }

    setKind(kind: GuiPreviewKind) {
        this.hostFrame = { kind: 'none' };
        this.selectEl.disabled = false;
        this.kind = kind;
        this.syncSelect();
        this.render();
        this.onKindChange(kind);
    }

    presentHostFrame(frame: GuiPreviewCommandFrame) {
        this.hostFrame = { kind: 'presented', frame };
        this.selectEl.disabled = true;
        this.render();
    }

    setFontSize(size: number) {
        this.fontSize = size;
        this.render();
    }

    syncSelect() {
        this.selectEl.value = this.kind;
    }

    resizeEditor() {
        const rect = this.canvas.getBoundingClientRect();
        const width = Math.max(1, Math.floor(rect.width));
        const height = Math.max(1, Math.floor(rect.height));
        const pixelRatio = window.devicePixelRatio || 1;
        this.canvas.width = Math.max(1, Math.floor(width * pixelRatio));
        this.canvas.height = Math.max(1, Math.floor(height * pixelRatio));
        this.ctx.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
        this.render();
    }

    render() {
        const width = this.canvas.clientWidth || Math.max(1, this.canvas.width);
        const height = this.canvas.clientHeight || Math.max(1, this.canvas.height);
        this.ctx.clearRect(0, 0, width, height);
        this.ctx.fillStyle = '#0d1117';
        this.ctx.fillRect(0, 0, width, height);
        if (this.hostFrame.kind === 'presented') {
            const rendered = renderGuiPreviewFrameToCanvas(this.ctx, this.hostFrame.frame, width, height, { fontSize: this.fontSize });
            this.viewport = rendered.viewport;
            this.metricsEl.textContent = `host commands ${this.hostFrame.frame.commands.length}`;
            return;
        }
        const scene = createGuiPreviewScene(this.kind, { kind: 'counter', counterValue: this.counterValue });
        const rendered = renderGuiPreviewSceneToCanvas(this.ctx, scene, width, height, { fontSize: this.fontSize });
        this.viewport = rendered.viewport;
        this.metricsEl.textContent = summarizeGuiPreviewScene(scene);
    }

    handleCanvasClick(event: MouseEvent) {
        if (this.hostFrame.kind === 'presented') {
            return;
        }
        const scene = createGuiPreviewScene(this.kind, { kind: 'counter', counterValue: this.counterValue });
        const point = this.toScenePoint(event);
        const hit = scene.hitTargets.find((target) => (
            point.x >= target.x
            && point.y >= target.y
            && point.x < target.x + target.width
            && point.y < target.y + target.height
        ));
        if (!hit) {
            return;
        }
        if (hit.action === 'increment-counter') {
            this.counterValue += 1;
            this.render();
        }
    }

    handleCanvasPointer(event: MouseEvent) {
        if (this.hostFrame.kind === 'presented') {
            this.canvas.style.cursor = 'default';
            return;
        }
        const scene = createGuiPreviewScene(this.kind, { kind: 'counter', counterValue: this.counterValue });
        const point = this.toScenePoint(event);
        const hasHit = scene.hitTargets.some((target) => (
            point.x >= target.x
            && point.y >= target.y
            && point.x < target.x + target.width
            && point.y < target.y + target.height
        ));
        this.canvas.style.cursor = hasHit ? 'pointer' : 'default';
    }

    toScenePoint(event: MouseEvent): { x: number; y: number } {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: (event.clientX - rect.left - this.viewport.left) / this.viewport.scale,
            y: (event.clientY - rect.top - this.viewport.top) / this.viewport.scale,
        };
    }

    dispose() {
    }
}

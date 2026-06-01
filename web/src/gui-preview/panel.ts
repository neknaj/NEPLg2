import {
    createGuiPreviewScene,
    GuiPreviewKind,
    GuiPreviewScene,
    guiPreviewKindFromPath,
    summarizeGuiPreviewScene,
} from './renderer.js';

type SceneViewport = {
    left: number;
    top: number;
    scale: number;
};

export type GuiPreviewSource =
    | { kind: 'none' }
    | { kind: 'path'; path: string };

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
    viewport: SceneViewport;
    onKindChange: (kind: GuiPreviewKind) => void;

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
        this.source = { kind: 'path', path };
        this.setKind(guiPreviewKindFromPath(path));
    }

    clearSourcePath() {
        this.source = { kind: 'none' };
        this.setKind('mandelbrot');
    }

    setKind(kind: GuiPreviewKind) {
        this.kind = kind;
        this.syncSelect();
        this.render();
        this.onKindChange(kind);
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
        const scene = createGuiPreviewScene(this.kind, { counterValue: this.counterValue });
        const width = this.canvas.clientWidth || Math.max(1, this.canvas.width);
        const height = this.canvas.clientHeight || Math.max(1, this.canvas.height);
        this.ctx.clearRect(0, 0, width, height);
        this.ctx.fillStyle = '#0d1117';
        this.ctx.fillRect(0, 0, width, height);
        this.drawScene(scene, width, height);
        this.metricsEl.textContent = summarizeGuiPreviewScene(scene);
    }

    drawScene(scene: GuiPreviewScene, width: number, height: number) {
        const padding = 18;
        const availableWidth = Math.max(1, width - padding * 2);
        const availableHeight = Math.max(1, height - padding * 2);
        const scale = Math.min(availableWidth / scene.width, availableHeight / scene.height);
        const sceneWidth = scene.width * scale;
        const sceneHeight = scene.height * scale;
        const left = Math.floor((width - sceneWidth) / 2);
        const top = Math.floor((height - sceneHeight) / 2);
        this.viewport = { left, top, scale };

        this.ctx.fillStyle = '#101820';
        this.ctx.fillRect(left - 1, top - 1, sceneWidth + 2, sceneHeight + 2);
        for (const rect of scene.rects) {
            this.ctx.fillStyle = rect.color;
            this.ctx.fillRect(
                left + rect.x * scale,
                top + rect.y * scale,
                Math.max(1, rect.width * scale),
                Math.max(1, rect.height * scale),
            );
        }

        this.ctx.textBaseline = 'top';
        for (const text of scene.texts) {
            this.ctx.fillStyle = text.color;
            this.ctx.font = `${Math.max(8, text.size * scale)}px "HackGenConsoleNF", "JetBrains Mono", Consolas, monospace`;
            this.ctx.textAlign = text.align || 'left';
            this.ctx.fillText(text.text, left + text.x * scale, top + text.y * scale);
        }

        this.ctx.textAlign = 'left';
        this.ctx.fillStyle = '#9fb1c1';
        this.ctx.font = `${Math.max(11, this.fontSize - 1)}px "HackGenConsoleNF", "JetBrains Mono", Consolas, monospace`;
        this.ctx.fillText(scene.title, 12, 10);
    }

    handleCanvasClick(event: MouseEvent) {
        const scene = createGuiPreviewScene(this.kind, { counterValue: this.counterValue });
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
        const scene = createGuiPreviewScene(this.kind, { counterValue: this.counterValue });
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
